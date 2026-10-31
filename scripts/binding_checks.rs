mod binding_checks {
    //! Binding checks that fail the build.
    //!
    //! - **Type names.** Every class a binding declares carries a name the
    //!   crate root exports in a plain Rust build: the classes in the generated
    //!   Swift wrapper (which Kotlin shares), in the Python stub, and in the
    //!   wasm exports, along with the classes the TypeScript types name. A
    //!   name only bindings use, such as `MobileWindowTarget`, needs a `type`
    //!   entry with a reason in `docs/api/PARITY`, and every entry there must
    //!   still name a class some binding declares.
    //! - **JS errors.** Every JS error is a `WebError`, an `Error` named after the class
    //!   Python raises. A `#[wasm_bindgen]` function that returns `Result<_, E>` returns
    //!   `WebError`, or an `E` whose `From<E> for JsValue` is exactly
    //!   `WebError::from(error).into()`, and so is every such `From` in the crate. A function
    //!   that returns a `Promise` builds it with `error::defer`, whose future fails with a
    //!   `WebError`, and nothing outside `src/error.rs` calls `future_to_promise`.

    use std::collections::{BTreeMap, BTreeSet};
    use std::fs;
    use std::path::{Path, PathBuf};

    use proc_macro2::{TokenStream, TokenTree};
    use quote::ToTokens;
    use syn::{Attribute, ImplItem, Item, ReturnType, Type, UseTree, Visibility};

    use super::reflect::{self, Build, Reflection};

    const MANIFEST: &str = "docs/api/PARITY";
    const SWIFT_WRAPPER: &str =
        "platforms/swift/Sources/FragmentColor/generated/FragmentColor.swift";
    const PYTHON_STUB: &str = "platforms/python/fragmentcolor/fragmentcolor.pyi";

    /// Run every check and fail the build with the full list of problems.
    pub fn check(root: &Path, model: &Reflection) {
        let mut problems = check_names(root, model);
        problems.extend(check_errors(root));
        if problems.is_empty() {
            println!("✅ Binding type names and JS errors match the Rust API.");
            return;
        }
        eprintln!("\nBinding checks failed:\n");
        for problem in &problems {
            eprintln!("- {problem}");
        }
        panic!(
            "{} binding check(s) failed; see the list above",
            problems.len()
        );
    }

    // -- the plain Rust build -------------------------------------------

    /// What compiles in a plain Rust build: desktop, no binding feature.
    /// Other features stay open, so the `winit` App counts.
    pub mod rust {
        use quote::ToTokens;
        use syn::punctuated::Punctuated;
        use syn::{Attribute, Meta, Token};

        use crate::reflect::Cfg;

        /// Whether `cfg` can hold.
        pub fn holds(cfg: &Cfg) -> bool {
            eval_cfg(cfg) != Some(false)
        }

        /// Whether every `#[cfg]` in `attrs` can hold.
        pub fn allows(attrs: &[Attribute]) -> bool {
            attrs.iter().filter(|a| a.path().is_ident("cfg")).all(|a| {
                match a.parse_args::<Meta>() {
                    Ok(meta) => eval_meta(&meta) != Some(false),
                    Err(_) => true,
                }
            })
        }

        fn eval_cfg(cfg: &Cfg) -> Option<bool> {
            match cfg {
                Cfg::Name(name) => eval_name(name),
                Cfg::Pair(key, value) => eval_pair(key, value),
                Cfg::All(parts) => all_of(parts.iter().map(eval_cfg)),
                Cfg::Any(parts) => any_of(parts.iter().map(eval_cfg)),
                Cfg::Not(inner) => eval_cfg(inner).map(|b| !b),
                Cfg::Unknown => None,
            }
        }

        fn eval_meta(meta: &Meta) -> Option<bool> {
            match meta {
                Meta::Path(path) => eval_name(&path.to_token_stream().to_string()),
                Meta::NameValue(nv) => match &nv.value {
                    syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Str(value),
                        ..
                    }) => eval_pair(&nv.path.to_token_stream().to_string(), &value.value()),
                    _ => None,
                },
                Meta::List(list) => {
                    let parts = list
                        .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
                        .ok()?;
                    let results = parts.iter().map(eval_meta);
                    match list.path.to_token_stream().to_string().as_str() {
                        "all" => all_of(results),
                        "any" => any_of(results),
                        "not" => parts.first().and_then(eval_meta).map(|b| !b),
                        _ => None,
                    }
                }
            }
        }

        fn eval_name(name: &str) -> Option<bool> {
            match name {
                "desktop" => Some(true),
                "wasm" | "python" | "ios" | "android" | "mobile" | "dev" | "test" | "doc"
                | "doctest" | "miri" => Some(false),
                _ => None,
            }
        }

        fn eval_pair(key: &str, value: &str) -> Option<bool> {
            match (key, value) {
                ("feature", "python") => Some(false),
                ("target_arch", "wasm32") | ("target_family", "wasm") => Some(false),
                ("target_os", "ios" | "android" | "unknown") => Some(false),
                _ => None,
            }
        }

        fn all_of(results: impl Iterator<Item = Option<bool>>) -> Option<bool> {
            let results: Vec<Option<bool>> = results.collect();
            if results.contains(&Some(false)) {
                Some(false)
            } else if results.iter().all(|r| *r == Some(true)) {
                Some(true)
            } else {
                None
            }
        }

        fn any_of(results: impl Iterator<Item = Option<bool>>) -> Option<bool> {
            let results: Vec<Option<bool>> = results.collect();
            if results.contains(&Some(true)) {
                Some(true)
            } else if results.iter().all(|r| *r == Some(false)) {
                Some(false)
            } else {
                None
            }
        }
    }

    // -- type names -----------------------------------------------------

    /// Where a binding declares its classes.
    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    enum Source {
        /// The generated uniffi Swift wrapper. Kotlin gets the same names.
        Uniffi,
        Python,
        Wasm,
    }

    impl Source {
        fn label(self) -> &'static str {
            match self {
                Source::Uniffi => "Swift and Kotlin (the uniffi wrapper)",
                Source::Python => "Python (the type stub)",
                Source::Wasm => "JS (the wasm exports)",
            }
        }

        /// The `docs/api/PARITY` platforms an entry must list for this source.
        fn platforms(self) -> &'static [&'static str] {
            match self {
                Source::Uniffi => &["swift", "kotlin"],
                Source::Python => &["python"],
                Source::Wasm => &["web"],
            }
        }
    }

    /// A `type <Name>:<platforms>:<reason>` line of `docs/api/PARITY`.
    struct NameEntry {
        platforms: BTreeSet<String>,
        line: usize,
    }

    fn check_names(root: &Path, model: &Reflection) -> Vec<String> {
        let allowed = read_root(root);
        let entries = read_entries(&root.join(MANIFEST));
        let mut declared: BTreeMap<String, BTreeSet<Source>> = BTreeMap::new();
        let sources = [
            (Source::Uniffi, read_swift(&root.join(SWIFT_WRAPPER))),
            (Source::Python, read_python(&root.join(PYTHON_STUB))),
            (Source::Wasm, read_wasm(model)),
        ];
        for (source, names) in sources {
            for name in names {
                declared.entry(name).or_default().insert(source);
            }
        }

        let mut problems = Vec::new();
        for (name, sources) in &declared {
            if allowed.contains(name) {
                continue;
            }
            for source in sources {
                let listed = entries.get(name).is_some_and(|entry| {
                    source
                        .platforms()
                        .iter()
                        .all(|p| entry.platforms.contains(*p))
                });
                if !listed {
                    problems.push(format!(
                        "{}: `{name}` is not a Rust root export. Name the class after its \
                         Rust type, or list it in {MANIFEST} as `type {name}:{}:<reason>`.",
                        source.label(),
                        source.platforms().join(",")
                    ));
                }
            }
        }
        for (name, entry) in &entries {
            if !declared.contains_key(name) {
                problems.push(format!(
                    "{MANIFEST}:{}: `type {name}` names a class no binding declares; remove the entry.",
                    entry.line
                ));
            } else if allowed.contains(name) {
                problems.push(format!(
                    "{MANIFEST}:{}: `type {name}` is a Rust root export already; remove the entry.",
                    entry.line
                ));
            }
        }
        problems
    }

    fn read_entries(file: &Path) -> BTreeMap<String, NameEntry> {
        let mut entries = BTreeMap::new();
        let Ok(text) = fs::read_to_string(file) else {
            return entries;
        };
        for (index, line) in text.lines().enumerate() {
            let Some(rest) = line.trim().strip_prefix("type ") else {
                continue;
            };
            let mut parts = rest.splitn(3, ':');
            let (Some(name), Some(platforms)) = (parts.next(), parts.next()) else {
                continue;
            };
            entries.insert(
                name.trim().to_string(),
                NameEntry {
                    platforms: platforms
                        .split(',')
                        .map(|p| p.trim().to_string())
                        .filter(|p| !p.is_empty())
                        .collect(),
                    line: index + 1,
                },
            );
        }
        entries
    }

    /// Top-level public types the uniffi wrapper declares, without uniffi's
    /// own plumbing and without the `XProtocol` that comes with each object `X`.
    fn read_swift(file: &Path) -> BTreeSet<String> {
        let Ok(text) = fs::read_to_string(file) else {
            println!(
                "cargo::warning=binding checks: {} is missing, so Swift and Kotlin type names went unchecked",
                file.display()
            );
            return BTreeSet::new();
        };
        let words = read_words(&strip_comments(&text));
        let mut names = BTreeSet::new();
        for (i, word) in words.iter().enumerate() {
            if !matches!(word.as_str(), "public" | "open") {
                continue;
            }
            let mut j = i + 1;
            while words
                .get(j)
                .is_some_and(|w| matches!(w.as_str(), "final" | "indirect"))
            {
                j += 1;
            }
            let declares = words.get(j).is_some_and(|w| {
                matches!(
                    w.as_str(),
                    "class" | "struct" | "enum" | "protocol" | "typealias"
                )
            });
            if declares && let Some(name) = words.get(j + 1) {
                names.insert(name.clone());
            }
        }
        let plumbing = |name: &str| {
            [
                "FfiConverter",
                "Uniffi",
                "uniffi",
                "RustBuffer",
                "ForeignBytes",
                "RustCallStatus",
            ]
            .iter()
            .any(|prefix| name.starts_with(prefix))
        };
        let objects: BTreeSet<String> = names.into_iter().filter(|n| !plumbing(n)).collect();
        objects
            .iter()
            .filter(|name| {
                name.strip_suffix("Protocol")
                    .is_none_or(|object| !objects.contains(object))
            })
            .cloned()
            .collect()
    }

    /// The identifiers of `code` outside any `{ }` block.
    fn read_words(code: &str) -> Vec<String> {
        let mut words = Vec::new();
        let mut word = String::new();
        let mut depth = 0usize;
        for c in code.chars() {
            if c.is_ascii_alphanumeric() || c == '_' {
                word.push(c);
                continue;
            }
            if !word.is_empty() && depth == 0 {
                words.push(std::mem::take(&mut word));
            }
            word.clear();
            match c {
                '{' => depth += 1,
                '}' => depth = depth.saturating_sub(1),
                _ => {}
            }
        }
        if !word.is_empty() && depth == 0 {
            words.push(word);
        }
        words
    }

    /// Swift source without its `/* */` and `//` comments, so doc prose
    /// such as "a public struct" never reads as a declaration.
    fn strip_comments(text: &str) -> String {
        let mut out = String::with_capacity(text.len());
        let mut rest = text;
        while let Some(start) = rest.find("/*") {
            out.push_str(&rest[..start]);
            rest = rest[start..]
                .find("*/")
                .map_or("", |end| &rest[start + end + 2..]);
        }
        out.push_str(rest);
        out.lines()
            .map(|line| line.split("//").next().unwrap_or(""))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Classes the Python stub declares. A class a method only returns is
    /// written `_Name`, and its name is `Name`.
    fn read_python(file: &Path) -> BTreeSet<String> {
        let Ok(text) = fs::read_to_string(file) else {
            return BTreeSet::new();
        };
        text.lines()
            .filter_map(|line| line.strip_prefix("class "))
            .map(|rest| {
                let end = rest
                    .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
                    .unwrap_or(rest.len());
                rest[..end].trim_start_matches('_').to_string()
            })
            .filter(|name| !name.is_empty())
            .collect()
    }

    /// Classes the wasm package exports, and the crate's class names that
    /// a TypeScript parameter or return type spells out.
    fn read_wasm(model: &Reflection) -> BTreeSet<String> {
        let mut names: BTreeSet<String> = model
            .types_in(Build::Wasm)
            .filter_map(|t| reflect::exported_type_name(&t.attrs, &t.ident, Build::Wasm))
            .collect();
        let known: BTreeSet<&str> = model.types.iter().map(|t| t.ident.as_str()).collect();
        let fns = model
            .binding_impls(Build::Wasm, "wasm_bindgen")
            .flat_map(|imp| imp.fns.iter())
            .chain(model.binding_fns(Build::Wasm, "wasm_bindgen"))
            .filter(|f| f.cfg.holds(Build::Wasm));
        let mut spelled = Vec::new();
        for f in fns {
            spelled.extend(read_typescript(&f.attrs, "unchecked_return_type"));
            for input in &f.sig.inputs {
                if let syn::FnArg::Typed(arg) = input {
                    spelled.extend(read_typescript(&arg.attrs, "unchecked_param_type"));
                }
            }
        }
        for ts in spelled {
            for word in ts.split(|c: char| !(c.is_ascii_alphanumeric() || c == '_')) {
                if known.contains(word) || names.contains(word) {
                    names.insert(word.to_string());
                }
            }
        }
        names
    }

    /// The `key = "..."` TypeScript types in the `#[wasm_bindgen]` attributes of `attrs`.
    fn read_typescript(attrs: &[Attribute], key: &str) -> Vec<String> {
        reflect::metas(attrs, Build::Wasm)
            .iter()
            .filter(|m| reflect::meta_ident(m) == "wasm_bindgen")
            .filter_map(|m| reflect::arg_value(&reflect::meta_args(m), key))
            .collect()
    }

    // -- Rust root exports ----------------------------------------------

    /// A module of the crate as a plain Rust build compiles it.
    #[derive(Default)]
    struct Module {
        items: Vec<Item>,
        children: BTreeMap<String, Module>,
    }

    impl Module {
        fn load(file: &Path, child_dir: &Path) -> Module {
            let Ok(text) = fs::read_to_string(file) else {
                return Module::default();
            };
            let Ok(parsed) = syn::parse_file(&text) else {
                return Module::default();
            };
            Module::from_items(parsed.items, child_dir)
        }

        fn from_items(items: Vec<Item>, child_dir: &Path) -> Module {
            let mut module = Module::default();
            for item in &items {
                let Item::Mod(m) = item else {
                    continue;
                };
                if !rust::allows(&m.attrs) {
                    continue;
                }
                let name = m.ident.to_string();
                let dir = child_dir.join(&name);
                let child = match &m.content {
                    Some((_, inner)) => Module::from_items(inner.clone(), &dir),
                    None => {
                        let flat = child_dir.join(format!("{name}.rs"));
                        let file = if flat.exists() {
                            flat
                        } else {
                            dir.join("mod.rs")
                        };
                        Module::load(&file, &dir)
                    }
                };
                module.children.insert(name, child);
            }
            module.items = items;
            module
        }

        fn child(&self, path: &[String]) -> Option<&Module> {
            path.iter().try_fold(self, |m, name| m.children.get(name))
        }
    }

    /// Type and trait names `fragmentcolor::Name` reaches in a plain Rust build.
    fn read_root(root: &Path) -> BTreeSet<String> {
        let src = root.join("src");
        let crate_root = Module::load(&src.join("lib.rs"), &src);
        let mut names = BTreeSet::new();
        collect_exports(&crate_root, &[], &mut BTreeSet::new(), &mut names);
        names
    }

    /// Collect the public names of the module at `path` into `out`.
    fn collect_exports(
        crate_root: &Module,
        path: &[String],
        visiting: &mut BTreeSet<Vec<String>>,
        out: &mut BTreeSet<String>,
    ) {
        let Some(module) = crate_root.child(path) else {
            return;
        };
        if !visiting.insert(path.to_vec()) {
            return;
        }
        for item in &module.items {
            let (vis, attrs, name) = match item {
                Item::Struct(s) => (&s.vis, &s.attrs, s.ident.to_string()),
                Item::Enum(e) => (&e.vis, &e.attrs, e.ident.to_string()),
                Item::Trait(t) => (&t.vis, &t.attrs, t.ident.to_string()),
                Item::Type(t) => (&t.vis, &t.attrs, t.ident.to_string()),
                Item::Use(u) => {
                    if matches!(u.vis, Visibility::Public(_)) && rust::allows(&u.attrs) {
                        let mut leaves = Vec::new();
                        flatten_use(&u.tree, Vec::new(), &mut leaves);
                        for (prefix, leaf) in leaves {
                            let base = resolve(path, &prefix);
                            match leaf {
                                Leaf::Glob => collect_exports(crate_root, &base, visiting, out),
                                Leaf::Name(name) => {
                                    out.insert(name);
                                }
                            }
                        }
                    }
                    continue;
                }
                _ => continue,
            };
            if matches!(vis, Visibility::Public(_)) && rust::allows(attrs) {
                out.insert(name);
            }
        }
    }

    enum Leaf {
        Glob,
        Name(String),
    }

    /// Flatten a `use` tree into `(module path, leaf)` pairs.
    fn flatten_use(tree: &UseTree, prefix: Vec<String>, out: &mut Vec<(Vec<String>, Leaf)>) {
        match tree {
            UseTree::Path(p) => {
                let mut next = prefix;
                next.push(p.ident.to_string());
                flatten_use(&p.tree, next, out);
            }
            UseTree::Name(n) => out.push((prefix, Leaf::Name(n.ident.to_string()))),
            UseTree::Rename(r) => out.push((prefix, Leaf::Name(r.rename.to_string()))),
            UseTree::Glob(_) => out.push((prefix, Leaf::Glob)),
            UseTree::Group(g) => {
                for inner in &g.items {
                    flatten_use(inner, prefix.clone(), out);
                }
            }
        }
    }

    /// The module path a `use` prefix written in the module at `from` names.
    fn resolve(from: &[String], prefix: &[String]) -> Vec<String> {
        let mut path: Vec<String> = from.to_vec();
        for (i, segment) in prefix.iter().enumerate() {
            match segment.as_str() {
                "crate" if i == 0 => path.clear(),
                "self" => {}
                "super" => {
                    path.pop();
                }
                _ => path.push(segment.clone()),
            }
        }
        path
    }

    // -- JS errors ------------------------------------------------------

    /// A `#[wasm_bindgen]` function that returns a `Result`.
    struct Fallible {
        file: PathBuf,
        name: String,
        error: Type,
    }

    /// A `#[wasm_bindgen]` function that returns a `Promise`, which rejects with what its future returns.
    struct Deferred {
        file: PathBuf,
        name: String,
        body: TokenStream,
    }

    /// What the scan of `src` finds: the fallible and deferred exports, each
    /// `From<E> for JsValue` by the name of `E` with whether it goes through `WebError`,
    /// and the files that build a promise without `error::defer`.
    #[derive(Default)]
    struct Exports {
        fallible: Vec<Fallible>,
        deferred: Vec<Deferred>,
        conversions: BTreeMap<String, (PathBuf, bool)>,
        builders: Vec<PathBuf>,
    }

    fn check_errors(root: &Path) -> Vec<String> {
        let mut exports = Exports::default();
        for file in list_files(&root.join("src")) {
            let Ok(text) = fs::read_to_string(&file) else {
                continue;
            };
            let Ok(parsed) = syn::parse_file(&text) else {
                continue;
            };
            let relative = file.strip_prefix(root).unwrap_or(&file);
            scan_items(&parsed.items, relative, &mut exports);
            let home = relative == Path::new("src/error.rs");
            if !home && names(&parsed.to_token_stream(), "future_to_promise") {
                exports.builders.push(relative.to_path_buf());
            }
        }

        let mut problems = Vec::new();
        for f in &exports.fallible {
            let error = read_ident(&f.error).unwrap_or_default();
            let advice = match error.as_str() {
                "WebError" => continue,
                "JsError" | "JsValue" => "Return `WebError`.".to_string(),
                _ if exports.conversions.contains_key(&error) => continue,
                _ => format!(
                    "Return `WebError`, or convert through `WebError` in `From<{error}> for JsValue`."
                ),
            };
            problems.push(format!(
                "{}: `{}` returns `Result<_, {error}>`, and JS would catch an error that no class names. {advice}",
                f.file.display(),
                f.name
            ));
        }
        for (source, (file, named)) in &exports.conversions {
            if source != "WebError" && !named {
                problems.push(format!(
                    "{}: write `From<{source}> for JsValue` as `WebError::from(error).into()`, \
                     and put per-variant logic in `From<{source}> for WebError`.",
                    file.display()
                ));
            }
        }
        for d in &exports.deferred {
            if !names(&d.body, "defer") {
                problems.push(format!(
                    "{}: `{}` returns a `Promise` that `error::defer` didn't build. \
                     Build it with `defer`, whose future fails with a `WebError`.",
                    d.file.display(),
                    d.name
                ));
            }
        }
        for file in &exports.builders {
            problems.push(format!(
                "{}: calls `future_to_promise`, whose promise can reject with any `JsValue`. Use `error::defer`.",
                file.display()
            ));
        }
        problems
    }

    fn list_files(dir: &Path) -> Vec<PathBuf> {
        let mut out = Vec::new();
        let Ok(read) = fs::read_dir(dir) else {
            return out;
        };
        for entry in read.flatten() {
            let path = entry.path();
            if path.is_dir() {
                out.extend(list_files(&path));
            } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
                out.push(path);
            }
        }
        out.sort();
        out
    }

    fn scan_items(items: &[Item], file: &Path, exports: &mut Exports) {
        for item in items {
            match item {
                Item::Mod(m) => {
                    if let Some((_, inner)) = &m.content {
                        scan_items(inner, file, exports);
                    }
                }
                Item::Fn(f) if is_exported(&f.attrs) => {
                    record_export(file, &f.sig, f.block.to_token_stream(), exports);
                }
                Item::Impl(imp) => {
                    let into_js = imp.trait_.as_ref().is_some_and(|(_, path, _)| {
                        path.segments.last().is_some_and(|s| s.ident == "From")
                    }) && read_ident(&imp.self_ty).as_deref() == Some("JsValue");
                    if into_js {
                        if let Some(source) = read_source(imp).and_then(read_ident) {
                            let conversion = (file.to_path_buf(), converts_through(imp));
                            exports.conversions.insert(source, conversion);
                        }
                        continue;
                    }
                    if imp.trait_.is_some() || !is_exported(&imp.attrs) {
                        continue;
                    }
                    for member in &imp.items {
                        let ImplItem::Fn(f) = member else {
                            continue;
                        };
                        record_export(file, &f.sig, f.block.to_token_stream(), exports);
                    }
                }
                _ => {}
            }
        }
    }

    /// Whether `attrs` carry `#[wasm_bindgen]` in the wasm build.
    fn is_exported(attrs: &[Attribute]) -> bool {
        reflect::metas(attrs, Build::Wasm)
            .iter()
            .any(|m| reflect::meta_ident(m) == "wasm_bindgen")
    }

    /// `E` of a `Result<T, E>` return type.
    fn read_error(output: &ReturnType) -> Option<&Type> {
        let ReturnType::Type(_, ty) = output else {
            return None;
        };
        let Type::Path(path) = &**ty else {
            return None;
        };
        let segment = path.path.segments.last().filter(|s| s.ident == "Result")?;
        let syn::PathArguments::AngleBracketed(args) = &segment.arguments else {
            return None;
        };
        args.args
            .iter()
            .filter_map(|arg| match arg {
                syn::GenericArgument::Type(t) => Some(t),
                _ => None,
            })
            .nth(1)
    }

    fn read_source(imp: &syn::ItemImpl) -> Option<&Type> {
        let (_, path, _) = imp.trait_.as_ref()?;
        let syn::PathArguments::AngleBracketed(args) = &path.segments.last()?.arguments else {
            return None;
        };
        args.args.iter().find_map(|arg| match arg {
            syn::GenericArgument::Type(t) => Some(t),
            _ => None,
        })
    }

    fn read_ident(ty: &Type) -> Option<String> {
        match reflect::strip_type(ty) {
            Type::Path(p) => p.path.segments.last().map(|s| s.ident.to_string()),
            _ => None,
        }
    }

    /// Records an export that returns a `Result` or a `Promise`.
    fn record_export(file: &Path, sig: &syn::Signature, body: TokenStream, exports: &mut Exports) {
        let name = sig.ident.to_string();
        if let Some(error) = read_error(&sig.output) {
            let error = error.clone();
            let file = file.to_path_buf();
            exports.fallible.push(Fallible { file, name, error });
        } else if returns_promise(&sig.output) {
            let file = file.to_path_buf();
            exports.deferred.push(Deferred { file, name, body });
        }
    }

    /// Whether `output` is a `Promise`.
    fn returns_promise(output: &ReturnType) -> bool {
        let ReturnType::Type(_, ty) = output else {
            return false;
        };
        read_ident(ty).as_deref() == Some("Promise")
    }

    /// Whether a `From<E> for JsValue` impl's `from` is exactly `WebError::from(error).into()`,
    /// with `error` its own parameter. A `From<E> for WebError` outside `src/error.rs` can only
    /// build its value through a constructor that names a class, so this form always names one.
    fn converts_through(imp: &syn::ItemImpl) -> bool {
        let Some(from) = imp.items.iter().find_map(|item| match item {
            ImplItem::Fn(f) if f.sig.ident == "from" => Some(f),
            _ => None,
        }) else {
            return false;
        };
        let Some(syn::FnArg::Typed(argument)) = from.sig.inputs.first() else {
            return false;
        };
        let syn::Pat::Ident(param) = &*argument.pat else {
            return false;
        };
        let [syn::Stmt::Expr(syn::Expr::MethodCall(call), None)] = &from.block.stmts[..] else {
            return false;
        };
        if call.method != "into" || !call.args.is_empty() {
            return false;
        }
        let syn::Expr::Call(inner) = &*call.receiver else {
            return false;
        };
        let syn::Expr::Path(function) = &*inner.func else {
            return false;
        };
        let segments: Vec<String> = function
            .path
            .segments
            .iter()
            .map(|segment| segment.ident.to_string())
            .collect();
        let named = segments.ends_with(&["WebError".to_string(), "from".to_string()]);
        let passes = matches!(
            inner.args.iter().collect::<Vec<_>>()[..],
            [syn::Expr::Path(value)] if value.path.is_ident(&param.ident)
        );
        named && passes
    }

    /// Whether `tokens` name the identifier `name` anywhere, string literals and comments aside.
    fn names(tokens: &TokenStream, name: &str) -> bool {
        tokens.clone().into_iter().any(|token| match token {
            TokenTree::Ident(ident) => ident == name,
            TokenTree::Group(group) => names(&group.stream(), name),
            _ => false,
        })
    }
}
