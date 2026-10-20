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

    use std::fs;
    use std::path::{Path, PathBuf};

    use quote::ToTokens;
    use syn::punctuated::Punctuated;
    use syn::{Attribute, Fields, ImplItem, Item, Meta, Signature, Token, Type};

    /// A build the generators emit code for.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Build {
        /// The desktop wheel built with the `python` feature.
        Python,
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
                    match &nv.value {
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
        match (name, build) {
            ("python" | "desktop", Build::Python) => Some(true),
            ("wasm" | "mobile" | "ios" | "android" | "dev", Build::Python) => Some(false),
            ("test" | "doc" | "doctest" | "miri", _) => Some(false),
            _ => None,
        }
    }

    fn pair_holds(key: &str, value: &str, build: Build) -> Option<bool> {
        match (key, build) {
            // maturin builds the wheel with the default features plus `python`.
            ("feature", Build::Python) => match value {
                "python" | "default" => Some(true),
                v if v.starts_with("shaders") => Some(true),
                "winit" | "network" | "uniffi/cli" => Some(false),
                _ => None,
            },
            ("target_arch", Build::Python) if value == "wasm32" => Some(false),
            ("target_family", Build::Python) if value == "wasm" => Some(false),
            ("target_os", Build::Python) if matches!(value, "ios" | "android" | "unknown") => {
                Some(false)
            }
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
    }

    /// An `impl` block, inherent or for a trait.
    pub struct ImplDef {
        /// The file that declares the block.
        pub file: PathBuf,
        pub self_ident: String,
        pub is_trait_impl: bool,
        pub attrs: Vec<Attribute>,
        pub cfg: Cfg,
        pub fns: Vec<FnDef>,
    }

    /// A function in an `impl` block or at module level.
    pub struct FnDef {
        pub attrs: Vec<Attribute>,
        pub sig: Signature,
        pub cfg: Cfg,
    }

    #[derive(Default)]
    pub struct Reflection {
        pub types: Vec<TypeDef>,
        pub impls: Vec<ImplDef>,
        pub free_fns: Vec<FnDef>,
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
            model
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
                    && metas(&imp.attrs, build).iter().any(|m| meta_ident(m) == binding)
            })
        }

        /// Module-level functions that compile in `build` and carry `binding`.
        pub fn binding_fns(
            &self,
            build: Build,
            binding: &'static str,
        ) -> impl Iterator<Item = &FnDef> {
            self.free_fns.iter().filter(move |f| {
                f.cfg.holds(build) && metas(&f.attrs, build).iter().any(|m| meta_ident(m) == binding)
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
                    let Type::Path(self_ty) = imp.self_ty.as_ref() else {
                        continue;
                    };
                    let self_ident = self_ty
                        .path
                        .segments
                        .last()
                        .map(|s| s.ident.to_string())
                        .unwrap_or_default();
                    let impl_cfg = cfg.and(&imp.attrs);
                    let fns = imp
                        .items
                        .iter()
                        .filter_map(|it| match it {
                            ImplItem::Fn(f) => Some(FnDef {
                                attrs: f.attrs.clone(),
                                sig: f.sig.clone(),
                                cfg: impl_cfg.and(&f.attrs),
                            }),
                            _ => None,
                        })
                        .collect();
                    model.impls.push(ImplDef {
                        file: file.to_path_buf(),
                        self_ident,
                        is_trait_impl: imp.trait_.is_some(),
                        attrs: imp.attrs.clone(),
                        cfg: impl_cfg,
                        fns,
                    });
                }
                Item::Fn(f) => model.free_fns.push(FnDef {
                    attrs: f.attrs.clone(),
                    sig: f.sig.clone(),
                    cfg: cfg.and(&f.attrs),
                }),
                _ => {}
            }
        }
    }

    fn field_def(field: &syn::Field) -> FieldDef {
        FieldDef {
            ident: field.ident.as_ref().map(|i| i.to_string()),
            ty: field.ty.clone(),
            attrs: field.attrs.clone(),
        }
    }

}
