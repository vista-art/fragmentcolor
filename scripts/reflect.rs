mod reflect {
    //! Source model shared by the build-time generators.
    //!
    //! [`Reflection::build`] walks the module tree from `src/lib.rs` once, the
    //! way rustc resolves `mod` declarations, and records every type, impl
    //! block and free function together with the `cfg` that gates it. A
    //! generator asks for the facts of one [`Build`] (the Python wheel, the
    //! wasm package, iOS or Android) and reads binding signatures, trait
    //! implementors and conversions from here. The Rust sources stay the only
    //! place an API fact is written down.

    use std::collections::{BTreeMap, BTreeSet};
    use std::fs;
    use std::path::{Path, PathBuf};

    use heck::ToLowerCamelCase;
    use proc_macro2::{Delimiter, Group, TokenStream, TokenTree};
    use quote::ToTokens;
    use syn::punctuated::Punctuated;
    use syn::{Attribute, Fields, ImplItem, Item, Meta, Signature, Token, Type};

    /// A build the generators emit code for.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Build {
        /// The desktop wheel built with the `python` feature.
        Python,
        /// The npm package, built for `wasm32` without default features.
        Wasm,
        /// The xcframework the Swift package links.
        Ios,
        /// The `.so` libraries the Kotlin package links.
        Android,
    }

    impl Build {
        /// The attribute that marks a binding impl block in this build.
        fn binding_attr(self) -> &'static str {
            match self {
                Build::Python => "pymethods",
                Build::Wasm => "wasm_bindgen",
                Build::Ios | Build::Android => "export",
            }
        }
    }

    /// A parsed `cfg(...)` predicate.
    #[derive(Clone, Debug)]
    pub enum Cfg {
        Name(String),
        Pair(String, String),
        All(Vec<Cfg>),
        Any(Vec<Cfg>),
        Not(Box<Cfg>),
        /// A predicate this model can't read. It never excludes an item.
        Unknown,
    }

    impl Cfg {
        fn parse(meta: &Meta) -> Cfg {
            match meta {
                Meta::Path(path) => Cfg::Name(path.to_token_stream().to_string()),
                Meta::NameValue(nv) => {
                    let key = nv.path.to_token_stream().to_string();
                    match plain_expr(&nv.value) {
                        syn::Expr::Lit(syn::ExprLit {
                            lit: syn::Lit::Str(s),
                            ..
                        }) => Cfg::Pair(key, s.value()),
                        _ => Cfg::Unknown,
                    }
                }
                Meta::List(list) => {
                    let Ok(inner) =
                        list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
                    else {
                        return Cfg::Unknown;
                    };
                    let inner: Vec<Cfg> = inner.iter().map(Cfg::parse).collect();
                    match list.path.to_token_stream().to_string().as_str() {
                        "all" => Cfg::All(inner),
                        "any" => Cfg::Any(inner),
                        "not" => match inner.into_iter().next() {
                            Some(only) => Cfg::Not(Box::new(only)),
                            None => Cfg::Unknown,
                        },
                        _ => Cfg::Unknown,
                    }
                }
            }
        }

        /// `Some(true)` or `Some(false)` when the predicate is decided for
        /// `build`, `None` when it depends on something the model leaves
        /// open, such as the exact desktop OS a wheel is built for.
        fn eval(&self, build: Build) -> Option<bool> {
            match self {
                Cfg::Name(name) => name_holds(name, build),
                Cfg::Pair(key, value) => pair_holds(key, value, build),
                Cfg::All(parts) => {
                    let results: Vec<Option<bool>> = parts.iter().map(|c| c.eval(build)).collect();
                    if results.contains(&Some(false)) {
                        Some(false)
                    } else if results.iter().all(|r| *r == Some(true)) {
                        Some(true)
                    } else {
                        None
                    }
                }
                Cfg::Any(parts) => {
                    let results: Vec<Option<bool>> = parts.iter().map(|c| c.eval(build)).collect();
                    if results.contains(&Some(true)) {
                        Some(true)
                    } else if results.iter().all(|r| *r == Some(false)) {
                        Some(false)
                    } else {
                        None
                    }
                }
                Cfg::Not(inner) => inner.eval(build).map(|b| !b),
                Cfg::Unknown => None,
            }
        }

        /// Whether code under this predicate can compile in `build`. Undecided
        /// predicates count as compiled, so an item is only dropped when its
        /// `cfg` rules the build out.
        pub fn holds(&self, build: Build) -> bool {
            self.eval(build) != Some(false)
        }

        fn and(&self, more: &[Attribute]) -> Cfg {
            let mut parts = vec![self.clone()];
            parts.extend(cfgs_of(more));
            Cfg::All(parts)
        }
    }

    /// The `cfg_aliases!` block in `build.rs` and the cfg names rustc sets.
    fn name_holds(name: &str, build: Build) -> Option<bool> {
        use Build::*;
        match name {
            "python" | "desktop" => Some(build == Python),
            "wasm" => Some(build == Wasm),
            "ios" => Some(build == Ios),
            "android" => Some(build == Android),
            "mobile" => Some(matches!(build, Ios | Android)),
            "dev" | "test" | "doc" | "doctest" | "miri" => Some(false),
            // The wheel builds for every desktop OS, so these stay open there.
            "apple" => match build {
                Ios => Some(true),
                Python => None,
                Wasm | Android => Some(false),
            },
            "macos" | "windows" => match build {
                Python => None,
                Wasm | Ios | Android => Some(false),
            },
            "unix" => match build {
                Ios | Android => Some(true),
                Python => None,
                Wasm => Some(false),
            },
            _ => None,
        }
    }

    fn pair_holds(key: &str, value: &str, build: Build) -> Option<bool> {
        use Build::*;
        match key {
            // maturin and the mobile scripts build with the default features
            // (plus `python` for the wheel); build_web disables them.
            "feature" => match value {
                "python" => Some(build == Python),
                "default" => Some(build != Wasm),
                v if v.starts_with("shaders") => Some(build != Wasm),
                "winit" | "network" | "uniffi/cli" => Some(false),
                _ => None,
            },
            "target_arch" => match build {
                Wasm => Some(value == "wasm32"),
                Python | Android if value == "wasm32" => Some(false),
                Python | Android => None,
                Ios => Some(value == "aarch64"),
            },
            "target_family" => match value {
                "wasm" => Some(build == Wasm),
                "unix" | "windows" => name_holds(value, build),
                _ => None,
            },
            "target_os" => match build {
                Ios => Some(value == "ios"),
                Android => Some(value == "android"),
                Wasm => Some(value == "unknown"),
                Python if matches!(value, "ios" | "android" | "unknown") => Some(false),
                Python => None,
            },
            _ => None,
        }
    }

    fn cfgs_of(attrs: &[Attribute]) -> Vec<Cfg> {
        attrs
            .iter()
            .filter(|a| a.path().is_ident("cfg"))
            .map(|a| match a.parse_args::<Meta>() {
                Ok(meta) => Cfg::parse(&meta),
                Err(_) => Cfg::Unknown,
            })
            .collect()
    }

    /// Every attribute meta that applies in `build`, with each
    /// `cfg_attr(pred, a, b)` expanded into `a` and `b` when `pred` holds.
    pub fn metas(attrs: &[Attribute], build: Build) -> Vec<Meta> {
        let mut out = Vec::new();
        for attr in attrs {
            expand_meta(&attr.meta, build, &mut out);
        }
        out
    }

    fn expand_meta(meta: &Meta, build: Build, out: &mut Vec<Meta>) {
        let Meta::List(list) = meta else {
            out.push(meta.clone());
            return;
        };
        if !list.path.is_ident("cfg_attr") {
            out.push(meta.clone());
            return;
        }
        let Ok(parts) = list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
        else {
            return;
        };
        let mut parts = parts.into_iter();
        let Some(pred) = parts.next() else {
            return;
        };
        if Cfg::parse(&pred).holds(build) {
            for part in parts {
                expand_meta(&part, build, out);
            }
        }
    }

    /// Last path segment of a meta, e.g. `pymethods` for `pyo3::pymethods`.
    pub fn meta_ident(meta: &Meta) -> String {
        meta.path()
            .segments
            .last()
            .map(|s| s.ident.to_string())
            .unwrap_or_default()
    }

    /// The tokens inside `name(...)`, empty for a bare path.
    pub fn meta_args(meta: &Meta) -> String {
        match meta {
            Meta::List(list) => list.tokens.to_string(),
            _ => String::new(),
        }
    }

    /// Splits `a , b = "x" , c = (1 , 2)` on commas outside brackets and strings.
    pub fn split_top_level(args: &str) -> Vec<String> {
        let mut parts = Vec::new();
        let mut depth = 0i32;
        let mut in_str = false;
        let mut current = String::new();
        let mut prev = '\0';
        for ch in args.chars() {
            match ch {
                '"' if prev != '\\' => in_str = !in_str,
                '(' | '[' | '{' if !in_str => depth += 1,
                ')' | ']' | '}' if !in_str => depth -= 1,
                ',' if !in_str && depth == 0 => {
                    parts.push(current.trim().to_string());
                    current.clear();
                    prev = ch;
                    continue;
                }
                _ => {}
            }
            current.push(ch);
            prev = ch;
        }
        if !current.trim().is_empty() {
            parts.push(current.trim().to_string());
        }
        parts
    }

    /// Value of `key = value` in an attribute argument list. String values
    /// lose their quotes, parenthesized values their outer parentheses.
    pub fn arg_value(args: &str, key: &str) -> Option<String> {
        for part in split_top_level(args) {
            let Some((k, v)) = part.split_once('=') else {
                continue;
            };
            if k.trim() != key {
                continue;
            }
            let v = v.trim();
            if let Some(s) = v.strip_prefix('"').and_then(|s| s.strip_suffix('"')) {
                return Some(s.to_string());
            }
            if let Some(s) = v.strip_prefix('(').and_then(|s| s.strip_suffix(')')) {
                return Some(s.trim().to_string());
            }
            return Some(v.to_string());
        }
        None
    }

    /// The page an item's `#[lsp_doc("...")]` names.
    pub fn lsp_doc(attrs: &[Attribute]) -> Option<String> {
        attrs
            .iter()
            .find(|a| a.path().is_ident("lsp_doc"))
            .and_then(|a| a.parse_args::<syn::LitStr>().ok())
            .map(|s| s.value())
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum TypeKind {
        Struct,
        Enum,
    }

    /// A struct or enum declared anywhere in the crate.
    pub struct TypeDef {
        pub ident: String,
        pub kind: TypeKind,
        pub attrs: Vec<Attribute>,
        pub cfg: Cfg,
        /// Named fields of a struct.
        pub fields: Vec<FieldDef>,
        /// Variants of an enum.
        pub variants: Vec<VariantDef>,
    }

    pub struct FieldDef {
        pub ident: Option<String>,
        pub ty: Type,
        pub attrs: Vec<Attribute>,
    }

    pub struct VariantDef {
        pub ident: String,
        pub attrs: Vec<Attribute>,
        pub cfg: Cfg,
        pub is_unit: bool,
        /// Field types, named or positional.
        pub fields: Vec<Type>,
    }

    /// An `impl` block, inherent or for a trait.
    pub struct ImplDef {
        /// The file that declares the block.
        pub file: PathBuf,
        /// Last path segment of the self type.
        /// Empty for slices, references and other types without a path.
        pub self_ident: String,
        pub self_ty: Type,
        /// First segment of a qualified self type, such as `wgpu` in `wgpu::Color`.
        pub self_root: Option<String>,
        pub is_trait_impl: bool,
        /// The implemented trait with its generic arguments, such as `From<f32>`.
        pub trait_path: Option<syn::Path>,
        pub generics: syn::Generics,
        pub attrs: Vec<Attribute>,
        pub cfg: Cfg,
        pub fns: Vec<FnDef>,
        pub consts: Vec<ConstDef>,
    }

    /// An associated `const` with a string literal value.
    pub struct ConstDef {
        pub ident: String,
        pub value: String,
        pub is_pub: bool,
        pub cfg: Cfg,
    }

    /// A `pub const NAME: &str` on a type a binding exports, keyed the way
    /// examples spell it: `Owner.NAME`.
    pub struct StrConst {
        pub owner: String,
        pub name: String,
        pub value: String,
    }

    /// A function in an `impl` block or at module level.
    pub struct FnDef {
        pub attrs: Vec<Attribute>,
        pub sig: Signature,
        pub is_pub: bool,
        pub cfg: Cfg,
    }

    impl ImplDef {
        /// Last segment of the implemented trait, such as `From`.
        pub fn trait_ident(&self) -> Option<String> {
            self.trait_path
                .as_ref()
                .and_then(|p| p.segments.last())
                .map(|s| s.ident.to_string())
        }

        /// First type argument of the implemented trait: `X` in `From<X>`.
        pub fn trait_arg(&self) -> Option<&Type> {
            let segment = self.trait_path.as_ref()?.segments.last()?;
            let syn::PathArguments::AngleBracketed(args) = &segment.arguments else {
                return None;
            };
            args.args.iter().find_map(|a| match a {
                syn::GenericArgument::Type(t) => Some(t),
                _ => None,
            })
        }
    }

    #[derive(Default)]
    pub struct Reflection {
        pub types: Vec<TypeDef>,
        pub impls: Vec<ImplDef>,
        pub free_fns: Vec<FnDef>,
        /// Names of the crate's modules, at any depth.
        modules: BTreeSet<String>,
        /// Names of the traits the crate declares.
        traits: BTreeSet<String>,
        /// Names of the public traits that carry `#[lsp_doc]`, such as `Target`.
        documented_traits: BTreeSet<String>,
        macros: Vec<MacroDef>,
        pending: Vec<Invocation>,
    }

    // syn types only implement Debug behind its `extra-traits` feature, so
    // the model reports its size instead of its contents.
    impl std::fmt::Debug for Reflection {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("Reflection")
                .field("types", &self.types.len())
                .field("impls", &self.impls.len())
                .field("free_fns", &self.free_fns.len())
                .finish()
        }
    }

    impl Reflection {
        /// Walk the crate rooted at `root/src/lib.rs`.
        pub fn build(root: &Path) -> Reflection {
            let mut model = Reflection::default();
            let lib = root.join("src/lib.rs");
            walk_file(&lib, &root.join("src"), Cfg::All(Vec::new()), &mut model);
            // Expanded items may invoke further macros; a few rounds settle it.
            for _ in 0..4 {
                let pending = std::mem::take(&mut model.pending);
                if pending.is_empty() {
                    break;
                }
                for call in pending {
                    let Some(items) = model.expand(&call) else {
                        if model.macros.iter().any(|m| m.name == call.name) {
                            println!(
                                "cargo::warning=reflect: could not expand {}! in {}",
                                call.name,
                                call.file.display()
                            );
                        }
                        continue;
                    };
                    walk_items(&items, &call.file, &call.child_dir, &call.cfg, &mut model);
                }
            }
            for call in &model.pending {
                println!(
                    "cargo::warning=reflect: {}! in {} expands to more macro calls than the model follows",
                    call.name,
                    call.file.display()
                );
            }
            model
        }

        /// Types named `ident` that compile in `build`.
        pub fn type_named(&self, ident: &str, build: Build) -> Option<&TypeDef> {
            self.types_in(build).find(|t| t.ident == ident)
        }

        /// Impls of the trait named `trait_ident` that compile in `build`.
        ///
        /// For a trait from another crate, such as `From`, an impl whose self
        /// type is written as a path into another crate (`From<Color> for
        /// wgpu::Color`) says nothing about this crate's API and is left out.
        /// A foreign type brought in by `use` and written bare still counts.
        /// Every impl of one of this crate's own traits counts.
        pub fn trait_impls(
            &self,
            trait_ident: &str,
            build: Build,
        ) -> impl Iterator<Item = &ImplDef> {
            let own_trait = self.traits.contains(trait_ident);
            self.impls.iter().filter(move |imp| {
                imp.cfg.holds(build)
                    && imp.trait_ident().as_deref() == Some(trait_ident)
                    && (own_trait || !self.is_foreign(imp))
            })
        }

        /// Whether the impl's self type is a path into another crate.
        fn is_foreign(&self, imp: &ImplDef) -> bool {
            imp.self_root.as_deref().is_some_and(|root| {
                !matches!(root, "crate" | "self" | "super" | "Self") && !self.modules.contains(root)
            })
        }

        /// Source types of the `conversion` (`From` or `TryFrom`) impls
        /// that target the type named `target`, with the impl that declares each.
        pub fn conversions_into(
            &self,
            target: &str,
            conversion: &str,
            build: Build,
        ) -> Vec<(&ImplDef, &Type)> {
            self.trait_impls(conversion, build)
                .filter(|imp| imp.self_ident == target)
                .filter_map(|imp| imp.trait_arg().map(|arg| (imp, arg)))
                .collect()
        }

        /// Methods of plain impl blocks (not bindings, not trait impls)
        /// that compile in `build`, with their impl: the canonical Rust API.
        pub fn plain_methods(&self, build: Build) -> impl Iterator<Item = (&ImplDef, &FnDef)> {
            self.impls
                .iter()
                .filter(move |imp| {
                    imp.cfg.holds(build)
                        && !imp.is_trait_impl
                        && !metas(&imp.attrs, build).iter().any(|m| {
                            matches!(
                                meta_ident(m).as_str(),
                                "pymethods" | "wasm_bindgen" | "export"
                            )
                        })
                })
                .flat_map(|imp| imp.fns.iter().map(move |f| (imp, f)))
                .filter(move |(_, f)| f.cfg.holds(build))
        }

        /// Trait methods that belong to their self type's API.
        ///
        /// That is every method of an impl of a documented trait, such as
        /// `Target::size`, and a method of any other trait, such as
        /// `Default::default`, that a binding exports on the same type
        /// without `#[doc(hidden)]`.
        pub fn api_trait_methods(&self) -> Vec<(&ImplDef, &FnDef)> {
            let exported: BTreeSet<(&str, String)> =
                [Build::Python, Build::Wasm, Build::Ios, Build::Android]
                    .into_iter()
                    .flat_map(|build| {
                        self.binding_impls(build, build.binding_attr())
                            .flat_map(move |imp| {
                                imp.fns
                                    .iter()
                                    .filter(move |f| f.cfg.holds(build) && !is_hidden(f, build))
                                    .map(move |f| (imp, fold_name(&exported_name(f, build))))
                            })
                    })
                    .map(|(imp, name)| (imp.self_ident.as_str(), name))
                    .collect();
            self.impls
                .iter()
                .filter(|imp| imp.is_trait_impl && !self.is_foreign(imp))
                .flat_map(|imp| {
                    let documented = imp
                        .trait_ident()
                        .is_some_and(|t| self.documented_traits.contains(&t));
                    let exported = &exported;
                    imp.fns
                        .iter()
                        .filter(move |f| {
                            documented
                                || exported.contains(&(
                                    imp.self_ident.as_str(),
                                    fold_name(&f.sig.ident.to_string()),
                                ))
                        })
                        .map(move |f| (imp, f))
                })
                .collect()
        }

        /// Types that compile in `build`.
        pub fn types_in(&self, build: Build) -> impl Iterator<Item = &TypeDef> {
            self.types.iter().filter(move |t| t.cfg.holds(build))
        }

        /// Impl blocks that compile in `build` and carry `binding` (such as
        /// `pymethods`) on the block, directly or through `cfg_attr`.
        pub fn binding_impls(
            &self,
            build: Build,
            binding: &'static str,
        ) -> impl Iterator<Item = &ImplDef> {
            self.impls.iter().filter(move |imp| {
                imp.cfg.holds(build)
                    && !imp.is_trait_impl
                    && metas(&imp.attrs, build)
                        .iter()
                        .any(|m| meta_ident(m) == binding)
            })
        }

        /// String constants that a type exported in `build` declares, minus the
        /// ones its binding already exposes under the same name. Examples that
        /// write `Vertex::UV0` need the literal on platforms that lack it.
        pub fn str_consts(&self, build: Build) -> Vec<StrConst> {
            let exported: BTreeSet<&str> = self
                .types_in(build)
                .filter(|t| exported_type_name(&t.attrs, &t.ident, build).is_some())
                .map(|t| t.ident.as_str())
                .collect();
            let exposed: BTreeSet<(&str, String)> = self
                .binding_impls(build, build.binding_attr())
                .flat_map(|imp| {
                    imp.fns
                        .iter()
                        .map(move |f| (imp.self_ident.as_str(), exported_name(f, build)))
                })
                .collect();
            let mut out: Vec<StrConst> = self
                .impls
                .iter()
                .filter(|imp| {
                    !imp.is_trait_impl
                        && imp.cfg.holds(build)
                        && exported.contains(imp.self_ident.as_str())
                })
                .flat_map(|imp| {
                    imp.consts
                        .iter()
                        .filter(|c| c.is_pub && c.cfg.holds(build))
                        .map(move |c| StrConst {
                            owner: imp.self_ident.clone(),
                            name: c.ident.clone(),
                            value: c.value.clone(),
                        })
                })
                .filter(|c| !exposed.contains(&(c.owner.as_str(), c.name.clone())))
                .collect();
            out.sort_by(|a, b| (&a.owner, &a.name).cmp(&(&b.owner, &b.name)));
            out
        }

        /// Module-level functions that compile in `build` and carry `binding`.
        pub fn binding_fns(
            &self,
            build: Build,
            binding: &'static str,
        ) -> impl Iterator<Item = &FnDef> {
            self.free_fns.iter().filter(move |f| {
                f.cfg.holds(build)
                    && metas(&f.attrs, build)
                        .iter()
                        .any(|m| meta_ident(m) == binding)
            })
        }
    }

    fn walk_file(file: &Path, child_dir: &Path, cfg: Cfg, model: &mut Reflection) {
        let text = fs::read_to_string(file)
            .unwrap_or_else(|e| panic!("reflect: cannot read {}: {e}", file.display()));
        let parsed = syn::parse_file(&text)
            .unwrap_or_else(|e| panic!("reflect: cannot parse {}: {e}", file.display()));
        let cfg = cfg.and(&parsed.attrs);
        walk_items(&parsed.items, file, child_dir, &cfg, model);
    }

    fn walk_items(
        items: &[Item],
        file: &Path,
        child_dir: &Path,
        cfg: &Cfg,
        model: &mut Reflection,
    ) {
        for item in items {
            match item {
                Item::Mod(m) => {
                    let mod_cfg = cfg.and(&m.attrs);
                    let name = m.ident.to_string();
                    model.modules.insert(name.clone());
                    match &m.content {
                        Some((_, inner)) => {
                            walk_items(inner, file, &child_dir.join(&name), &mod_cfg, model)
                        }
                        None => {
                            let flat = child_dir.join(format!("{name}.rs"));
                            let mod_file = if flat.exists() {
                                flat
                            } else {
                                child_dir.join(&name).join("mod.rs")
                            };
                            walk_file(&mod_file, &child_dir.join(&name), mod_cfg, model);
                        }
                    }
                }
                Item::Struct(s) => {
                    let fields = match &s.fields {
                        Fields::Named(named) => named.named.iter().map(field_def).collect(),
                        _ => Vec::new(),
                    };
                    model.types.push(TypeDef {
                        ident: s.ident.to_string(),
                        kind: TypeKind::Struct,
                        attrs: s.attrs.clone(),
                        cfg: cfg.and(&s.attrs),
                        fields,
                        variants: Vec::new(),
                    });
                }
                Item::Enum(e) => {
                    let variants = e
                        .variants
                        .iter()
                        .map(|v| VariantDef {
                            ident: v.ident.to_string(),
                            attrs: v.attrs.clone(),
                            cfg: Cfg::All(cfgs_of(&v.attrs)),
                            is_unit: matches!(v.fields, Fields::Unit),
                            fields: v.fields.iter().map(|f| f.ty.clone()).collect(),
                        })
                        .collect();
                    model.types.push(TypeDef {
                        ident: e.ident.to_string(),
                        kind: TypeKind::Enum,
                        attrs: e.attrs.clone(),
                        cfg: cfg.and(&e.attrs),
                        fields: Vec::new(),
                        variants,
                    });
                }
                Item::Impl(imp) => {
                    let (self_ident, self_root) = match strip_type(&imp.self_ty) {
                        Type::Path(p) => {
                            let segments = &p.path.segments;
                            let last = segments.last().map(|s| s.ident.to_string());
                            let root = (segments.len() > 1)
                                .then(|| segments.first().map(|s| s.ident.to_string()))
                                .flatten();
                            (last.unwrap_or_default(), root)
                        }
                        _ => (String::new(), None),
                    };
                    let impl_cfg = cfg.and(&imp.attrs);
                    let fns = imp
                        .items
                        .iter()
                        .filter_map(|it| match it {
                            ImplItem::Fn(f) => Some(FnDef {
                                attrs: f.attrs.clone(),
                                sig: f.sig.clone(),
                                is_pub: matches!(f.vis, syn::Visibility::Public(_)),
                                cfg: impl_cfg.and(&f.attrs),
                            }),
                            _ => None,
                        })
                        .collect();
                    let consts = imp
                        .items
                        .iter()
                        .filter_map(|it| match it {
                            ImplItem::Const(c) => match plain_expr(&c.expr) {
                                syn::Expr::Lit(syn::ExprLit {
                                    lit: syn::Lit::Str(value),
                                    ..
                                }) => Some(ConstDef {
                                    ident: c.ident.to_string(),
                                    value: value.value(),
                                    is_pub: matches!(c.vis, syn::Visibility::Public(_)),
                                    cfg: impl_cfg.and(&c.attrs),
                                }),
                                _ => None,
                            },
                            _ => None,
                        })
                        .collect();
                    model.impls.push(ImplDef {
                        file: file.to_path_buf(),
                        self_ident,
                        self_ty: (*imp.self_ty).clone(),
                        self_root,
                        is_trait_impl: imp.trait_.is_some(),
                        trait_path: imp.trait_.as_ref().map(|(_, path, _)| path.clone()),
                        generics: imp.generics.clone(),
                        attrs: imp.attrs.clone(),
                        cfg: impl_cfg,
                        fns,
                        consts,
                    });
                }
                Item::Trait(t) => {
                    let name = t.ident.to_string();
                    if matches!(t.vis, syn::Visibility::Public(_)) && lsp_doc(&t.attrs).is_some() {
                        model.documented_traits.insert(name.clone());
                    }
                    model.traits.insert(name);
                }
                Item::Fn(f) => model.free_fns.push(FnDef {
                    attrs: f.attrs.clone(),
                    sig: f.sig.clone(),
                    is_pub: matches!(f.vis, syn::Visibility::Public(_)),
                    cfg: cfg.and(&f.attrs),
                }),
                Item::Macro(m) => {
                    let name = m
                        .mac
                        .path
                        .segments
                        .last()
                        .map(|s| s.ident.to_string())
                        .unwrap_or_default();
                    if name == "macro_rules" {
                        if let Some(ident) = &m.ident {
                            model
                                .macros
                                .push(MacroDef::parse(ident.to_string(), &m.mac.tokens));
                        }
                    } else {
                        model.pending.push(Invocation {
                            name,
                            tokens: m.mac.tokens.clone(),
                            file: file.to_path_buf(),
                            child_dir: child_dir.to_path_buf(),
                            cfg: cfg.and(&m.attrs),
                        });
                    }
                }
                _ => {}
            }
        }
    }

    /// An expression without the parenthesis and invisible-group wrappers a
    /// macro expansion can leave around it.
    fn plain_expr(expr: &syn::Expr) -> &syn::Expr {
        match expr {
            syn::Expr::Group(g) => plain_expr(&g.expr),
            syn::Expr::Paren(p) => plain_expr(&p.expr),
            _ => expr,
        }
    }

    /// A type without the reference, parenthesis and invisible-group wrappers
    /// that macro expansion and `&T` sources add.
    pub fn strip_type(ty: &Type) -> &Type {
        match ty {
            Type::Reference(r) => strip_type(&r.elem),
            Type::Paren(p) => strip_type(&p.elem),
            Type::Group(g) => strip_type(&g.elem),
            _ => ty,
        }
    }

    /// One `macro_rules!` definition from the crate.
    struct MacroDef {
        name: String,
        arms: Vec<(Vec<Matcher>, TokenStream)>,
    }

    /// A piece of a macro arm's matcher.
    enum Matcher {
        /// `$name:kind`
        Fragment(String, String),
        /// A literal token such as `,` or `match`.
        Token(String),
        /// A delimited group, matched piece by piece.
        Group(Delimiter, Vec<Matcher>),
        /// `$( ... ) sep op`, where `sep` is an optional separator token and
        /// `op` is `*`, `+` or `?`.
        Repeat(Vec<Matcher>, Option<String>, char),
    }

    /// A macro call at item position, waiting for every definition to be read.
    struct Invocation {
        name: String,
        tokens: TokenStream,
        file: PathBuf,
        child_dir: PathBuf,
        cfg: Cfg,
    }

    /// What a call bound one fragment to: a token stream, or one capture per
    /// round of each repetition the fragment sits in.
    enum Capture {
        /// The tokens, and whether they splice in bare. Token-like fragments
        /// (`ident`, `lifetime`, `literal`, `tt`) do; the rest go in an
        /// invisible group, so an expression keeps its precedence.
        One(TokenStream, bool),
        Rounds(Vec<Capture>),
    }

    type Bindings = BTreeMap<String, Capture>;

    impl MacroDef {
        /// Reads `(matcher) => { transcriber };` arms.
        fn parse(name: String, body: &TokenStream) -> MacroDef {
            let tokens: Vec<TokenTree> = body.clone().into_iter().collect();
            let mut arms = Vec::new();
            let mut i = 0;
            while i < tokens.len() {
                let (TokenTree::Group(matcher), Some(TokenTree::Group(transcriber))) =
                    (&tokens[i], tokens.get(i + 3))
                else {
                    i += 1;
                    continue;
                };
                arms.push((matchers(&matcher.stream()), transcriber.stream()));
                i += 4;
            }
            MacroDef { name, arms }
        }
    }

    fn repeat_op(token: Option<&TokenTree>) -> Option<char> {
        match token {
            Some(TokenTree::Punct(p)) if matches!(p.as_char(), '*' | '+' | '?') => {
                Some(p.as_char())
            }
            _ => None,
        }
    }

    /// The separator and operator after a `$( ... )` group, and how many
    /// tokens they take.
    fn repeat_tail(tokens: &[TokenTree], at: usize) -> Option<(Option<TokenTree>, char, usize)> {
        match (repeat_op(tokens.get(at)), tokens.get(at)) {
            (Some(op), _) => Some((None, op, 1)),
            (None, Some(sep)) => repeat_op(tokens.get(at + 1)).map(|op| (Some(sep.clone()), op, 2)),
            (None, None) => None,
        }
    }

    fn matchers(stream: &TokenStream) -> Vec<Matcher> {
        let tokens: Vec<TokenTree> = stream.clone().into_iter().collect();
        let mut out = Vec::new();
        let mut i = 0;
        while i < tokens.len() {
            match (
                &tokens[i],
                tokens.get(i + 1),
                tokens.get(i + 2),
                tokens.get(i + 3),
            ) {
                (
                    TokenTree::Punct(dollar),
                    Some(TokenTree::Ident(name)),
                    Some(TokenTree::Punct(colon)),
                    Some(TokenTree::Ident(kind)),
                ) if dollar.as_char() == '$' && colon.as_char() == ':' => {
                    out.push(Matcher::Fragment(name.to_string(), kind.to_string()));
                    i += 4;
                }
                (TokenTree::Punct(dollar), Some(TokenTree::Group(group)), ..)
                    if dollar.as_char() == '$' && group.delimiter() == Delimiter::Parenthesis =>
                {
                    match repeat_tail(&tokens, i + 2) {
                        Some((sep, op, used)) => {
                            let sep = sep.map(|t| t.to_string());
                            out.push(Matcher::Repeat(matchers(&group.stream()), sep, op));
                            i += 2 + used;
                        }
                        None => {
                            out.push(Matcher::Token("$".into()));
                            i += 1;
                        }
                    }
                }
                (TokenTree::Group(group), ..) => {
                    out.push(Matcher::Group(group.delimiter(), matchers(&group.stream())));
                    i += 1;
                }
                (other, ..) => {
                    out.push(Matcher::Token(other.to_string()));
                    i += 1;
                }
            }
        }
        out
    }

    impl Reflection {
        /// Expand one call of a crate-local macro into the items it defines.
        fn expand(&self, call: &Invocation) -> Option<Vec<Item>> {
            use syn::parse::Parser;
            let def = self.macros.iter().find(|m| m.name == call.name)?;
            def.arms.iter().find_map(|(pattern, transcriber)| {
                let parser = |input: syn::parse::ParseStream| bind_all(input, pattern);
                let bindings = parser.parse2(call.tokens.clone()).ok()?;
                let tokens = transcribe(transcriber, &bindings, &[])?;
                syn::parse2::<syn::File>(tokens).ok().map(|f| f.items)
            })
        }
    }

    /// Parse one fragment of the given kind from `input`.
    fn fragment(input: syn::parse::ParseStream, kind: &str) -> syn::Result<TokenStream> {
        Ok(match kind {
            "ty" => input.parse::<Type>()?.to_token_stream(),
            "expr" => input.parse::<syn::Expr>()?.to_token_stream(),
            "ident" => input.parse::<syn::Ident>()?.to_token_stream(),
            "path" => input.parse::<syn::Path>()?.to_token_stream(),
            "literal" => input.parse::<syn::Lit>()?.to_token_stream(),
            "lifetime" => input.parse::<syn::Lifetime>()?.to_token_stream(),
            "pat" | "pat_param" => syn::Pat::parse_multi(input)?.to_token_stream(),
            "block" => input.parse::<syn::Block>()?.to_token_stream(),
            "item" => input.parse::<syn::Item>()?.to_token_stream(),
            "vis" => input.parse::<syn::Visibility>()?.to_token_stream(),
            "meta" => input.parse::<syn::Meta>()?.to_token_stream(),
            "tt" => input.parse::<TokenTree>()?.into(),
            _ => return Err(input.error("unsupported fragment")),
        })
    }

    /// Match the whole of `input` against `pattern`.
    fn bind_all(input: syn::parse::ParseStream, pattern: &[Matcher]) -> syn::Result<Bindings> {
        let bindings = bind_seq(input, pattern)?;
        if !input.is_empty() {
            return Err(input.error("unmatched tokens"));
        }
        Ok(bindings)
    }

    /// Match `pattern` against the front of `input`.
    fn bind_seq(input: syn::parse::ParseStream, pattern: &[Matcher]) -> syn::Result<Bindings> {
        use syn::parse::Parser;
        use syn::parse::discouraged::Speculative;
        let mut bindings = Bindings::new();
        for piece in pattern {
            match piece {
                Matcher::Fragment(name, kind) => {
                    let bare = matches!(kind.as_str(), "ident" | "lifetime" | "literal" | "tt");
                    bindings.insert(name.clone(), Capture::One(fragment(input, kind)?, bare));
                }
                Matcher::Token(expected) => {
                    let found: TokenTree = input.parse()?;
                    if found.to_string() != *expected {
                        return Err(input.error("token mismatch"));
                    }
                }
                Matcher::Group(delimiter, inner) => {
                    let group: Group = input.parse()?;
                    if group.delimiter() != *delimiter {
                        return Err(input.error("delimiter mismatch"));
                    }
                    let parser = |content: syn::parse::ParseStream| bind_all(content, inner);
                    bindings.extend(parser.parse2(group.stream())?);
                }
                Matcher::Repeat(inner, sep, op) => {
                    let mut rounds: Vec<Bindings> = Vec::new();
                    loop {
                        if *op == '?' && !rounds.is_empty() {
                            break;
                        }
                        let fork = input.fork();
                        // A separator belongs to the repetition only when another item follows it.
                        if let (Some(sep), false) = (sep, rounds.is_empty()) {
                            match fork.parse::<TokenTree>() {
                                Ok(found) if found.to_string() == *sep => {}
                                _ => break,
                            }
                        }
                        if fork.is_empty() {
                            break;
                        }
                        let Ok(round) = bind_seq(&fork, inner) else {
                            break;
                        };
                        input.advance_to(&fork);
                        rounds.push(round);
                    }
                    if *op == '+' && rounds.is_empty() {
                        return Err(input.error("repetition needs one item"));
                    }
                    let mut names = BTreeSet::new();
                    fragment_names(inner, &mut names);
                    for name in names {
                        let per_round = rounds
                            .iter_mut()
                            .map(|round| round.remove(&name).unwrap_or(Capture::Rounds(Vec::new())))
                            .collect();
                        bindings.insert(name, Capture::Rounds(per_round));
                    }
                }
            }
        }
        Ok(bindings)
    }

    /// Every fragment name a matcher binds, at any depth.
    fn fragment_names(pattern: &[Matcher], out: &mut BTreeSet<String>) {
        for piece in pattern {
            match piece {
                Matcher::Fragment(name, _) => {
                    out.insert(name.clone());
                }
                Matcher::Group(_, inner) | Matcher::Repeat(inner, ..) => fragment_names(inner, out),
                Matcher::Token(_) => {}
            }
        }
    }

    /// Names a transcriber group refers to with `$name`.
    fn referenced_names(stream: &TokenStream, out: &mut BTreeSet<String>) {
        let tokens: Vec<TokenTree> = stream.clone().into_iter().collect();
        for (i, token) in tokens.iter().enumerate() {
            match (token, tokens.get(i + 1)) {
                (TokenTree::Punct(p), Some(TokenTree::Ident(id))) if p.as_char() == '$' => {
                    out.insert(id.to_string());
                }
                (TokenTree::Group(g), _) => referenced_names(&g.stream(), out),
                _ => {}
            }
        }
    }

    /// The part of `capture` for the repetition rounds in `path`. A fragment
    /// bound outside a repetition reads the same in every round.
    fn capture_at<'a>(capture: &'a Capture, path: &[usize]) -> Option<&'a Capture> {
        match (capture, path.split_first()) {
            (Capture::Rounds(rounds), Some((k, rest))) => capture_at(rounds.get(*k)?, rest),
            (capture, _) => Some(capture),
        }
    }

    /// The transcriber with every `$name` replaced by its capture and every
    /// `$( ... )` repeated once per bound round. `path` holds the round of
    /// each enclosing repetition.
    fn transcribe(
        stream: &TokenStream,
        bindings: &Bindings,
        path: &[usize],
    ) -> Option<TokenStream> {
        let tokens: Vec<TokenTree> = stream.clone().into_iter().collect();
        let mut out: Vec<TokenTree> = Vec::new();
        let mut i = 0;
        while i < tokens.len() {
            match (&tokens[i], tokens.get(i + 1)) {
                (TokenTree::Punct(p), Some(TokenTree::Ident(id))) if p.as_char() == '$' => {
                    let name = id.to_string();
                    if name == "crate" {
                        out.push(TokenTree::Ident(id.clone()));
                    } else {
                        let Capture::One(value, bare) = capture_at(bindings.get(&name)?, path)?
                        else {
                            return None;
                        };
                        if *bare {
                            out.extend(value.clone());
                        } else {
                            out.push(Group::new(Delimiter::None, value.clone()).into());
                        }
                    }
                    i += 2;
                }
                (TokenTree::Punct(p), Some(TokenTree::Group(group)))
                    if p.as_char() == '$' && group.delimiter() == Delimiter::Parenthesis =>
                {
                    let (sep, _, used) = repeat_tail(&tokens, i + 2)?;
                    let mut names = BTreeSet::new();
                    referenced_names(&group.stream(), &mut names);
                    let count = names
                        .iter()
                        .filter_map(|n| match capture_at(bindings.get(n)?, path)? {
                            Capture::Rounds(rounds) => Some(rounds.len()),
                            Capture::One(..) => None,
                        })
                        .max()?;
                    for k in 0..count {
                        if let (Some(sep), true) = (&sep, k > 0) {
                            out.push(sep.clone());
                        }
                        let inner_path: Vec<usize> = path.iter().copied().chain([k]).collect();
                        out.extend(transcribe(&group.stream(), bindings, &inner_path)?);
                    }
                    i += 2 + used;
                }
                (TokenTree::Group(group), _) => {
                    let inner = transcribe(&group.stream(), bindings, path)?;
                    let mut copy = Group::new(group.delimiter(), inner);
                    copy.set_span(group.span());
                    out.push(copy.into());
                    i += 1;
                }
                (other, _) => {
                    out.push(other.clone());
                    i += 1;
                }
            }
        }
        Some(out.into_iter().collect())
    }

    /// The name a binding exports the type under in `build`:
    /// the `#[pyclass(name)]`, the `js_name`, or the Rust name.
    /// It is `None` when that build has no binding for the type.
    pub fn exported_type_name(attrs: &[Attribute], ident: &str, build: Build) -> Option<String> {
        let metas = metas(attrs, build);
        let binding = metas.iter().find(|m| match build {
            Build::Python => meta_ident(m) == "pyclass",
            Build::Wasm => meta_ident(m) == "wasm_bindgen",
            Build::Ios | Build::Android => {
                meta_ident(m) == "derive" && meta_args(m).contains("uniffi")
            }
        })?;
        let renamed = match build {
            Build::Python => arg_value(&meta_args(binding), "name"),
            Build::Wasm => arg_value(&meta_args(binding), "js_name"),
            Build::Ios | Build::Android => metas
                .iter()
                .filter(|m| meta_ident(m) == "uniffi")
                .find_map(|m| arg_value(&meta_args(m), "name")),
        };
        Some(renamed.unwrap_or_else(|| ident.to_string()))
    }

    /// What the `#[pymodule]` in `src/platforms/python.rs` registers, in registration order.
    pub struct PythonModule {
        /// Rust idents passed to `add_class`.
        pub classes: Vec<String>,
        /// Rust idents passed to `wrap_pyfunction!`.
        pub functions: Vec<String>,
        /// Names given to `m.add("...", ...)`.
        pub exceptions: Vec<String>,
    }

    impl PythonModule {
        pub const FILE: &'static str = "src/platforms/python.rs";

        pub fn read(root: &Path) -> PythonModule {
            let path = root.join(Self::FILE);
            let text = fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("reflect: cannot read {}: {e}", path.display()));
            let code: String = text
                .lines()
                .map(|line| line.split("//").next().unwrap_or(""))
                .collect::<Vec<_>>()
                .join("\n");
            let mut module = PythonModule {
                classes: Vec::new(),
                functions: Vec::new(),
                exceptions: Vec::new(),
            };
            let mut rest = code.as_str();
            while let Some(at) = rest.find("add_class::<") {
                rest = &rest[at + "add_class::<".len()..];
                let end = rest.find('>').unwrap_or(rest.len());
                let path = rest[..end].trim();
                module
                    .classes
                    .push(path.rsplit("::").next().unwrap_or(path).trim().to_string());
            }
            let mut rest = code.as_str();
            while let Some(at) = rest.find("wrap_pyfunction!(") {
                rest = &rest[at + "wrap_pyfunction!(".len()..];
                let end = rest.find(',').unwrap_or(rest.len());
                module.functions.push(rest[..end].trim().to_string());
            }
            let mut rest = code.as_str();
            while let Some(at) = rest.find(".add(") {
                rest = &rest[at + ".add(".len()..];
                if let Some(stripped) = rest.trim_start().strip_prefix('"')
                    && let Some(end) = stripped.find('"')
                {
                    module.exceptions.push(stripped[..end].to_string());
                }
            }
            module
        }
    }

    impl Reflection {
        /// Names Python code can import from `fragmentcolor`.
        pub fn python_exports(&self, root: &Path) -> BTreeSet<String> {
            let module = PythonModule::read(root);
            let classes = module.classes.iter().filter_map(|rust| {
                let ty = self.type_named(rust, Build::Python)?;
                exported_type_name(&ty.attrs, &ty.ident, Build::Python)
            });
            classes
                .chain(module.functions.iter().cloned())
                .chain(module.exceptions.iter().cloned())
                .collect()
        }

        /// Names JS code can import from `fragmentcolor`: exported classes and functions.
        pub fn wasm_exports(&self) -> BTreeSet<String> {
            let types = self
                .types_in(Build::Wasm)
                .filter_map(|t| exported_type_name(&t.attrs, &t.ident, Build::Wasm));
            let fns = self
                .binding_fns(Build::Wasm, "wasm_bindgen")
                .map(|f| exported_name(f, Build::Wasm));
            types.chain(fns).collect()
        }

        /// The methods of the `#[wasm_bindgen]` impl blocks: the name JS calls each by,
        /// and its parameter types after the receiver.
        pub fn wasm_methods(&self) -> Vec<(String, Vec<Type>)> {
            self.binding_impls(Build::Wasm, "wasm_bindgen")
                .flat_map(|imp| imp.fns.iter())
                .filter(|f| f.cfg.holds(Build::Wasm) && f.sig.receiver().is_some())
                .map(|f| (exported_name(f, Build::Wasm), typed_params(&f.sig)))
                .collect()
        }

        /// The methods of the `#[pymethods]` blocks, with the parameters Python passes.
        pub fn python_methods(&self) -> Vec<PyMethod> {
            self.binding_impls(Build::Python, "pymethods")
                .flat_map(|imp| {
                    imp.fns
                        .iter()
                        .filter(|f| f.cfg.holds(Build::Python) && f.sig.receiver().is_some())
                        .map(|f| PyMethod {
                            name: exported_name(f, Build::Python),
                            params: python_params(f),
                        })
                })
                .collect()
        }
    }

    /// A method a `#[pymethods]` block exposes.
    pub struct PyMethod {
        /// The name Python calls it by.
        pub name: String,
        /// Each parameter's name, and whether `#[pyo3(signature)]` gives it a default.
        pub params: Vec<(String, bool)>,
    }

    /// The parameters Python passes to `f`: those of its `#[pyo3(signature)]`,
    /// else every parameter but the receiver and a `Python` token.
    fn python_params(f: &FnDef) -> Vec<(String, bool)> {
        let signature = metas(&f.attrs, Build::Python)
            .iter()
            .filter(|m| meta_ident(m) == "pyo3")
            .find_map(|m| arg_value(&meta_args(m), "signature"));
        if let Some(signature) = signature {
            return split_top_level(&signature)
                .into_iter()
                .filter(|part| !part.starts_with('*') && part != "/")
                .map(|part| match part.split_once('=') {
                    Some((name, _)) => (name.trim().to_string(), true),
                    None => (part, false),
                })
                .collect();
        }
        f.sig
            .inputs
            .iter()
            .filter_map(|arg| match arg {
                syn::FnArg::Typed(t) if last_ident(&t.ty).as_deref() != Some("Python") => {
                    Some((pattern_name(&t.pat), false))
                }
                _ => None,
            })
            .collect()
    }

    /// The name a binding exports a function under: `js_name` on the web,
    /// `name` in `#[pyo3]` or `#[uniffi::method]`, else the Rust name.
    fn exported_name(f: &FnDef, build: Build) -> String {
        let key = match build {
            Build::Wasm => "js_name",
            Build::Python | Build::Ios | Build::Android => "name",
        };
        metas(&f.attrs, build)
            .iter()
            .filter(|m| {
                matches!(
                    meta_ident(m).as_str(),
                    "wasm_bindgen" | "pyo3" | "method" | "constructor"
                )
            })
            .find_map(|m| arg_value(&meta_args(m), key))
            .unwrap_or_else(|| f.sig.ident.to_string())
    }

    /// Whether `#[doc(hidden)]` marks the binding method as internal in `build`.
    fn is_hidden(f: &FnDef, build: Build) -> bool {
        metas(&f.attrs, build)
            .iter()
            .any(|m| meta_ident(m) == "doc" && meta_args(m).contains("hidden"))
    }

    /// A name in lower case without underscores, so `get_image` matches `getImage`.
    fn fold_name(name: &str) -> String {
        name.replace('_', "").to_lowercase()
    }

    /// Replace `needle` (such as `Vertex.UV0`) where it stands as a whole
    /// qualified name: not preceded by an identifier or `.`, not followed by
    /// an identifier character.
    pub fn replace_qualified(text: &str, needle: &str, replacement: &str) -> String {
        let is_ident = |c: char| c.is_alphanumeric() || c == '_';
        let mut out = String::with_capacity(text.len());
        let mut rest = text;
        while let Some(at) = rest.find(needle) {
            let before = rest[..at].chars().next_back();
            let after = rest[at + needle.len()..].chars().next();
            let bounded =
                !before.is_some_and(|c| is_ident(c) || c == '.') && !after.is_some_and(is_ident);
            out.push_str(&rest[..at]);
            out.push_str(if bounded { replacement } else { needle });
            rest = &rest[at + needle.len()..];
        }
        out.push_str(rest);
        out
    }

    /// Inline each `Owner.NAME` in `text` as its constant's literal, written
    /// by `quote`. Longer names go first so `UV10` is never read as `UV1`.
    pub fn inline_str_consts(text: &str, consts: &[StrConst], quote: fn(&str) -> String) -> String {
        let mut ordered: Vec<&StrConst> = consts.iter().collect();
        ordered.sort_by_key(|c| std::cmp::Reverse(c.owner.len() + c.name.len()));
        let mut out = text.to_string();
        for c in ordered {
            let needle = format!("{}.{}", c.owner, c.name);
            out = replace_qualified(&out, &needle, &quote(&c.value));
        }
        out
    }

    fn field_def(field: &syn::Field) -> FieldDef {
        FieldDef {
            ident: field.ident.as_ref().map(|i| i.to_string()),
            ty: field.ty.clone(),
            attrs: field.attrs.clone(),
        }
    }

    // ------------------------------------------------------------------
    // Mobile bindings
    // ------------------------------------------------------------------

    /// How Swift and Kotlin call a function, exported or hand-written.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum CallKind {
        /// The constructor named `new`, called as `T(...)`.
        Init,
        /// Any other constructor, called as `T.name(...)`.
        Static,
        /// A method called on an instance.
        Method,
    }

    /// A function an `#[uniffi::export]` impl block exposes.
    pub struct UniffiFn {
        /// The exported type, by the name Swift and Kotlin know it.
        pub owner: String,
        /// The camelCase name callers use.
        pub name: String,
        pub kind: CallKind,
        pub is_async: bool,
        /// Whether it returns a `Result`, which Swift marks `throws`.
        pub throws: bool,
        /// Parameter types, receiver excluded.
        pub params: Vec<Type>,
        /// How many trailing parameters a `default(...)` argument covers.
        pub defaults: usize,
    }

    /// A uniffi enum whose variants carry no data.
    /// Kotlin renders it as an `enum class`, and every other uniffi enum as a sealed class.
    pub struct FlatEnum {
        pub ident: String,
        /// Variant names as Rust spells them.
        pub variants: Vec<String>,
    }

    /// A slice getter of a type uniffi exports, such as `Mipmap::levels()`,
    /// that the binding leaves out, paired with the method that reads one element instead.
    pub struct SliceGetter {
        pub owner: String,
        /// The getter, camelCased the way examples call it.
        pub getter: String,
        /// The one exported method of `owner`, such as `level`,
        /// that takes an unsigned index and returns the slice's element type.
        pub accessor: Option<String>,
    }

    /// A struct uniffi exports as a record.
    pub struct UniffiRecord {
        pub ident: String,
        /// Field names and types, in declaration order.
        pub fields: Vec<(String, Type)>,
        /// Whether the crate's `T::new` takes the fields in that order,
        /// so an array literal fills them by position.
        pub positional: bool,
    }

    /// A crate type uniffi doesn't export, and the mobile-only type that converts into it.
    /// Swift and Kotlin code names the second where Rust names the first.
    pub struct StandIn {
        /// The crate type, such as `Color`.
        pub ty: String,
        /// The name Swift and Kotlin give the mobile-only type.
        pub stand_in: String,
    }

    impl Reflection {
        /// Each crate type uniffi doesn't export in `build`,
        /// paired with the one exported type no Python or wasm build declares that converts into it,
        /// the way `impl From<TextureRegionMobile> for TextureRegion` pairs those two.
        /// A pair whose mobile type exports under the crate type's own name needs no rename.
        pub fn stand_ins(&self, build: Build) -> Vec<StandIn> {
            let declared: BTreeSet<&str> = self.types_in(build).map(|t| t.ident.as_str()).collect();
            let exported: BTreeSet<&str> = self
                .types_in(build)
                .filter(|t| exported_type_name(&t.attrs, &t.ident, build).is_some())
                .map(|t| t.ident.as_str())
                .collect();
            let mobile_only = |ident: &str| {
                exported.contains(ident)
                    && [Build::Python, Build::Wasm]
                        .iter()
                        .all(|b| self.type_named(ident, *b).is_none())
            };
            let mut sources: BTreeMap<&str, BTreeSet<String>> = BTreeMap::new();
            for imp in self.trait_impls("From", build) {
                let target = imp.self_ident.as_str();
                let source = imp.trait_arg().and_then(|t| last_ident(strip_type(t)));
                if let Some(source) = source.filter(|s| mobile_only(s))
                    && declared.contains(target)
                    && !exported.contains(target)
                {
                    sources.entry(target).or_default().insert(source);
                }
            }
            sources
                .into_iter()
                .filter_map(|(ty, sources)| match Vec::from_iter(sources).as_slice() {
                    [only] => Some(StandIn {
                        ty: ty.to_string(),
                        stand_in: self.uniffi_type_name(only, build),
                    }),
                    _ => None,
                })
                .filter(|s| s.ty != s.stand_in)
                .collect()
        }

        /// The name Swift and Kotlin give the crate type `ident`.
        fn uniffi_type_name(&self, ident: &str, build: Build) -> String {
            self.type_named(ident, build)
                .and_then(|t| exported_type_name(&t.attrs, &t.ident, build))
                .unwrap_or_else(|| ident.to_string())
        }

        /// Functions the uniffi bindings export in `build`.
        pub fn uniffi_fns(&self, build: Build) -> Vec<UniffiFn> {
            self.uniffi_defs(build)
                .map(|(owner, f)| {
                    let ctor = metas(&f.attrs, build)
                        .iter()
                        .any(|m| meta_ident(m) == "constructor");
                    let name = uniffi_name(f, build);
                    let kind = match (ctor, name.as_str()) {
                        (true, "new") => CallKind::Init,
                        (true, _) => CallKind::Static,
                        (false, _) => CallKind::Method,
                    };
                    UniffiFn {
                        owner: self.uniffi_type_name(owner, build),
                        name,
                        kind,
                        is_async: f.sig.asyncness.is_some(),
                        throws: returns_result(&f.sig),
                        params: typed_params(&f.sig),
                        defaults: uniffi_defaults(f, build),
                    }
                })
                .collect()
        }

        /// Structs that derive `uniffi::Record` in `build`.
        pub fn uniffi_records(&self, build: Build) -> Vec<UniffiRecord> {
            self.types_in(build)
                .filter(|t| t.kind == TypeKind::Struct && derives_uniffi(&t.attrs, build, "Record"))
                .map(|t| {
                    let fields: Vec<(String, Type)> = t
                        .fields
                        .iter()
                        .filter_map(|f| Some((f.ident.clone()?, f.ty.clone())))
                        .collect();
                    let names: Vec<&str> = fields.iter().map(|(name, _)| name.as_str()).collect();
                    let positional = self.plain_fns(&t.ident, build).any(|f| {
                        f.sig.ident == "new" && f.sig.receiver().is_none() && param_names(&f.sig) == names
                    });
                    UniffiRecord {
                        ident: t.ident.clone(),
                        fields,
                        positional,
                    }
                })
                .collect()
        }

        /// Enums that derive `uniffi::Enum` in `build`
        /// and have only unit variants once the variants' own `cfg`s apply.
        pub fn flat_enums(&self, build: Build) -> Vec<FlatEnum> {
            self.types_in(build)
                .filter(|t| t.kind == TypeKind::Enum && derives_uniffi(&t.attrs, build, "Enum"))
                .filter_map(|t| {
                    let variants: Vec<&VariantDef> =
                        t.variants.iter().filter(|v| v.cfg.holds(build)).collect();
                    variants.iter().all(|v| v.is_unit).then(|| FlatEnum {
                        ident: t.ident.clone(),
                        variants: variants.iter().map(|v| v.ident.clone()).collect(),
                    })
                })
                .collect()
        }

        /// The `pub` slice getters of the types uniffi exports in `build`
        /// that no uniffi export of the same type shares a name with.
        pub fn slice_getters(&self, build: Build) -> Vec<SliceGetter> {
            let mut out = Vec::new();
            for ty in self
                .types_in(build)
                .filter(|t| exported_type_name(&t.attrs, &t.ident, build).is_some())
            {
                let owner = ty.ident.as_str();
                let exported: Vec<&FnDef> = self
                    .uniffi_defs(build)
                    .filter(|(o, _)| *o == owner)
                    .map(|(_, f)| f)
                    .collect();
                let names: BTreeSet<String> =
                    exported.iter().map(|f| uniffi_name(f, build)).collect();
                for getter in self.plain_fns(owner, build).filter(|f| f.is_pub) {
                    let name = getter.sig.ident.to_string().to_lower_camel_case();
                    let Some(element) = slice_element(&getter.sig) else {
                        continue;
                    };
                    let reads_self =
                        getter.sig.receiver().is_some() && typed_params(&getter.sig).is_empty();
                    if !reads_self || names.contains(&name) {
                        continue;
                    }
                    let accessors: Vec<&FnDef> = exported
                        .iter()
                        .copied()
                        .filter(|f| reads_element(f, element))
                        .collect();
                    out.push(SliceGetter {
                        owner: owner.to_string(),
                        getter: name,
                        accessor: match accessors.as_slice() {
                            [only] => Some(uniffi_name(only, build)),
                            _ => None,
                        },
                    });
                }
            }
            out
        }

        /// Every function of a uniffi export block in `build`, with its type.
        fn uniffi_defs(&self, build: Build) -> impl Iterator<Item = (&str, &FnDef)> {
            self.binding_impls(build, build.binding_attr())
                .flat_map(move |imp| {
                    imp.fns
                        .iter()
                        .filter(move |f| f.cfg.holds(build))
                        .map(move |f| (imp.self_ident.as_str(), f))
                })
        }

        /// Functions of the impl blocks of `owner` in `build`
        /// that are neither trait impls nor a binding's export blocks.
        fn plain_fns<'a>(
            &'a self,
            owner: &'a str,
            build: Build,
        ) -> impl Iterator<Item = &'a FnDef> {
            let binding = build.binding_attr();
            self.impls
                .iter()
                .filter(move |imp| {
                    imp.self_ident == owner
                        && !imp.is_trait_impl
                        && imp.cfg.holds(build)
                        && !metas(&imp.attrs, build)
                            .iter()
                            .any(|m| meta_ident(m) == binding)
                })
                .flat_map(move |imp| imp.fns.iter().filter(move |f| f.cfg.holds(build)))
        }
    }

    /// How many trailing parameters of `f` the `default(...)` argument of its uniffi attribute names.
    fn uniffi_defaults(f: &FnDef, build: Build) -> usize {
        let named: BTreeSet<String> = metas(&f.attrs, build)
            .iter()
            .filter(|m| matches!(meta_ident(m).as_str(), "method" | "constructor"))
            .flat_map(|m| split_top_level(&meta_args(m)))
            .filter_map(|part| {
                let inner = part.strip_prefix("default")?.trim();
                Some(inner.strip_prefix('(')?.strip_suffix(')')?.to_string())
            })
            .flat_map(|inner| split_top_level(&inner))
            .filter_map(|pair| Some(pair.split_once('=')?.0.trim().to_string()))
            .collect();
        param_names(&f.sig)
            .iter()
            .rev()
            .take_while(|name| named.contains(*name))
            .count()
    }

    /// The name uniffi gives a function: its `name` argument or Rust name, in lowerCamelCase.
    fn uniffi_name(f: &FnDef, build: Build) -> String {
        exported_name(f, build).to_lower_camel_case()
    }

    /// Whether `attrs` derive `uniffi::{name}` in `build`.
    fn derives_uniffi(attrs: &[Attribute], build: Build, name: &str) -> bool {
        metas(attrs, build).iter().any(|m| {
            let Meta::List(list) = m else {
                return false;
            };
            list.path.is_ident("derive")
                && list
                    .parse_args_with(Punctuated::<syn::Path, Token![,]>::parse_terminated)
                    .is_ok_and(|paths| {
                        paths.iter().any(|p| {
                            let segments: Vec<String> =
                                p.segments.iter().map(|s| s.ident.to_string()).collect();
                            segments == ["uniffi", name]
                        })
                    })
        })
    }

    /// Parameter types after the receiver.
    fn typed_params(sig: &Signature) -> Vec<Type> {
        sig.inputs
            .iter()
            .filter_map(|arg| match arg {
                syn::FnArg::Typed(t) => Some((*t.ty).clone()),
                syn::FnArg::Receiver(_) => None,
            })
            .collect()
    }

    /// Parameter names after the receiver.
    fn param_names(sig: &Signature) -> Vec<String> {
        sig.inputs
            .iter()
            .filter_map(|arg| match arg {
                syn::FnArg::Typed(t) => Some(pattern_name(&t.pat)),
                syn::FnArg::Receiver(_) => None,
            })
            .collect()
    }

    /// The name a parameter pattern binds. A pattern other than a name counts as `_`.
    fn pattern_name(pat: &syn::Pat) -> String {
        match pat {
            syn::Pat::Ident(p) => p.ident.to_string(),
            _ => "_".to_string(),
        }
    }

    fn return_type(sig: &Signature) -> Option<&Type> {
        match &sig.output {
            syn::ReturnType::Type(_, ty) => Some(ty),
            syn::ReturnType::Default => None,
        }
    }

    fn last_ident(ty: &Type) -> Option<String> {
        match ty {
            Type::Path(p) => p.path.segments.last().map(|s| s.ident.to_string()),
            _ => None,
        }
    }

    fn returns_result(sig: &Signature) -> bool {
        return_type(sig).and_then(last_ident).as_deref() == Some("Result")
    }

    /// `T` of a `&[T]` or `Vec<T>` return type.
    fn slice_element(sig: &Signature) -> Option<&Type> {
        match return_type(sig)? {
            Type::Reference(r) => match &*r.elem {
                Type::Slice(slice) => Some(&slice.elem),
                _ => None,
            },
            ty => generic_arg(ty, "Vec"),
        }
    }

    /// Whether `f` reads the receiver at one unsigned index
    /// and returns `element`, directly or in a `Result`.
    fn reads_element(f: &FnDef, element: &Type) -> bool {
        let index = match typed_params(&f.sig).as_slice() {
            [ty] => last_ident(ty),
            _ => None,
        };
        let unsigned = matches!(index.as_deref(), Some("u8" | "u16" | "u32" | "u64"));
        let value = return_type(&f.sig).map(|ty| generic_arg(ty, "Result").unwrap_or(ty));
        unsigned && f.sig.receiver().is_some() && value.is_some_and(|v| same_type(v, element))
    }

    /// The first type argument of `ty` when its last path segment is `name`.
    fn generic_arg<'a>(ty: &'a Type, name: &str) -> Option<&'a Type> {
        let Type::Path(p) = ty else {
            return None;
        };
        let segment = p.path.segments.last().filter(|s| s.ident == name)?;
        let syn::PathArguments::AngleBracketed(args) = &segment.arguments else {
            return None;
        };
        args.args.iter().find_map(|arg| match arg {
            syn::GenericArgument::Type(t) => Some(t),
            _ => None,
        })
    }

    fn same_type(a: &Type, b: &Type) -> bool {
        a.to_token_stream().to_string() == b.to_token_stream().to_string()
    }
}
