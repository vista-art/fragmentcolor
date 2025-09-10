use cfg_aliases::cfg_aliases;

fn main() {
    cfg_aliases! {
        wasm: { target_arch = "wasm32" },
        ios: { target_os = "ios" },
        android: { target_os = "android" },
        mobile: { any(android, ios) },
        desktop: { not(any(wasm, mobile)) },
        python: { all(desktop, feature="python") },
        dev: { all(desktop, feature="uniffi/cli") },
    }
    println!("cargo::rustc-check-cfg=cfg(wasm)");
    println!("cargo::rustc-check-cfg=cfg(ios)");
    println!("cargo::rustc-check-cfg=cfg(android)");
    println!("cargo::rustc-check-cfg=cfg(mobile)");
    println!("cargo::rustc-check-cfg=cfg(desktop)");
    println!("cargo::rustc-check-cfg=cfg(dev)");

    // Ensure changes under docs/api retrigger build.rs
    println!("cargo:rerun-if-changed=docs/api");

    println!("\n🗺️ Generating API map...");
    let api_map = codegen::scan_api();
    codegen::export_api_map(&api_map);
    println!("✅ API map successfully generated!\n");

    println!("🔎 Validating documentation...");
    validation::validate_docs(&api_map);
    println!("✅ Docs validated!\n");

    println!("🧱 Exporting website (examples + pages)...");
    validation::export_website(&api_map);
    println!("✅ Website export done!\n");
}

mod codegen {
    use quote::ToTokens;
    use std::{
        collections::{HashMap, HashSet, hash_map::Entry},
        convert::AsRef,
        fs,
        hash::Hash,
        io::Write,
        path::{Path, PathBuf},
    };
    use syn::{
        Ident, ImplItem, Item, ItemFn, ItemImpl, ItemStruct, ReturnType, Visibility, parse_file,
    };

    pub const API_MAP_KEYWORD: &str = "API_MAP";
    pub const API_MAP_FILE: &str = "generated/api_map.rs";
    pub const OBJECT_PROPERTY_STRUCT_DEFINITION: &str = "
#[derive(Clone, Debug, PartialEq)]
struct FunctionParameter {
    pub name: &'static str,
    pub type_name: &'static str,
}

#[derive(Clone, Debug, PartialEq)]
struct FunctionSignature {
    pub name: &'static str,
    pub parameters: &'static [FunctionParameter],
    pub return_type: Option<&'static str>,
}

#[derive(Clone, Debug, PartialEq)]
struct ObjectProperty {
    pub name: &'static str,
    pub type_name: &'static str,
    pub function: Option<FunctionSignature>,
}
";

    #[derive(Clone, Debug, PartialEq)]
    pub struct FunctionParameter {
        pub name: String,
        pub type_name: String,
    }
    #[derive(Clone, Debug, PartialEq)]
    pub struct FunctionSignature {
        pub name: String,
        pub parameters: Vec<FunctionParameter>,
        pub return_type: Option<String>,
    }

    #[derive(Clone, Debug)]
    pub struct ObjectProperty {
        pub name: String,
        pub type_name: String,
        pub function: Option<FunctionSignature>,
    }

    impl Eq for ObjectProperty {}
    impl PartialEq for ObjectProperty {
        fn eq(&self, other: &Self) -> bool {
            self.name == other.name && self.type_name == other.type_name
        }
    }

    impl Hash for ObjectProperty {
        fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
            self.name.hash(state);
        }
    }

    pub type ApiMap = HashMap<String, Vec<ObjectProperty>>;

    #[derive(Clone, Debug, PartialEq)]
    enum NameFilter {
        Global,
        Specific(String),
        Rename(String, String),
    }

    pub fn scan_api() -> ApiMap {
        let crate_root = super::meta::workspace_root();

        // Extract functions from source
        let mut api_map = extract_public_functions(crate_root.as_ref());

        // Derive canonical public objects from AST: all top-level pub structs excluding #[doc(hidden)]
        let objects = super::validation::public_structs_excluding_hidden();

        // Keep only objects discovered in code (exclude file-key entries and hidden/internal types)
        api_map.retain(|k, _| objects.contains(k));

        api_map
    }

    /// Traverses a Rust library `/src` directory and returns
    /// a HashMap of its public functions and their signatures
    fn extract_public_functions(crate_path: &Path) -> ApiMap {
        let mut signatures = ApiMap::new();
        let (entry_path, parsed_file) = parse_lib_entry_point(crate_path);

        traverse_and_extract(
            entry_path.as_ref(),
            parsed_file.items,
            &mut signatures,
            NameFilter::Global,
        );

        signatures
    }

    /// Builds an AST from the lib's entry point file
    pub fn parse_lib_entry_point(file_path: &Path) -> (PathBuf, syn::File) {
        let entry_point = file_path.join("src/lib.rs");
        let content = fs::read_to_string(&entry_point).expect("Couldn't find src/lib.rs file");
        let parsed_file = parse_file(&content).expect("Failed to parse lib.rs file");

        (entry_point, parsed_file)
    }

    /// Traverses the AST and extracts all public functions
    fn traverse_and_extract(
        current_path: &Path,
        items: Vec<Item>,
        signatures: &mut ApiMap,
        name_filter: NameFilter,
    ) {
        let mut pub_uses = Vec::new();
        let mut private_modules = HashSet::new();
        let mut reexported_modules = HashMap::new();

        // First pass: Collect all private modules
        // and all `pub use` statements
        for item in &items {
            match item {
                Item::Mod(item_mod) => {
                    if let Visibility::Public(_) = item_mod.vis {
                        continue;
                    }
                    let mod_name = item_mod.ident.to_string();
                    private_modules.insert(mod_name);
                }
                Item::Use(item_use) => {
                    if let Visibility::Public(_) = item_use.vis {
                        pub_uses.push(item_use.clone());
                    }
                }
                _ => {}
            }
        }

        // Second pass: Loop through all public use statements
        // and check for reexported items in private modules
        for item_use in pub_uses {
            if let syn::UseTree::Path(use_path) = &item_use.tree {
                let full_path = extract_full_path_from_use_tree(&item_use.tree);
                let last_segment = full_path.last().unwrap();
                let mod_name = last_segment.to_string();

                if private_modules.contains(&mod_name) {
                    let mut mod_structs = extract_names_from_use_tree(&use_path.tree);

                    match reexported_modules.entry(mod_name) {
                        Entry::Vacant(entry) if !mod_structs.is_empty() => {
                            entry.insert(mod_structs);
                        }
                        Entry::Occupied(mut entry) => {
                            entry.get_mut().append(&mut mod_structs);
                        }
                        _ => {}
                    }
                }
            }
        }

        // Third pass: Process the items
        for item in items {
            match item {
                // If the item is a module, we will recurse into it
                Item::Mod(item_mod) => {
                    if let Visibility::Public(_) = item_mod.vis {
                        let (mod_path, mod_items) = parse_module(current_path, &item_mod);
                        traverse_and_extract(&mod_path, mod_items, signatures, NameFilter::Global)
                    } else {
                        let mod_name = item_mod.ident.to_string();
                        let reexported = reexported_modules.get(&mod_name);
                        if reexported.is_none() {
                            continue;
                        }

                        reexported.unwrap().iter().for_each(|name_filter| {
                            let (mod_path, mod_items) = parse_module(current_path, &item_mod);
                            traverse_and_extract(
                                &mod_path,
                                mod_items,
                                signatures,
                                name_filter.clone(),
                            );
                        });
                    }
                }
                // If the item is a struct, we will extract its public fields
                Item::Struct(item_struct) => {
                    extract_struct(item_struct, signatures, name_filter.clone());
                }
                // If the item is an impl block, we will extract its public methods and properties
                Item::Impl(item_impl) => {
                    extract_impl(item_impl, signatures, name_filter.clone());
                }
                // If the item is a function, we will extract its signature
                Item::Fn(item_fn) => {
                    extract_fn(current_path, item_fn, signatures, name_filter.clone());
                }
                _ => {}
            }
        }
    }

    /// Returns only the module path names from a `use` statement,
    /// discarding the imported item names, so we can compare
    /// our module names with the last path segment.
    fn extract_full_path_from_use_tree(tree: &syn::UseTree) -> Vec<Ident> {
        match tree {
            syn::UseTree::Path(use_path) => {
                let mut path: Vec<Ident> = vec![use_path.ident.clone()];
                path.extend(extract_full_path_from_use_tree(&use_path.tree));
                path
            }
            _ => vec![],
        }
    }

    /// Extracts all imported item names from a `use` tree,
    /// discarding the module path names, and flattening
    /// all nested items in a Vec of NameFilter.
    fn extract_names_from_use_tree(tree: &syn::UseTree) -> Vec<NameFilter> {
        match tree {
            // Root
            syn::UseTree::Path(use_path) => extract_names_from_use_tree(&use_path.tree),

            // Branch
            syn::UseTree::Group(use_group) => {
                let mut names = Vec::new();
                for use_tree in &use_group.items {
                    let new_vec = extract_names_from_use_tree(use_tree);
                    names.append(&mut Vec::from(new_vec.as_slice()));
                }
                names
            }

            // Leaf nodes
            syn::UseTree::Glob(_) => {
                vec![NameFilter::Global]
            }
            syn::UseTree::Name(use_name) => {
                vec![NameFilter::Specific(use_name.ident.to_string())]
            }
            syn::UseTree::Rename(use_rename) => {
                vec![NameFilter::Rename(
                    use_rename.ident.to_string(),
                    use_rename.rename.to_string(),
                )]
            }
        }
    }

    /// Parses inline and external file modules
    pub fn parse_module(
        current_path: &Path,
        current_module: &syn::ItemMod,
    ) -> (PathBuf, Vec<Item>) {
        if let Some((_, items)) = &current_module.content {
            (current_path.to_path_buf(), items.to_vec())
        } else {
            let external_module_name = current_module.ident.to_string();
            parse_external_module(current_path, external_module_name)
        }
    }

    /// Parses a module from the filesystem
    fn parse_external_module(current_path: &Path, module_name: String) -> (PathBuf, Vec<Item>) {
        let current_dir = current_path.parent().unwrap();

        let module_path = if current_dir.join(format!("{}.rs", module_name)).exists() {
            current_dir.join(format!("{}.rs", module_name))
        } else {
            current_dir.join(module_name).join("mod.rs")
        };

        let content = fs::read_to_string(&module_path).unwrap_or_else(|_| {
            panic!(
                "Couldn't find module file: {}",
                module_path.to_str().unwrap()
            )
        });
        (
            module_path,
            parse_file(&content)
                .expect("Failed to parse module file")
                .items,
        )
    }

    /// Maps a struct name to its public fields
    fn extract_struct(item_struct: ItemStruct, signatures: &mut ApiMap, filter: NameFilter) {
        let struct_name = match filter {
            NameFilter::Global => item_struct.ident.to_string(),
            NameFilter::Specific(name) if item_struct.ident == name => name,
            NameFilter::Rename(name, rename) if item_struct.ident == name => rename,
            _ => return,
        };

        let mut fields = Vec::new();
        for field in &item_struct.fields {
            if let Visibility::Public(_) = field.vis {
                fields.push(extract_field(field));
            }
        }

        match signatures.entry(struct_name) {
            Entry::Vacant(entry) => {
                entry.insert(fields);
            }
            Entry::Occupied(mut entry) => {
                let vec = entry.get_mut();
                for f in fields {
                    if !vec.iter().any(|e| e == &f) {
                        vec.push(f);
                    }
                }
            }
        }
    }

    /// Maps a struct name to its public method signatures
    fn extract_impl(item_impl: ItemImpl, signatures: &mut ApiMap, filter: NameFilter) {
        let struct_name = match *item_impl.self_ty {
            syn::Type::Path(type_path) => type_path.path.segments.last().unwrap().ident.to_string(),
            _ => return,
        };

        let struct_name = match filter {
            NameFilter::Global => struct_name,
            NameFilter::Specific(name) if struct_name == name => name,
            NameFilter::Rename(name, rename) if struct_name == name => rename,
            _ => return,
        };

        let mut methods = Vec::new();
        for impl_item in &item_impl.items {
            if let ImplItem::Fn(method) = impl_item
                && matches!(method.vis, Visibility::Public(_))
            {
                methods.push(extract_signature(&method.sig));
            }
        }

        match signatures.entry(struct_name) {
            Entry::Vacant(entry) => {
                entry.insert(methods);
            }
            Entry::Occupied(mut entry) => {
                let vec = entry.get_mut();
                for m in methods {
                    if !vec.iter().any(|e| e == &m) {
                        vec.push(m);
                    }
                }
            }
        }
    }

    /// Maps a public function name to its signature
    fn extract_fn(path: &Path, item_fn: ItemFn, signatures: &mut ApiMap, filter: NameFilter) {
        if let Visibility::Public(_) = item_fn.vis {
            let mut signature = extract_signature(&item_fn.sig);

            signature.name = match filter {
                NameFilter::Global => signature.name,
                NameFilter::Specific(name) if signature.name == name => name,
                NameFilter::Rename(name, rename) if signature.name == name => rename,
                _ => return,
            };

            let ancestor = path
                .ancestors()
                .find(|ancestor| ancestor.ends_with("src"))
                .expect("Couldn't find parent /src directory");

            let key = path
                .strip_prefix(ancestor)
                .unwrap()
                .to_str()
                .unwrap()
                .replace('/', "_");

            match signatures.entry(key) {
                Entry::Vacant(entry) => {
                    entry.insert(vec![signature]);
                }
                Entry::Occupied(mut entry) => {
                    let vec = entry.get_mut();
                    if !vec.iter().any(|e| e == &signature) {
                        vec.push(signature);
                    }
                }
            }
        }
    }

    /// Extracts the name, parameters and return type of a function
    fn extract_signature(method: &syn::Signature) -> ObjectProperty {
        let name = method.ident.to_string();

        let parameters: Vec<FunctionParameter> = method
            .inputs
            .iter()
            .filter_map(|arg| {
                if let syn::FnArg::Typed(pattern) = arg {
                    let name = pattern.pat.to_token_stream().to_string();
                    let type_name = pattern.ty.to_token_stream().to_string();

                    Some(FunctionParameter { name, type_name })
                } else {
                    None
                }
            })
            .collect();

        let return_type: Option<String> = match &method.output {
            ReturnType::Default => None,
            ReturnType::Type(_, ty) => Some(ty.to_token_stream().to_string()),
        };

        ObjectProperty {
            name: name.clone(),
            type_name: "FunctionSignature".to_string(),
            function: Some(FunctionSignature {
                name,
                parameters,
                return_type,
            }),
        }
    }

    /// Extracts the name and type of a struct field
    fn extract_field(field: &syn::Field) -> ObjectProperty {
        let type_name = field.ty.to_token_stream().to_string();
        let name = if let Some(name) = &field.ident {
            name.to_string()
        } else {
            "".to_string()
        };

        ObjectProperty {
            name,
            type_name,
            function: None,
        }
    }

    /// Exports the generated API map to a static Rust file
    pub fn export_api_map(api_map: &ApiMap) {
        let target_file = super::meta::workspace_root().join(API_MAP_FILE);
        let mut static_map_builder = phf_codegen::Map::new();
        let mut target_file = fs::File::create(target_file).unwrap();
        let mut writer = std::io::BufWriter::new(&mut target_file);

        for (struct_name, functions) in api_map {
            static_map_builder.entry(
                struct_name.clone(),
                format!(
                    "&[{}]",
                    functions
                        .iter()
                        .map(|function| {
                            format!("{:?}, ", function).replace("parameters: [", "parameters: &[")
                        })
                        .collect::<String>()
                ),
            );
        }

        write!(
            &mut writer,
            "{}\n\nstatic {}: phf::Map<&'static str, &[ObjectProperty]> = {};\n",
            OBJECT_PROPERTY_STRUCT_DEFINITION,
            API_MAP_KEYWORD,
            static_map_builder.build()
        )
        .unwrap();
    }
}

/// Conversion utilities: transforms Rust example lines into idiomatic, runnable JS/Python
/// while preserving per-line alignment of visible content (hidden Rust lines starting
/// with '#' are never exported to JS/Python).
///
/// Rules implemented here (from user requirements):
/// - Skip hidden lines ('#') entirely for JS/Python exports.
/// - UFCS and associated functions: Type::method(&obj, ...) -> obj.method(...);
///   Static calls use dot form in JS/Python; special-case new()/from_shader()/default().
/// - Await/Result artifacts: `.await?` -> `await` in JS; removed in Python. Trailing '?' removed.
/// - JS naming: convert snake_case method names to camelCase everywhere (after '.' or 'Type.').
/// - Strip Rust refs `&` and `&mut`.
/// - Python Target.size is a property: `target.size()` -> `target.size`.
/// - Visible `.into()` is forbidden (panic). Keep examples minimal.
/// - Assert mapping: assert_eq!(a, b) -> JS one-liner throw; Python `assert a == b`.
/// - Shader::default(): map to example helpers for now to keep CI runnable:
///   JS -> globalThis.exampleShader(), Python -> example_shader() with an import injected.
mod convert {
    #[derive(Copy, Clone, Debug)]
    enum Lang {
        Js,
        Py,
    }

    pub fn to_js(items: &[(String, bool)]) -> String {
        convert(items, Lang::Js)
    }

    pub fn to_py(items: &[(String, bool)]) -> String {
        convert(items, Lang::Py)
    }

    fn is_ident_char(c: char) -> bool {
        c.is_ascii_alphanumeric() || c == '_'
    }

    // Helper: convert common Rust Vec constructs to JS/Python arrays
    fn convert_vec_syntax(s: &str) -> String {
        let mut out = s.replace("Vec::new()", "[]");
        out = out.replace("vec![", "[");
        out = out.replace("vec ![", "["); // tolerate space
        out
    }

    fn snake_to_camel(s: &str) -> String {
        let mut out = String::with_capacity(s.len());
        let mut upper = false;
        for ch in s.chars() {
            if ch == '_' {
                upper = true;
            } else if upper {
                out.push(ch.to_ascii_uppercase());
                upper = false;
            } else {
                out.push(ch);
            }
        }
        out
    }

    fn camelize_method_calls_js(line: &str) -> String {
        let mut out = String::with_capacity(line.len());
        let chars: Vec<char> = line.chars().collect();
        let mut i = 0usize;
        while i < chars.len() {
            let c = chars[i];
            out.push(c);
            if c == '.' {
                // capture identifier after '.'
                let start = i + 1;
                let mut j = start;
                while j < chars.len() && is_ident_char(chars[j]) {
                    j += 1;
                }
                if j > start {
                    let ident: String = chars[start..j].iter().collect();
                    let camel = snake_to_camel(&ident);
                    out.push_str(&camel);
                    i = j; // skip consumed ident; next loop pushes current char again, so avoid double push
                    continue;
                }
            }
            i += 1;
        }
        out
    }

    fn replace_static_call_to_dot(line: &str) -> String {
        // Replace patterns like Type::method( -> Type.method(
        let mut out = String::new();
        let mut i = 0usize;
        let bytes = line.as_bytes();
        while i < bytes.len() {
            if i + 2 < bytes.len() && bytes[i] == b':' && bytes[i + 1] == b':' {
                // Replace '::' with '.'
                out.push('.');
                i += 2;
                continue;
            }
            out.push(bytes[i] as char);
            i += 1;
        }
        out
    }

    fn strip_refs(s: &str) -> String {
        s.replace('&', "")
    }

    fn strip_trailing_semicolon(s: &str) -> String {
        let t = s.trim_end();
        if let Some(stripped) = t.strip_suffix(';') {
            stripped.to_string()
        } else {
            s.to_string()
        }
    }

    fn ensure_js_semicolon(s: &str) -> String {
        let t = s.trim_end();
        if t.is_empty() {
            return String::new();
        }
        if t.ends_with(';') {
            s.to_string()
        } else {
            format!("{};", s)
        }
    }

    fn transform_await(line: &str, lang: Lang) -> String {
        // Handle `.await?` -> `await` (JS) or remove (Py). Also drop Rust error `?` in JS/Py.
        let mut out = line.to_string();
        if out.contains(".await?") {
            match lang {
                Lang::Js => {
                    if let Some(pos) = out.find(".await?") {
                        // Preserve left side if present
                        if let Some(eq) = out[..pos].rfind('=') {
                            let (lhs, expr) = out.split_at(eq + 1);
                            let expr = expr.trim();
                            let before_await = &expr[..expr.rfind(".await?").unwrap_or(expr.len())];
                            let mut s = String::new();
                            s.push_str(lhs);
                            s.push(' ');
                            s.push_str("await ");
                            s.push_str(before_await.trim());
                            let tail = &out[pos + ".await?".len()..];
                            s.push_str(tail);
                            out = s;
                        } else {
                            let before_await = &out[..pos];
                            let mut s = String::new();
                            s.push_str("await ");
                            s.push_str(before_await.trim());
                            let tail = &out[pos + ".await?".len()..];
                            s.push_str(tail);
                            out = s;
                        }
                    }
                }
                Lang::Py => {
                    out = out.replace(".await?", "");
                }
            }
        }
        // Remove stray Rust error-propagation '?' for both langs (JS/Py)
        match lang {
            Lang::Js => {
                // Replace common patterns
                let mut s = out.replace(")?;", ");");
                s = s.replace(")?\n", ")\n");
                s = s.replace(")? ", ") ");
                if s.trim_end().ends_with('?') {
                    s = s.trim_end_matches('?').to_string();
                }
                s
            }
            Lang::Py => out.replace('?', ""),
        }
    }

    fn map_assert(line: &str, lang: Lang) -> Option<String> {
        // assert_eq!(a, b);
        let t = line.trim_start();
        if !t.starts_with("assert_eq!(") {
            return None;
        }
        let mut inner = &t["assert_eq!(".len()..];
        if let Some(end) = inner.find(')') {
            inner = &inner[..end];
        }
        let mut parts = inner.splitn(2, ',');
        let a = parts.next().unwrap_or("").trim();
        let b = parts.next().unwrap_or("").trim();
        match lang {
            Lang::Js => Some(format!(
                "if (JSON.stringify({}) !== JSON.stringify({})) {{ throw new Error(\"assert_eq failed\"); }}",
                a, b
            )),
            Lang::Py => Some(format!("assert {} == {}", a, b)),
        }
    }

    fn parse_use_fragmentcolor(line: &str) -> Option<Vec<String>> {
        // Returns a flat list of imported names from `use fragmentcolor::...` syntax
        let t = line.trim();
        if !t.starts_with("use fragmentcolor::") {
            return None;
        }
        let after = &t["use fragmentcolor::".len()..];
        if after.starts_with('{') {
            if let Some(end) = after.find('}') {
                let inside = &after[1..end];
                let list: Vec<String> = inside
                    .split(',')
                    .map(|p| p.trim().rsplit("::").next().unwrap_or("").to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                return Some(list);
            }
            None
        } else {
            let name = after.split(';').next().unwrap_or(after).trim();
            let short = name.rsplit("::").next().unwrap_or(name).to_string();
            Some(vec![short])
        }
    }

    fn handle_let_assignment(
        line: &str,
        lang: Lang,
        py_renames: &mut std::collections::HashMap<String, String>,
        js_renames: &mut std::collections::HashMap<String, String>,
        need_rendercanvas_import: &mut bool,
    ) -> Option<String> {
        let t = line.trim_start();
        if !t.starts_with("let ") {
            return None;
        }
        // strip leading `let` and optional `mut`
        let rest = t.trim_start_matches("let ").trim_start();
        let rest = if let Some(stripped) = rest.strip_prefix("mut ") {
            stripped
        } else {
            rest
        };
        let eq = rest.find('=')?;
        let (lhs, rhs0) = rest.split_at(eq);
        let mut var = lhs.trim();
        // Strip type annotation in `var: Type`
        if let Some(colon) = var.find(':') {
            var = var[..colon].trim();
        }
        let mut rhs = rhs0.trim_start_matches('=').trim().to_string();
        // Remove trailing ';'
        rhs = strip_trailing_semicolon(&rhs);

        // Replace fragmentcolor headless helpers with native canvases
        // Python: fragmentcolor::headless_window([w,h]) -> RenderCanvas(size=(w,h))
        // JS: fragmentcolor.headlessWindow([w,h]) -> (()=>{const c=document.createElement('canvas');c.width=w;c.height=h;return c;})()
        let rhs_lc = rhs.replace("::", ".");
        if rhs_lc.contains("headless_window(") || rhs_lc.contains("headlessWindow(") {
            match lang {
                Lang::Py => {
                    if let (Some(lp), Some(rp)) = (rhs.find('('), rhs.rfind(')')) {
                        let inside = rhs[lp + 1..rp].trim();
                        let inner = inside
                            .trim_start_matches('[')
                            .trim_end_matches(']')
                            .trim_start_matches('(')
                            .trim_end_matches(')')
                            .trim();
                        rhs = format!("RenderCanvas(size=({}))", inner);
                        *need_rendercanvas_import = true;
                        if var == "window" {
                            // rename variable to canvas in Python output and future references
                            py_renames.insert("window".into(), "canvas".into());
                        }
                    }
                }
                Lang::Js => {
                    if let (Some(lp), Some(rp)) = (rhs.find('('), rhs.rfind(')')) {
                        let inside = rhs[lp + 1..rp].trim();
                        let mut parts = inside
                            .trim_start_matches('[')
                            .trim_end_matches(']')
                            .split(',')
                            .map(|s| s.trim());
                        let w = parts.next().unwrap_or("1");
                        let h = parts.next().unwrap_or("1");
                        rhs = format!(
                            "(()=>{{const c=document.createElement('canvas');c.width={};c.height={};return c;}})()",
                            w, h
                        );
                        if var == "window" {
                            js_renames.insert("window".into(), "canvas".into());
                        }
                    }
                }
            }
        } else {
            // Type::new(args)
            if let Some(pos) = rhs.find("::new(") {
                let ty = rhs[..pos].trim().rsplit("::").next().unwrap_or("");
                let args_with = &rhs[pos + "::new(".len()..];
                if let Some(endp) = args_with.rfind(')') {
                    let args = &args_with[..endp];
                    rhs = match lang {
                        Lang::Js => format!("new {}({})", ty, args.trim()),
                        Lang::Py => format!("{}({})", ty, args.trim()),
                    };
                }
            }
        }
        // Pass::from_shader(name, shader) -> one-line construct+add
        if rhs.starts_with("Pass::from_shader(")
            || rhs.starts_with("fragmentcolor::Pass::from_shader(")
        {
            // Extract args inside (...)
            if let Some(lp) = rhs.find('(')
                && let Some(rp) = rhs.rfind(')')
            {
                let inside = &rhs[lp + 1..rp];
                let mut parts = inside.splitn(2, ',');
                let a1 = parts.next().unwrap_or("").trim();
                let mut a2 = parts.next().unwrap_or("").trim().to_string();
                a2 = strip_refs(&a2);
                match lang {
                    Lang::Js => {
                        rhs = format!("new Pass({}); {}.addShader({})", a1, var, a2);
                    }
                    Lang::Py => {
                        rhs = format!("Pass({}); {}.add_shader({})", a1, var, a2);
                    }
                }
            }
        }

        // UFCS associated calls remaining: Type::method( -> Type.method(
        rhs = replace_static_call_to_dot(&rhs);
        // Strip refs '&'
        rhs = strip_refs(&rhs);
        // Await transform and remove '?' for lang
        rhs = transform_await(&rhs, lang);

        // JS: camelize method names after '.'
        if let Lang::Js = lang {
            rhs = camelize_method_calls_js(&rhs);
        }
        // Python: size() -> size property and '//' comment to '#'
        if let Lang::Py = lang {
            rhs = rhs.replace(".size()", ".size");
            if let Some(idx) = rhs.find("//") {
                let (mut head, tail) = rhs.split_at(idx);
                head = head.trim_end_matches(';').trim_end();
                rhs = format!("{}#{}", head, &tail[2..]);
            }
        }

        // Var rename for Python reserved keyword "pass" and window->canvas mapping
        let var_out: &str = match lang {
            Lang::Py => {
                if var == "pass" {
                    py_renames.insert("pass".into(), "rpass".into());
                    "rpass"
                } else if var == "window" && py_renames.get("window").is_some() {
                    "canvas"
                } else {
                    var
                }
            }
            Lang::Js => {
                if var == "window" && js_renames.get("window").is_some() {
                    "canvas"
                } else {
                    var
                }
            }
        };

        // If RHS references the original var (e.g., Pass::from_shader expansion), adjust to var_out for Python
        if let Lang::Py = lang {
            let needle = format!("{}.", var);
            let replacement = format!("{}.", var_out);
            rhs = rhs.replace(&needle, &replacement);
        }

        // Convert Rust vec! macros and Vec::new() in RHS to JS/Python arrays
        rhs = convert_vec_syntax(&rhs);

        // Apply pending variable renames inside RHS for both languages (e.g., window->canvas)
        rhs = match lang {
            Lang::Py => apply_renames_py(&rhs, py_renames),
            Lang::Js => apply_renames_py(&rhs, js_renames),
        };

        let line_out = match lang {
            Lang::Js => ensure_js_semicolon(&format!("const {} = {}", var_out, rhs)),
            Lang::Py => format!("{} = {}", var_out, rhs),
        };
        Some(line_out)
    }

    fn apply_renames_py(s: &str, renames: &std::collections::HashMap<String, String>) -> String {
        if renames.is_empty() {
            return s.to_string();
        }
        fn replace_word(src: &str, from: &str, to: &str) -> String {
            let bytes: Vec<char> = src.chars().collect();
            let from_chars: Vec<char> = from.chars().collect();
            let mut out = String::with_capacity(src.len());
            let mut i = 0usize;
            while i < bytes.len() {
                // Try to match `from` at position i with word boundaries
                let end = i + from_chars.len();
                if end <= bytes.len() && bytes[i..end] == from_chars[..] {
                    let left_ok = i == 0 || !super::convert::is_ident_char(bytes[i - 1]);
                    let right_ok = end == bytes.len() || !super::convert::is_ident_char(bytes[end]);
                    if left_ok && right_ok {
                        out.push_str(to);
                        i = end;
                        continue;
                    }
                }
                out.push(bytes[i]);
                i += 1;
            }
            out
        }
        let mut out = s.to_string();
        for (from, to) in renames {
            out = replace_word(&out, from, to);
        }
        out
    }

    fn convert(items: &[(String, bool)], lang: Lang) -> String {
        use std::collections::HashMap;
        // Collect visible lines only
        let mut src: Vec<String> = Vec::new();
        for (t, hidden) in items {
            if !*hidden {
                src.push(t.clone());
            }
        }
        // Pre-scan usage to adjust imports (Shader usage in examples)
        let uses_shader = src.iter().any(|l| {
            let t = l.trim();
            t.contains("Shader::")
                || t.contains("fragmentcolor::Shader")
                || t.contains("fragmentcolor.Shader")
                || t.contains("Shader.default(")
        });
        // Enforce no visible into()
        for line in &src {
            if line.contains(".into(") || line.contains(".into()") || line.contains(" into(") {
                panic!(
                    "Visible .into() found in an example. Hide it with '#' or remove it: \n {}",
                    line
                );
            }
        }
        // Enforce no visible pollster
        for line in &src {
            if line.contains("pollster") {
                panic!(
                    "Visible pollster found in an example. Hide it with '#' or remove it: \n {}",
                    line
                );
            }
        }
        // Enforce no visible pollster
        for line in &src {
            if line.contains("assert_eq") {
                panic!(
                    "Visible assert_eq found in an example. Hide it with '#' or remove it: \n {}",
                    line
                );
            }
        }

        let mut out: Vec<String> = Vec::new();
        let mut py_renames: HashMap<String, String> = HashMap::new();
        let mut js_renames: HashMap<String, String> = HashMap::new();
        let mut need_rendercanvas_import: bool = false;

        for s in &src {
            let t = s.trim();
            if t.is_empty() {
                out.push(String::new());
                continue;
            }

            // Imports from fragmentcolor
            if let Some(mut list) = parse_use_fragmentcolor(t) {
                // Ensure Shader is imported if used in snippet
                if uses_shader && !list.iter().any(|s| s == "Shader") {
                    list.push("Shader".to_string());
                }
                match lang {
                    Lang::Js => out.push(format!(
                        "import {{ {} }} from \"fragmentcolor\";",
                        list.join(", ")
                    )),
                    Lang::Py => {
                        // Python package does not expose Target/WindowTarget/TextureTarget as importable symbols
                        list.retain(|s| {
                            s != "Target" && s != "WindowTarget" && s != "TextureTarget"
                        });
                        out.push(format!("from fragmentcolor import {}", list.join(", ")))
                    }
                }
                continue;
            }

            // Map assert_eq!
            if let Some(mapped) = map_assert(t, lang) {
                out.push(match lang {
                    Lang::Js => ensure_js_semicolon(&mapped),
                    Lang::Py => mapped,
                });
                continue;
            }

            // let assignments
            if let Some(mapped) = handle_let_assignment(
                t,
                lang,
                &mut py_renames,
                &mut js_renames,
                &mut need_rendercanvas_import,
            ) {
                out.push(mapped);
                continue;
            }

            // General expression/method calls
            let mut line = s.to_string();

            // 1) UFCS static call -> dot first
            line = replace_static_call_to_dot(&line);

            // 1.1) Convert Vec syntax (vec![], Vec::new()) into JS/Py arrays
            line = convert_vec_syntax(&line);

            // 2) Drop explicit module prefix for JS/Py when present: fragmentcolor.Shader -> Shader
            if matches!(lang, Lang::Js | Lang::Py) {
                line = line.replace("fragmentcolor.Shader", "Shader");
            }

            // 3) No special handling for Shader::default(); keep the call intact so languages use their own default().

            // 4) Strip refs
            line = strip_refs(&line);

            // 5) Await / remove error '?' artifacts
            line = transform_await(&line, lang);

            // 6) Python size property + comment conversion
            if let Lang::Py = lang {
                line = line.replace(".size()", ".size");
                // Convert JS-style '//' comments to Python '#'
                if let Some(idx) = line.find("//") {
                    let (mut head, tail) = line.split_at(idx);
                    // Strip any trailing semicolon immediately before comment
                    head = head.trim_end_matches(';').trim_end();
                    line = format!("{} #{}", head, &tail[2..]);
                }
            }

            // 7) JS camelize methods
            if let Lang::Js = lang {
                line = camelize_method_calls_js(&line);
            }

            // 8) Language-specific trailing cleanup
            match lang {
                Lang::Js => {
                    line = ensure_js_semicolon(&line);
                }
                Lang::Py => {
                    line = strip_trailing_semicolon(&line).replace('?', "");
                }
            }

            // 9) Apply var renames in Python/JS lines after we possibly introduced references
            match lang {
                Lang::Py => {
                    line = apply_renames_py(&line, &py_renames);
                }
                Lang::Js => {
                    line = apply_renames_py(&line, &js_renames);
                }
            }

            out.push(line);
        }

        // Prepend RenderCanvas import if needed for Python examples
        if need_rendercanvas_import {
            let mut out2 = Vec::with_capacity(out.len() + 1);
            out2.push("from rendercanvas.auto import RenderCanvas, loop".to_string());
            out2.extend(out);
            out = out2;
        }

        // Normalize ending newline joining in caller
        out.join("\n")
    }
}

mod validation {
    use crate::codegen::ApiMap;

    use super::*;
    use std::fs;
    use std::path::Path;

    fn to_snake_case(s: &str) -> String {
        // Basic snake_case converter for method names (already snake in Rust)
        s.to_string()
    }

    fn object_dir_name(object: &str) -> String {
        // Convert CamelCase to snake_case for directory names
        let mut out = String::new();
        for (i, ch) in object.chars().enumerate() {
            if ch.is_uppercase() {
                if i != 0 {
                    out.push('_');
                }
                out.push(ch.to_ascii_lowercase());
            } else {
                out.push(ch);
            }
        }
        out
    }

    fn ensure_object_md_ok(object: &str, path: &Path, problems: &mut Vec<String>) {
        if !path.exists() {
            problems.push(format!("Missing object file: {}", path.display()));
            return;
        }
        let content = fs::read_to_string(path).unwrap_or_default();
        if content.trim().is_empty() {
            problems.push(format!("Empty object file: {}", path.display()));
        }
        if !content.lines().any(|l| l.trim() == format!("# {}", object)) {
            problems.push(format!("{}: H1 '# {}' not found", path.display(), object));
        }
        if !content.contains("## Example") {
            problems.push(format!("{}: '## Example' section missing", path.display()));
        }
        if content.contains("fragmentcolor.com") {
            problems.push(format!("{}: contains fragmentcolor.com", path.display()));
        }
        // BLOCK build on legacy winit API usage in public examples
        if content.contains("WindowBuilder") || content.contains("EventLoop") {
            problems.push(format!(
                "{}: legacy winit API (WindowBuilder/EventLoop) detected in docs — replace with RenderCanvas/HTMLCanvas",
                path.display()
            ));
        }
    }

    fn ensure_method_md_ok(object: &str, method: &str, path: &Path, problems: &mut Vec<String>) {
        if !path.exists() {
            problems.push(format!(
                "Missing method file for {}.{}: {}",
                object,
                method,
                path.display()
            ));
            return;
        }
        let content = fs::read_to_string(path).unwrap_or_default();
        if content.trim().is_empty() {
            problems.push(format!("Empty method file: {}", path.display()));
        }
        if !content.lines().any(|l| l.trim().starts_with('#')) {
            problems.push(format!("{}: H1 heading missing", path.display()));
        } else {
            // Enforce method title starts with "<Object>::"
            if let Some(head) = content
                .lines()
                .find(|l| l.trim_start().starts_with('#'))
            {
                let mut t = head.trim_start();
                while t.starts_with('#') { t = t[1..].trim_start(); }
                let expected_prefix = format!("{}::", object);
                if !t.starts_with(&expected_prefix) {
                    problems.push(format!(
                        "{}: method title must start with '{}' (e.g., '{}{}(...)')",
                        path.display(),
                        expected_prefix,
                        expected_prefix,
                        method
                    ));
                }
            }
        }
        if !content.contains("## Example") {
            problems.push(format!("{}: '## Example' section missing", path.display()));
        }
        if content.contains("fragmentcolor.com") {
            problems.push(format!("{}: contains fragmentcolor.com", path.display()));
        }
    }

    /// Locate a method doc within any nested `hidden/` subdirectory under the given object directory.
    /// Returns Some(path) if a file named `<file_stem>.md` exists below a path that contains a
    /// directory segment named `hidden`; otherwise None.
    fn find_hidden_method_doc(
        obj_dir: &std::path::Path,
        file_stem: &str,
    ) -> Option<std::path::PathBuf> {
        fn contains_hidden(mut p: &std::path::Path) -> bool {
            while let Some(parent) = p.parent() {
                if parent.file_name().and_then(|s| s.to_str()) == Some("hidden") {
                    return true;
                }
                p = parent;
            }
            false
        }
        fn walk(dir: &std::path::Path, file_stem: &str, out: &mut Option<std::path::PathBuf>) {
            if out.is_some() {
                return;
            }
            if let Ok(rd) = std::fs::read_dir(dir) {
                for entry in rd.flatten() {
                    let p = entry.path();
                    if p.is_dir() {
                        walk(&p, file_stem, out);
                        if out.is_some() {
                            return;
                        }
                    } else if p.extension().and_then(|s| s.to_str()) == Some("md")
                        && p.file_stem().and_then(|s| s.to_str()) == Some(file_stem)
                        && contains_hidden(&p)
                    {
                        *out = Some(p);
                        return;
                    }
                }
            }
        }
        let mut found = None;
        walk(obj_dir, file_stem, &mut found);
        found
    }

    pub fn validate_docs(api_map: &ApiMap) {
        let mut problems = Vec::new();
        let mut warnings = Vec::new();
        let root = meta::workspace_root();
        let docs_root = root.join("docs/api");

        // Helper: find the directory for an object recursively (must contain <dir>/<dir>.md)
        fn find_object_dir(
            docs_root: &std::path::Path,
            dir_name: &str,
        ) -> Option<std::path::PathBuf> {
            fn walk(root: &std::path::Path, target: &str) -> Option<std::path::PathBuf> {
                if !root.is_dir() {
                    return None;
                }
                for entry in std::fs::read_dir(root).ok()?.flatten() {
                    let p = entry.path();
                    if p.is_dir() {
                        if p.file_name().and_then(|s| s.to_str()) == Some(target) {
                            let md = p.join(format!("{}.md", target));
                            if md.exists() {
                                return Some(p);
                            }
                        }
                        if let Some(found) = walk(&p, target) {
                            return Some(found);
                        }
                    }
                }
                None
            }
            walk(docs_root, dir_name)
        }

        // Helper: recursively enumerate all object dirs found under docs_root
        fn scan_docs_objects(docs_root: &std::path::Path) -> Vec<(String, std::path::PathBuf)> {
            fn walk(
                dir: &std::path::Path,
                _root: &std::path::Path,
                out: &mut Vec<(String, std::path::PathBuf)>,
            ) {
                if !dir.is_dir() {
                    return;
                }
                for entry in std::fs::read_dir(dir).ok().into_iter().flatten().flatten() {
                    let p = entry.path();
                    if p.is_dir() {
                        if let Some(name) = p.file_name().and_then(|s| s.to_str()) {
                            let md = p.join(format!("{}.md", name));
                            if md.exists() {
                                let object = dir_to_object_name(name);
                                out.push((object, p.clone()));
                                // Do not descend into this object dir further
                                continue;
                            }
                        }
                        walk(&p, _root, out);
                    }
                }
            }
            let mut out = Vec::new();
            walk(docs_root, docs_root, &mut out);
            out
        }

        // Discover all objects and their categories under docs/api (recursively)
        fn scan_docs_objects_with_cat(
            docs_root: &std::path::Path,
        ) -> Vec<(
            String,             /*object name*/
            std::path::PathBuf, /*dir*/
            String,             /*cat_rel*/
            String,             /*dir_slug*/
        )> {
            fn walk(
                dir: &std::path::Path,
                root: &std::path::Path,
                out: &mut Vec<(String, std::path::PathBuf, String, String)>,
            ) {
                if !dir.is_dir() {
                    return;
                }
                if let Ok(rd) = std::fs::read_dir(dir) {
                    for entry in rd.flatten() {
                        let p = entry.path();
                        if p.is_dir() {
                            if let Some(dir_name) = p.file_name().and_then(|s| s.to_str()) {
                                let md = p.join(format!("{}.md", dir_name));
                                if md.exists() {
                                    let object = dir_to_object_name(dir_name);
                                    let parent = p.parent().unwrap_or(root);
                                    let cat_rel = if let Ok(rel) = parent.strip_prefix(root) {
                                        rel.to_string_lossy().replace('\\', "/")
                                    } else {
                                        String::new()
                                    };
                                    out.push((object, p.clone(), cat_rel, dir_name.to_string()));
                                    // Do not descend further into an object dir
                                    continue;
                                }
                            }
                            walk(&p, root, out);
                        }
                    }
                }
            }
            let mut out = Vec::new();
            walk(docs_root, docs_root, &mut out);
            out
        }

        fn canonical_url(cat_rel: &str, object_lower: &str) -> String {
            let mut url = String::from("https://fragmentcolor.org/api/");
            if !cat_rel.is_empty() {
                url.push_str(cat_rel);
                url.push('/');
            }
            url.push_str(object_lower);
            url
        }

        // Fix incorrect absolute links and auto-link unlinked object names in source docs
        fn rewrite_source_links_and_autolink(docs_root: &std::path::Path) {
            use std::collections::HashMap;

            let objects = scan_docs_objects_with_cat(docs_root);

            // Build maps: by object name (raw), by lowercased object slug, by folder dir slug
            let mut by_name: HashMap<String, (String /*cat*/, String /*obj_lower*/)> =
                HashMap::new();
            let mut by_obj_slug: HashMap<String, (String, String)> = HashMap::new();
            let mut by_dir_slug: HashMap<String, (String, String)> = HashMap::new();
            for (name, _dir, cat, dir_slug) in &objects {
                let obj_lower = name.to_lowercase();
                by_name.insert(name.clone(), (cat.clone(), obj_lower.clone()));
                by_obj_slug.insert(obj_lower.clone(), (cat.clone(), obj_lower.clone()));
                by_dir_slug.insert(dir_slug.clone(), (cat.clone(), obj_lower.clone()));
            }

            fn is_word_boundary(ch: Option<char>) -> bool {
                match ch {
                    None => true,
                    Some(c) => !c.is_alphanumeric() && c != '_',
                }
            }

            fn process_file(
                path: &std::path::Path,
                by_name: &std::collections::HashMap<String, (String, String)>,
                by_obj_slug: &std::collections::HashMap<String, (String, String)>,
                by_dir_slug: &std::collections::HashMap<String, (String, String)>,
            ) {
                let Ok(src) = std::fs::read_to_string(path) else {
                    return;
                };
                let mut changed = false;

                // Pass 1: fix absolute links
                let mut out = String::new();
                let bytes = src.as_bytes();
                let mut i = 0usize;

                let needle_https = "](https://fragmentcolor.org/api/".as_bytes();
                let needle_http = "](http://fragmentcolor.org/api/".as_bytes();
                let needle_www_https = "](https://www.fragmentcolor.org/api/".as_bytes();
                let needle_www_http = "](http://www.fragmentcolor.org/api/".as_bytes();

                while i < bytes.len() {
                    let mut matched = None::<&[u8]>;
                    if i + needle_https.len() <= bytes.len()
                        && &bytes[i..i + needle_https.len()] == needle_https
                    {
                        matched = Some(needle_https);
                    } else if i + needle_http.len() <= bytes.len()
                        && &bytes[i..i + needle_http.len()] == needle_http
                    {
                        matched = Some(needle_http);
                    } else if i + needle_www_https.len() <= bytes.len()
                        && &bytes[i..i + needle_www_https.len()] == needle_www_https
                    {
                        matched = Some(needle_www_https);
                    } else if i + needle_www_http.len() <= bytes.len()
                        && &bytes[i..i + needle_www_http.len()] == needle_www_http
                    {
                        matched = Some(needle_www_http);
                    }

                    if let Some(m) = matched {
                        out.push_str("](");
                        i += m.len();
                        // Capture until ')'
                        let start = i;
                        while i < bytes.len() && bytes[i] != b')' {
                            i += 1;
                        }
                        let tail = &src[start..i];
                        let mut parts: Vec<&str> =
                            tail.split('/').filter(|s| !s.is_empty()).collect();
                        if let Some(last) = parts.pop() {
                            let key = last.to_string();
                            let best = by_obj_slug
                                .get(&key)
                                .or_else(|| by_dir_slug.get(&key))
                                .cloned();
                            if let Some((cat, obj_lower)) = best {
                                let canon = canonical_url(&cat, &obj_lower);
                                out.push_str(&canon);
                                changed = true;
                            } else {
                                out.push_str("https://fragmentcolor.org/api/");
                                out.push_str(tail);
                            }
                        } else {
                            out.push_str("https://fragmentcolor.org/api/");
                            out.push_str(tail);
                        }
                    } else {
                        out.push(bytes[i] as char);
                        i += 1;
                    }
                }

                let content = if out.is_empty() { src.clone() } else { out };
                if content != src {
                    changed = true;
                }

                // Pass 2: auto-link unlinked names (skip code fences, inline code, and headings)
                let mut names: Vec<&String> = by_name.keys().collect();
                names.sort_by_key(|s| std::cmp::Reverse(s.len()));

                let mut result = String::new();
                let mut in_code_block = false;
                for line in content.lines() {
                    let trimmed = line.trim_start();
                    if trimmed.starts_with("```") {
                        in_code_block = !in_code_block;
                        result.push_str(line);
                        result.push('\n');
                        continue;
                    }
                    if trimmed.starts_with('#') || in_code_block {
                        result.push_str(line);
                        result.push('\n');
                        continue;
                    }

                    let mut i = 0usize;
                    let chars: Vec<char> = line.chars().collect();
                    let mut in_inline = false;
                    while i < chars.len() {
                        let c = chars[i];
                        if c == '`' {
                            in_inline = !in_inline;
                            result.push(c);
                            i += 1;
                            continue;
                        }
                        if !in_inline && c == '[' {
                            // Copy existing link intact
                            let mut j = i + 1;
                            while j < chars.len() && chars[j] != ']' {
                                j += 1;
                            }
                            if j < chars.len() && j + 1 < chars.len() && chars[j + 1] == '(' {
                                let mut k = j + 2;
                                while k < chars.len() && chars[k] != ')' {
                                    k += 1;
                                }
                                let upper = k.min(chars.len().saturating_sub(1));
                                for ch in chars.iter().take(upper + 1).skip(i) {
                                    result.push(*ch);
                                }
                                i = k.min(chars.len());
                                if i < chars.len() && chars[i] == ')' {
                                    i += 1;
                                }
                                continue;
                            }
                        }

                        if in_inline {
                            result.push(c);
                            i += 1;
                            continue;
                        }

                        let mut matched_any = false;
                        for name in &names {
                            let name_chars: Vec<char> = name.chars().collect();
                            let end = i + name_chars.len();
                            if end <= chars.len()
                                && chars[i..end].iter().copied().eq(name_chars.iter().copied())
                                && is_word_boundary(
                                    i.checked_sub(1).and_then(|p| chars.get(p)).copied(),
                                )
                                && is_word_boundary(chars.get(end).copied())
                            {
                                let (cat, obj_lower) = by_name.get(*name).cloned().unwrap();
                                let url = canonical_url(&cat, &obj_lower);
                                result.push('[');
                                result.push_str(name);
                                result.push_str("](");
                                result.push_str(&url);
                                result.push(')');
                                i = end;
                                matched_any = true;
                                changed = true;
                                break;
                            }
                        }
                        if !matched_any {
                            result.push(c);
                            i += 1;
                        }
                    }
                    if !line.ends_with('\n') {
                        result.push('\n');
                    }
                }

                if changed {
                    let _ = std::fs::write(path, result);
                }
            }

            fn walk_files(dir: &std::path::Path, cb: &dyn Fn(&std::path::Path)) {
                if let Ok(rd) = std::fs::read_dir(dir) {
                    for e in rd.flatten() {
                        let p = e.path();
                        if p.is_dir() {
                            walk_files(&p, cb);
                        } else if p.extension().and_then(|s| s.to_str()) == Some("md") {
                            cb(&p);
                        }
                    }
                }
            }

            walk_files(docs_root, &|p| {
                process_file(p, &by_name, &by_obj_slug, &by_dir_slug)
            });
        }

        rewrite_source_links_and_autolink(&docs_root);

        // Enforce documentation for ALL public objects (including wrappers)
        let objects = public_structs_excluding_hidden();
        let _all_objects = objects.clone();

        // Validate objects and their methods
        for object in objects.iter() {
            let methods_vec = api_map.get(object).cloned().unwrap_or_default();
            let dir = object_dir_name(object);
            let obj_dir = find_object_dir(&docs_root, &dir).unwrap_or(docs_root.join(&dir));
            let object_md = obj_dir.join(format!("{}.md", dir));
            ensure_object_md_ok(object, &object_md, &mut problems);
            // Link enforcement disabled; links are auto-rewritten during export

            for m in &methods_vec {
                if let Some(fun) = &m.function {
                    let name = &fun.name;

                    let file = to_snake_case(name);
                    let path = obj_dir.join(format!("{}.md", file));
                    if path.exists() {
                        ensure_method_md_ok(object, name, &path, &mut problems);
                    } else if let Some(hidden_path) = find_hidden_method_doc(&obj_dir, &file) {
                        warnings.push(format!(
                            "Hidden method: {}.{} ({})",
                            object,
                            name,
                            hidden_path.display()
                        ));
                    } else {
                        ensure_method_md_ok(object, name, &path, &mut problems);
                    }
                    // Link enforcement disabled; links are auto-rewritten during export
                }
            }
        }

        // Also validate any docs-only objects under docs/api not present in allowed (recursively)
        for (object, obj_dir) in scan_docs_objects(&docs_root) {
            if objects.iter().any(|o| o == &object) {
                continue;
            }
            let dir_name = obj_dir.file_name().and_then(|s| s.to_str()).unwrap_or("");
            let object_md = obj_dir.join(format!("{}.md", dir_name));
            ensure_object_md_ok(&object, &object_md, &mut problems);
            // Link enforcement disabled; links are auto-rewritten during export
        }

        if !warnings.is_empty() {
            eprintln!("\nDocumentation warnings:\n");
            for w in &warnings {
                eprintln!("- {}", w);
            }
        }

        if !problems.is_empty() {
            eprintln!("\nDocumentation validation failed with the following issues:\n");
            for p in &problems {
                eprintln!("- {}", p);
            }
            panic!("documentation incomplete");
        }

        // Enforce that all public structs and their public methods have #[lsp_doc]
        enforce_lsp_doc_coverage(&objects, &mut problems);

        // If we reach here, validation passed.
    }

    /// Explicit function to export the website (examples + pages)
    pub fn export_website(api_map: &ApiMap) {
        website::update(api_map);
    }

    pub fn dir_to_object_name(dir: &str) -> String {
        let mut out = String::new();
        let mut capitalize = true;
        for ch in dir.chars() {
            if ch == '_' {
                capitalize = true;
                continue;
            }
            if capitalize {
                out.push(ch.to_ascii_uppercase());
            } else {
                out.push(ch);
            }
            capitalize = false;
        }
        out
    }

    pub fn public_structs_excluding_hidden() -> Vec<String> {
        use syn::{Item, Visibility};
        let entry = super::codegen::parse_lib_entry_point(&meta::workspace_root());
        fn is_doc_hidden(attrs: &[syn::Attribute]) -> bool {
            use quote::ToTokens;
            attrs.iter().any(|a| {
                a.to_token_stream().to_string().contains("doc")
                    && a.to_token_stream().to_string().contains("hidden")
            })
        }
        fn walk(path: &std::path::Path, items: Vec<Item>, out: &mut Vec<String>) {
            for item in items {
                match item {
                    Item::Mod(m) => {
                        if let Visibility::Public(_) = m.vis {
                            let (mod_path, mod_items) = super::codegen::parse_module(path, &m);
                            walk(&mod_path, mod_items, out);
                        }
                    }
                    Item::Struct(s) => {
                        if let Visibility::Public(_) = s.vis
                            && !is_doc_hidden(&s.attrs)
                            && has_lsp_doc(&s.attrs)
                        {
                            let name = s.ident.to_string();
                            if !out.contains(&name) {
                                out.push(name);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        let mut out = Vec::new();
        walk(entry.0.as_path(), entry.1.items, &mut out);
        out.sort();
        out
    }

    fn collect_public_structs_info() -> Vec<(String, Vec<syn::Attribute>, Vec<String>)> {
        use syn::{Item, Visibility};
        let entry = codegen::parse_lib_entry_point(&meta::workspace_root());
        fn is_doc_hidden(attrs: &[syn::Attribute]) -> bool {
            use quote::ToTokens;
            attrs.iter().any(|a| {
                a.to_token_stream().to_string().contains("doc")
                    && a.to_token_stream().to_string().contains("hidden")
            })
        }
        fn collect_type_idents(ty: &syn::Type, out: &mut Vec<String>) {
            use syn::{GenericArgument, PathArguments, Type};
            match ty {
                Type::Path(tp) => {
                    if let Some(seg) = tp.path.segments.last() {
                        out.push(seg.ident.to_string());
                    }
                    for seg in &tp.path.segments {
                        if let PathArguments::AngleBracketed(ab) = &seg.arguments {
                            for arg in &ab.args {
                                if let GenericArgument::Type(inner) = arg {
                                    collect_type_idents(inner, out);
                                }
                            }
                        }
                    }
                }
                Type::Reference(r) => collect_type_idents(&r.elem, out),
                Type::Paren(p) => collect_type_idents(&p.elem, out),
                Type::Group(g) => collect_type_idents(&g.elem, out),
                Type::Tuple(t) => {
                    for elem in &t.elems {
                        collect_type_idents(elem, out);
                    }
                }
                Type::Array(a) => collect_type_idents(&a.elem, out),
                _ => {}
            }
        }
        fn walk(
            path: &std::path::Path,
            items: Vec<Item>,
            out: &mut Vec<(String, Vec<syn::Attribute>, Vec<String>)>,
        ) {
            for item in items {
                match item {
                    Item::Mod(m) => {
                        if let Visibility::Public(_) = m.vis {
                            let (mod_path, mod_items) = super::codegen::parse_module(path, &m);
                            walk(&mod_path, mod_items, out);
                        }
                    }
                    Item::Struct(s) => {
                        if let Visibility::Public(_) = s.vis
                            && !is_doc_hidden(&s.attrs)
                        {
                            let name = s.ident.to_string();
                            let mut inner_types = Vec::new();
                            match &s.fields {
                                syn::Fields::Unnamed(unnamed) => {
                                    if unnamed.unnamed.len() == 1 {
                                        collect_type_idents(
                                            &unnamed.unnamed[0].ty,
                                            &mut inner_types,
                                        );
                                    }
                                }
                                syn::Fields::Named(named) => {
                                    for f in named.named.iter() {
                                        collect_type_idents(&f.ty, &mut inner_types);
                                    }
                                }
                                syn::Fields::Unit => {}
                            }
                            // Dedup to reduce noise like [Option, WindowTarget, WindowTarget]
                            inner_types.sort();
                            inner_types.dedup();
                            out.push((name, s.attrs.clone(), inner_types));
                        }
                    }
                    _ => {}
                }
            }
        }
        let mut out = Vec::new();
        walk(entry.0.as_path(), entry.1.items, &mut out);
        out
    }

    pub fn base_public_objects() -> Vec<String> {
        let info = collect_public_structs_info();
        // Consider only documented types as canonical API objects.
        let documented: std::collections::HashSet<String> = info
            .iter()
            .filter(|(_, attrs, _)| has_lsp_doc(attrs))
            .map(|(n, _, _)| n.clone())
            .collect();

        // A struct is a wrapper if:
        // - It is a tuple newtype around a documented type, OR
        // - It contains a field that references a documented type (possibly nested in generics like Option<T>, Arc<T>, etc.)
        // We detect by checking whether any collected inner type idents match a documented name.
        let wrapper_names: std::collections::HashSet<String> = info
            .iter()
            .filter(|(n, _attrs, inner)| {
                // Prevent self-matching in degenerate cases
                inner.iter().any(|t| documented.contains(t) && t != n)
            })
            .map(|(n, _, _)| n.clone())
            .collect();

        // Base objects are documented types that are not wrappers
        let base: Vec<String> = info
            .iter()
            .filter(|(n, attrs, _)| {
                documented.contains(n) && !wrapper_names.contains(n) && has_lsp_doc(attrs)
            })
            .map(|(n, _, _)| n.clone())
            .collect();
        base
    }

    fn has_lsp_doc(attrs: &[syn::Attribute]) -> bool {
        attrs.iter().any(|a| a.path().is_ident("lsp_doc"))
    }

    #[allow(dead_code)]
    fn object_url(name: &str) -> String {
        format!("https://fragmentcolor.org/api/{}", object_dir_name(name))
    }

    #[allow(dead_code)]
    fn enforce_links_in_file(path: &Path, objects: &[String], problems: &mut Vec<String>) {
        let content = fs::read_to_string(path).unwrap_or_default();
        if content.is_empty() {
            return;
        }

        // Pre-sort names by length desc to avoid partial hits like Target in TextureTarget
        let mut names = objects.to_owned();
        names.sort_by_key(|s| std::cmp::Reverse(s.len()));

        let mut in_code_block = false;
        for line in content.lines() {
            let trimmed = line.trim_start();
            if trimmed.starts_with("```") {
                in_code_block = !in_code_block;
                continue;
            }
            if in_code_block {
                continue;
            }
            if trimmed.starts_with('#') {
                continue;
            } // skip headings

            // Strip inline code spans delimited by backticks
            let mut cleaned = String::new();
            let mut in_tick = false;
            for ch in line.chars() {
                if ch == '`' {
                    in_tick = !in_tick;
                    continue;
                }
                if !in_tick {
                    cleaned.push(ch);
                }
            }

            for obj in &names {
                let url = object_url(obj);
                let good = format!("[{}]({})", obj, url);

                // If there is any [Obj](...) but not fully qualified, flag it explicitly.
                if cleaned.contains(&format!("[{}](", obj)) && !cleaned.contains(&good) {
                    problems.push(format!(
                        "{}: Incorrect link for {} (must be [{}]({}))",
                        path.display(),
                        obj,
                        obj,
                        url
                    ));
                    continue;
                }

                // Remove all correct links to avoid false positives
                let cleaned = cleaned.replace(&good, "");

                // Search for unlinked mentions as whole tokens
                let mut start = 0usize;
                while let Some(idx_rel) = cleaned[start..].find(obj) {
                    let idx = start + idx_rel;
                    let before = cleaned[..idx].chars().next_back();
                    let after = cleaned[idx + obj.len()..].chars().next();
                    let before_ok = before.is_none_or(|c| !c.is_alphanumeric() && c != '_');
                    let after_ok = after.is_none_or(|c| !c.is_alphanumeric() && c != '_');
                    if before_ok && after_ok {
                        problems.push(format!(
                            "{}: Unlinked mention of {} (must be [{}]({}))",
                            path.display(),
                            obj,
                            obj,
                            url
                        ));
                        break;
                    }
                    start = idx + obj.len();
                }
            }
        }
    }

    pub fn enforce_lsp_doc_coverage(objects: &[String], problems: &mut Vec<String>) {
        use syn::{ImplItem, Item, Visibility};
        let entry = super::codegen::parse_lib_entry_point(&meta::workspace_root());
        use quote::ToTokens;
        fn is_doc_hidden(attrs: &[syn::Attribute]) -> bool {
            attrs.iter().any(|a| {
                a.to_token_stream().to_string().contains("doc")
                    && a.to_token_stream().to_string().contains("hidden")
            })
        }
        // Collect documented structs and methods
        use std::collections::HashSet;
        let mut doc_structs: HashSet<String> = HashSet::new();
        let mut doc_methods: HashSet<(String, String)> = HashSet::new();
        fn walk(
            path: &std::path::Path,
            items: Vec<Item>,
            doc_structs: &mut HashSet<String>,
            doc_methods: &mut HashSet<(String, String)>,
        ) {
            for item in items {
                match item {
                    Item::Mod(m) => {
                        if let Visibility::Public(_) = m.vis {
                            let (mod_path, mod_items) = super::codegen::parse_module(path, &m);
                            walk(&mod_path, mod_items, doc_structs, doc_methods);
                        }
                    }
                    Item::Struct(s) => {
                        if let Visibility::Public(_) = s.vis
                            && !is_doc_hidden(&s.attrs)
                            && super::validation::has_lsp_doc(&s.attrs)
                        {
                            doc_structs.insert(s.ident.to_string());
                        }
                    }
                    Item::Impl(item_impl) => {
                        // Only track inherent impls for types
                        if let syn::Type::Path(type_path) = *item_impl.self_ty {
                            let type_name =
                                type_path.path.segments.last().unwrap().ident.to_string();
                            for impl_item in item_impl.items {
                                if let ImplItem::Fn(method) = impl_item
                                    && matches!(method.vis, Visibility::Public(_))
                                    && !is_doc_hidden(&method.attrs)
                                    && super::validation::has_lsp_doc(&method.attrs)
                                {
                                    let name = method.sig.ident.to_string();
                                    doc_methods.insert((type_name.clone(), name));
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        walk(
            entry.0.as_path(),
            entry.1.items,
            &mut doc_structs,
            &mut doc_methods,
        );
        // Enforce coverage
        for o in objects {
            if !doc_structs.contains(o) {
                problems.push(format!(
                    "Missing #[lsp_doc] attribute on public struct {}",
                    o
                ));
            }
        }
        // We need the full API map to know method names; but we can read the exported one back
        let api_map_file = meta::workspace_root().join(super::codegen::API_MAP_FILE);
        let api_map_src = std::fs::read_to_string(&api_map_file).unwrap_or_default();
        // A simple heuristic: look for tuples ("Type", [ObjectProperty { name: "method" ... }])
        for o in objects {
            let marker = format!("(\"{}\", &[", o);
            if let Some(idx) = api_map_src.find(&marker) {
                let rest = &api_map_src[idx + marker.len()..];
                let until = rest.find("]),").unwrap_or(0);
                let slice = &rest[..until];
                for line in slice.lines() {
                    if let Some(pos) = line.find("function: Some(FunctionSignature { name: \"") {
                        let name_start = pos + "function: Some(FunctionSignature { name: \"".len();
                        if let Some(end) = line[name_start..].find("\"") {
                            let name = &line[name_start..name_start + end];
                            if !doc_methods.contains(&(o.clone(), name.to_string())) {
                                problems
                                    .push(format!("Missing #[lsp_doc] on method {}::{}", o, name));
                            }
                        }
                    }
                }
            }
        }
    }

    mod website {
        use super::*;

        fn first_paragraph(md: &str) -> String {
            let mut lines = md.lines();
            // Skip the first H1 line entirely
            for line in lines.by_ref() {
                if line.trim_start().starts_with('#') {
                    break;
                }
            }

            let mut out = String::new();
            let mut started = false;
            for line in lines {
                let t = line.trim();
                // Skip any heading lines
                if t.starts_with('#') {
                    if started && !out.is_empty() {
                        break;
                    } else {
                        continue;
                    }
                }
                // Stop on blank line once we've started
                if t.is_empty() {
                    if started && !out.is_empty() {
                        break;
                    } else {
                        continue;
                    }
                }
                started = true;
                out.push_str(line);
                out.push('\n');
            }
            out.trim().to_string()
        }

        fn escape_mdx_specials_in_heading_text(s: &str) -> String {
            // Escape MDX-reserved characters in headings so MDX doesn't parse them as JSX/expressions.
            // Respect inline code spans (`...`), leaving their contents untouched.
            let mut out = String::new();
            let mut in_tick = false;
            for ch in s.chars() {
                if ch == '`' {
                    in_tick = !in_tick;
                    out.push(ch);
                    continue;
                }
                if in_tick {
                    out.push(ch);
                } else {
                    match ch {
                        '<' => out.push_str("&lt;"),
                        '>' => out.push_str("&gt;"),
                        '{' => out.push_str("&#123;"),
                        '}' => out.push_str("&#125;"),
                        _ => out.push(ch),
                    }
                }
            }
            out
        }

        fn downshift_headings(md: &str) -> String {
            // Shift headings down by two levels so that method H1 becomes H3 under the "## Methods" section.
            let mut out = String::new();
            let mut in_code = false;
            for l in md.lines() {
                let line = l;
                if line.trim_start().starts_with("```") {
                    in_code = !in_code;
                    out.push_str(line);
                    out.push('\n');
                    continue;
                }
                if in_code {
                    out.push_str(line);
                    out.push('\n');
                    continue;
                }
                let mut shifted = if let Some(stripped) = line.strip_prefix("######") {
                    format!("######{}", stripped)
                } else if let Some(stripped) = line.strip_prefix("#####") {
                    format!("######{}", stripped)
                } else if let Some(stripped) = line.strip_prefix("####") {
                    format!("######{}", stripped)
                } else if let Some(stripped) = line.strip_prefix("###") {
                    format!("#####{}", stripped)
                } else if let Some(stripped) = line.strip_prefix("##") {
                    format!("####{}", stripped)
                } else if let Some(stripped) = line.strip_prefix('#') {
                    format!("###{}", stripped)
                } else {
                    line.to_string()
                };
                // Escape MDX-reserved characters in headings (outside code blocks) to avoid parsing errors.
                if shifted.trim_start().starts_with('#') {
                    shifted = escape_mdx_specials_in_heading_text(&shifted);
                }
                out.push_str(&shifted);
                out.push('\n');
            }
            out.trim_end().to_string()
        }

        fn find_object_dir(
            docs_root: &std::path::Path,
            dir_name: &str,
        ) -> Option<std::path::PathBuf> {
            fn walk(root: &std::path::Path, target: &str) -> Option<std::path::PathBuf> {
                if !root.is_dir() {
                    return None;
                }
                for entry in std::fs::read_dir(root).ok()?.flatten() {
                    let p = entry.path();
                    if p.is_dir() {
                        if p.file_name().and_then(|s| s.to_str()) == Some(target) {
                            let md = p.join(format!("{}.md", target));
                            if md.exists() {
                                return Some(p);
                            }
                        }
                        if let Some(found) = walk(&p, target) {
                            return Some(found);
                        }
                    }
                }
                None
            }
            walk(docs_root, dir_name)
        }

        fn category_rel_from(docs_root: &std::path::Path, obj_dir: &std::path::Path) -> String {
            let parent = obj_dir.parent().unwrap_or(docs_root);
            if let Ok(rel) = parent.strip_prefix(docs_root) {
                rel.to_string_lossy().replace('\\', "/")
            } else {
                String::new()
            }
        }

        fn scan_docs_objects(
            docs_root: &std::path::Path,
        ) -> Vec<(String, std::path::PathBuf, String)> {
            fn walk(
                dir: &std::path::Path,
                root: &std::path::Path,
                out: &mut Vec<(String, std::path::PathBuf, String)>,
            ) {
                if !dir.is_dir() {
                    return;
                }
                for entry in std::fs::read_dir(dir).ok().into_iter().flatten().flatten() {
                    let p = entry.path();
                    if p.is_dir() {
                        if let Some(name) = p.file_name().and_then(|s| s.to_str()) {
                            let md = p.join(format!("{}.md", name));
                            if md.exists() {
                                let object = super::validation::dir_to_object_name(name);
                                let cat = category_rel_from(root, &p);
                                out.push((object, p.clone(), cat));
                                // Do not descend into this object dir further
                                continue;
                            }
                        }
                        walk(&p, root, out);
                    }
                }
            }
            let mut out = Vec::new();
            walk(docs_root, docs_root, &mut out);
            out
        }

        fn is_hidden_category(cat_rel: &str) -> bool {
            cat_rel.split('/').any(|seg| seg == "hidden")
        }

        fn category_title(cat_rel: &str) -> String {
            if cat_rel.is_empty() {
                return String::new();
            }
            cat_rel
                .split('/')
                .map(|seg| {
                    let mut chars = seg.chars();
                    match chars.next() {
                        Some(first) => {
                            first.to_uppercase().collect::<String>()
                                + &chars.as_str().to_ascii_lowercase()
                        }
                        None => String::new(),
                    }
                })
                .collect::<Vec<_>>()
                .join(" / ")
        }
        fn strip_leading_h1(md: &str) -> String {
            let mut out = String::new();
            let mut first = true;
            for line in md.lines() {
                if first && line.trim_start().starts_with('#') {
                    first = false;
                    continue; // skip the H1 line entirely
                }
                first = false;
                out.push_str(line);
                out.push('\n');
            }
            out.trim_start().to_string()
        }

        fn strip_after_methods(md: &str) -> String {
            let mut out = String::new();
            for line in md.lines() {
                if line.trim_start().starts_with("## Methods") {
                    break;
                }
                out.push_str(line);
                out.push('\n');
            }
            out.trim_end().to_string()
        }

        /// Post-process MDX content to transform Rust code fences:
        /// - Strip leading `#` from hidden lines (doc-test convention)
        /// - Move top-level `use ...` lines inside the first hidden wrapper block
        /// - Compute collapse ranges for all contiguous hidden runs and annotate the fence
        ///
        /// Only affects ```rust* code fences. All other languages are left unchanged.
        fn transform_rust_code_fences(mdx: &str) -> String {
            #[derive(Clone)]
            struct LineItem {
                text: String,
                hidden: bool,
            }

            fn process_rust_block(header: &str, body: &[String]) -> (String, Vec<String>) {
                // Build items with hidden flags by stripping leading '#'
                let mut items: Vec<LineItem> = Vec::with_capacity(body.len());
                for l in body {
                    let trimmed = l.trim_start();
                    let indent_len = l.len() - trimmed.len();
                    if trimmed.starts_with('#') {
                        // Strip one leading '#' and one optional following space
                        let after = if let Some(stripped) = trimmed.strip_prefix('#') {
                            stripped
                        } else {
                            trimmed
                        };

                        let after = if let Some(stripped) = after.strip_prefix(' ') {
                            stripped
                        } else {
                            after
                        };

                        let new_text = format!("{}{}", &l[..indent_len], after);
                        items.push(LineItem {
                            text: new_text,
                            hidden: true,
                        });
                    } else {
                        items.push(LineItem {
                            text: l.clone(),
                            hidden: false,
                        });
                    }
                }

                // Compute collapse ranges over final items
                let mut ranges: Vec<(usize, usize)> = Vec::new();
                let mut i = 0usize;
                let mut line_no = 1usize;
                while i < items.len() {
                    if items[i].hidden {
                        let start = line_no;
                        while i < items.len() && items[i].hidden {
                            i += 1;
                            line_no += 1;
                        }
                        let end = line_no.saturating_sub(1);
                        if end >= start {
                            ranges.push((start, end));
                        }
                        continue;
                    }
                    i += 1;
                    line_no += 1;
                }

                // Prepare new header with collapse={...}
                let mut new_header = header.to_string();
                if !ranges.is_empty() {
                    let mut collapse = String::new();
                    collapse.push_str("collapse={");
                    for (idx, (s, e)) in ranges.iter().enumerate() {
                        if idx > 0 {
                            collapse.push_str(", ");
                        }
                        // Ranges are already computed as 0-based above.
                        collapse.push_str(&format!("{}-{}", s, e));
                    }
                    collapse.push('}');

                    // Remove any existing collapse={...} chunk in header (best-effort without regex)
                    if let Some(pos) = new_header.find("collapse={")
                        && let Some(end_rel) = new_header[pos..].find('}')
                    {
                        let end = pos + end_rel + 1;
                        new_header.replace_range(pos..end, "");
                    }
                    if !new_header.ends_with(' ') {
                        new_header.push(' ');
                    }
                    new_header.push_str(&collapse);
                }

                let new_body: Vec<String> = items.into_iter().map(|it| it.text).collect();
                (new_header, new_body)
            }

            let mut out = String::new();
            let mut in_code = false;
            let mut header = String::new();
            let mut is_rust = false;
            let mut block: Vec<String> = Vec::new();

            for line in mdx.lines() {
                if !in_code {
                    if line.starts_with("```") {
                        header = line.to_string();
                        let trimmed = header.trim();
                        if trimmed == "```" {
                            // Default unlabeled fences to rust for our docs pages
                            header = "```rust".to_string();
                            is_rust = true;
                        } else {
                            is_rust = header.starts_with("```rust");
                        }
                        in_code = true;
                        block.clear();
                    } else {
                        out.push_str(line);
                        out.push('\n');
                    }
                } else if line.starts_with("```") {
                    // End of block
                    if is_rust {
                        let (new_header, new_body) = process_rust_block(&header, &block);
                        out.push_str(&new_header);
                        out.push('\n');
                        for l in new_body {
                            out.push_str(&l);
                            out.push('\n');
                        }
                        out.push_str("```");
                        out.push('\n');
                    } else {
                        out.push_str(&header);
                        out.push('\n');
                        for l in &block {
                            out.push_str(l);
                            out.push('\n');
                        }
                        out.push_str("```");
                        out.push('\n');
                    }
                    in_code = false;
                    header.clear();
                    is_rust = false;
                    block.clear();
                } else {
                    block.push(line.to_string());
                }
            }

            // If the MDX ended while inside a code block, flush and ensure it is closed
            if in_code {
                out.push_str(&header);
                out.push('\n');
                for l in &block {
                    out.push_str(l);
                    out.push('\n');
                }
                out.push_str("```");
                out.push('\n');
            }

            out
        }

        fn inject_platform_banner_if_needed(object: &str, mdx: &str) -> String {
            fn needs_banner(obj: &str) -> bool {
                obj.starts_with("Android") || obj.starts_with("Ios")
            }
            if !needs_banner(object) {
                return mdx.to_string();
            }
            // Insert after front matter end (---\n\n). If not found, prepend at top.
            let banner_import = "import { Aside } from \"@astrojs/starlight/components\";\n\n";
            let banner = "<Aside type=\"caution\" title=\"Work in progress\">\nThese platform bindings are not yet available; implementation is in the works. APIs may change.\n</Aside>\n\n";
            if let Some(idx) = mdx.find("---\n\n") {
                let (head, tail) = mdx.split_at(idx + 4);
                let mut out = String::with_capacity(mdx.len() + banner.len() + banner_import.len());
                out.push_str(head);
                out.push_str(banner_import);
                out.push_str(banner);
                out.push_str(tail);
                out
            } else {
                let mut out = String::new();
                out.push_str(banner_import);
                out.push_str(banner);
                out.push_str(mdx);
                out
            }
        }

        pub fn update(api_map: &ApiMap) {
            use std::collections::{BTreeMap, HashSet};
            let root = meta::workspace_root();
            let docs_root = root.join("docs/api");
            let site_root = root.join("docs/website/src/content/docs/api");

            // Sync docs/website VersionBadge.astro version with the crate version
            {
                let version =
                    std::env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "0.0.0".to_string());
                let comp_path = root.join("docs/website/src/components/VersionBadge.astro");
                if let Ok(src) = std::fs::read_to_string(&comp_path) {
                    let mut out = String::new();
                    let mut changed = false;
                    for line in src.lines() {
                        let lt = line.trim_start();
                        if lt.starts_with("const VERSION = '") {
                            out.push_str(&format!("const VERSION = '{}';\n", version));
                            changed = true;
                        } else {
                            out.push_str(line);
                            out.push('\n');
                        }
                    }
                    if changed {
                        let _ = std::fs::write(&comp_path, out);
                    }
                }
            }

            // Track expected output files for cleanup; store paths relative to site_root (forward slashes)
            let mut expected: HashSet<String> = HashSet::new();
            // Group objects by category (relative path under docs/api)
            let mut groups: BTreeMap<String, Vec<String>> = BTreeMap::new();
            // Track which objects were processed (to avoid duplicates when scanning extras)
            let mut processed: HashSet<String> = HashSet::new();
            // Collect generated example file paths for platform healthchecks
            let mut ex_js: std::collections::HashSet<String> = std::collections::HashSet::new();
            let mut ex_py: std::collections::HashSet<String> = std::collections::HashSet::new();

            // Build set of known top-level categories for link normalization
            let mut top_categories: HashSet<String> = HashSet::new();
            for (_object, _dir, cat_rel) in scan_docs_objects(&docs_root) {
                if !cat_rel.is_empty()
                    && let Some(first) = cat_rel.split('/').next()
                {
                    top_categories.insert(first.to_string());
                }
            }

            // Escape backticks for MDX template literal usage in <Code code={`...`} />
            fn sanitize_for_template(s: &str) -> String {
                let mut out = String::with_capacity(s.len());
                for ch in s.chars() {
                    match ch {
                        '`' => out.push_str("\\`"),
                        _ => out.push(ch),
                    }
                }
                out
            }

            // Reusable: turn a Rust code body into (text, hidden) items using doc-test '#'
            fn lines_to_items(rust: &str) -> Vec<(String, bool)> {
                let mut items: Vec<(String, bool)> = Vec::new();
                for l in rust.lines() {
                    let trimmed = l.trim_start();
                    let indent_len = l.len() - trimmed.len();
                    if trimmed.starts_with('#') {
                        let after = if let Some(stripped) = trimmed.strip_prefix('#') {
                            stripped
                        } else {
                            trimmed
                        };
                        // Optional single space after '#'
                        let after = if let Some(stripped) = trimmed.strip_prefix(' ') {
                            stripped
                        } else {
                            after
                        };
                        let new_text = format!("{}{}", &l[..indent_len], after);
                        items.push((new_text, true));
                    } else {
                        items.push((l.to_string(), false));
                    }
                }
                items
            }

            // Central helper to build Tabs for an example and persist JS/Py files
            fn build_tabs_for_example(
                items: &[(String, bool)],
                cat_rel: &str,
                obj_dir_slug: &str,
                file_stem: &str,
                root: &std::path::Path,
                ex_js: &mut std::collections::HashSet<String>,
                ex_py: &mut std::collections::HashSet<String>,
            ) -> String {
                // Compute collapse ranges (1-based inclusive) from hidden runs
                let mut ranges: Vec<(usize, usize)> = Vec::new();
                let mut i_ln = 0usize;
                while i_ln < items.len() {
                    if items[i_ln].1 {
                        let start = i_ln + 1;
                        while i_ln < items.len() && items[i_ln].1 {
                            i_ln += 1;
                        }
                        let end = i_ln; // inclusive
                        ranges.push((start, end));
                    } else {
                        i_ln += 1;
                    }
                }
                let rust_lines: Vec<String> = items.iter().map(|(t, _)| t.clone()).collect();
                let rust_processed = rust_lines.join("\n");
                let meta_attr = if ranges.is_empty() {
                    String::new()
                } else {
                    let mut s = String::from(" meta=\"collapse={");
                    for (idx, (s1, e1)) in ranges.iter().enumerate() {
                        if idx > 0 {
                            s.push_str(", ");
                        }
                        s.push_str(&format!("{}-{}", s1, e1));
                    }
                    s.push_str("}\"");
                    s
                };

                // Convert to JS/Python
                let js_code = crate::convert::to_js(items);
                let py_code = crate::convert::to_py(items);

                // Persist example files for healthchecks
                let js_rel = if cat_rel.is_empty() {
                    format!("{}/{}.js", obj_dir_slug, file_stem)
                } else {
                    format!("{}/{}/{}.js", cat_rel, obj_dir_slug, file_stem)
                };
                let py_rel = if cat_rel.is_empty() {
                    format!("{}/{}.py", obj_dir_slug, file_stem)
                } else {
                    format!("{}/{}/{}.py", cat_rel, obj_dir_slug, file_stem)
                };
                let js_abs = root.join("platforms/web/examples").join(&js_rel);
                let py_abs = root.join("platforms/python/examples").join(&py_rel);
                if let Some(p) = js_abs.parent() {
                    let _ = std::fs::create_dir_all(p);
                }
                if let Some(p) = py_abs.parent() {
                    let _ = std::fs::create_dir_all(p);
                }
                let _ = std::fs::write(&js_abs, &js_code);
                let _ = std::fs::write(&py_abs, &py_code);
                ex_js.insert(js_rel.clone());
                ex_py.insert(py_rel.clone());

                // Swift/Kotlin placeholders
                let sk_rel = if cat_rel.is_empty() {
                    format!("{}/{}.txt", obj_dir_slug, file_stem)
                } else {
                    format!("{}/{}/{}.txt", cat_rel, obj_dir_slug, file_stem)
                };
                let swift_abs = root.join("platforms/swift/examples").join(&sk_rel);
                let kotlin_abs = root.join("platforms/kotlin/examples").join(&sk_rel);
                if let Some(parent) = swift_abs.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                if let Some(parent) = kotlin_abs.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                if !swift_abs.exists() {
                    let _ = std::fs::write(&swift_abs, "// Swift placeholder — bindings WIP\n");
                }
                if !kotlin_abs.exists() {
                    let _ = std::fs::write(&kotlin_abs, "// Kotlin placeholder — bindings WIP\n");
                }
                let swift_code = std::fs::read_to_string(&swift_abs)
                    .unwrap_or_else(|_| "// Swift placeholder — bindings WIP\n".to_string());
                let kotlin_code = std::fs::read_to_string(&kotlin_abs)
                    .unwrap_or_else(|_| "// Kotlin placeholder — bindings WIP\n".to_string());

                // Build Tabs snippet
                let mut tabs = String::new();
                tabs.push_str("<Tabs>\n\n");

                tabs.push_str("<TabItem label=\"Rust\">\n");
                tabs.push_str("<Code\n");
                tabs.push_str("code={`\n");
                tabs.push_str(&rust_processed);
                tabs.push_str("\n`}\n");
                tabs.push_str("lang=\"rust\"");
                tabs.push_str(&meta_attr);
                tabs.push_str("\n/>\n\n</TabItem>\n\n");

                tabs.push_str("<TabItem label=\"JavaScript\">\n");
                tabs.push_str("<Code\n");
                tabs.push_str("code={`\n");
                tabs.push_str(&sanitize_for_template(&js_code));
                tabs.push_str("\n`}\nlang=\"js\"\n/>\n\n</TabItem>\n\n");

                tabs.push_str("<TabItem label=\"Python\">\n");
                tabs.push_str("<Code\n");
                tabs.push_str("code={`\n");
                tabs.push_str(&sanitize_for_template(&py_code));
                tabs.push_str("\n`}\nlang=\"python\"\n/>\n\n</TabItem>\n\n");

                tabs.push_str("<TabItem label=\"Swift\">\n");
                tabs.push_str("<Aside type=\"caution\" title=\"Work in progress\">Swift bindings are not yet available; implementation is in the works.</Aside>\n");
                tabs.push_str("<Code\n");
                tabs.push_str("code={`\n");
                tabs.push_str(&swift_code);
                tabs.push_str("\n`}\nlang=\"text\"\n/>\n\n</TabItem>\n\n");

                tabs.push_str("<TabItem label=\"Kotlin\">\n");
                tabs.push_str("<Aside type=\"caution\" title=\"Work in progress\">Kotlin/Android bindings are not yet available; implementation is in the works.</Aside>\n");
                tabs.push_str("<Code\n");
                tabs.push_str("code={`\n");
                tabs.push_str(&kotlin_code);
                tabs.push_str("\n`}\nlang=\"text\"\n/>\n\n</TabItem>\n\n");

                tabs.push_str("</Tabs>\n\n");
                tabs
            }

            // Helper to write a page given an object, its docs dir, category relative path, and ordered method files
            let mut write_page = |object: &str,
                                  obj_dir: &std::path::Path,
                                  cat_rel: &str,
                                  method_files: Vec<String>|
             -> String {
                let obj_dirname = obj_dir.file_name().and_then(|s| s.to_str()).unwrap_or("");
                let obj_md = std::fs::read_to_string(obj_dir.join(format!("{}.md", obj_dirname)))
                    .unwrap_or_default();
                let description = first_paragraph(&obj_md);
                let body = strip_after_methods(&strip_leading_h1(&obj_md));

                let mut out = String::new();
                out.push_str("---\n");
                out.push_str(&format!("title: {}\n", object));
                let desc = description.replace('\n', " ").replace('"', "\\\"");
                out.push_str(&format!("description: \"{}\"\n", desc));
                if !cat_rel.is_empty() {
                    out.push_str(&format!("category: {}\n", cat_rel));
                    out.push_str(&format!("categoryLabel: {}\n", category_title(cat_rel)));
                }
                out.push_str("---\n\n");

                // Tabs/Code components for examples
                out.push_str("import { Code, Tabs, TabItem, Aside } from \"@astrojs/starlight/components\";\n\n");

                out.push_str("## Description\n\n");

                // Extract and remove the main "## Example" section from the object body (pre-Methods)
                let mut desc_without = String::new();
                let mut main_rust = String::new();
                let mut in_example = false;
                let mut in_code = false;
                let mut capture_rust = false;
                let mut captured_any = false;
                for line in body.lines() {
                    let t = line.trim_start();
                    if !in_example {
                        if t.starts_with("## Example") {
                            in_example = true;
                            continue; // drop header
                        }
                        desc_without.push_str(line);
                        desc_without.push('\n');
                        continue;
                    }
                    // in_example section: skip everything until next top-level subsection or EOF
                    if !in_code && t.starts_with("## ") {
                        // End of example section; resume copying
                        in_example = false;
                        desc_without.push_str(line);
                        desc_without.push('\n');
                        continue;
                    }
                    if !in_code && t.starts_with("```") {
                        in_code = true;
                        let is_unlabeled = t == "```";
                        let is_rust = t.starts_with("```rust") || is_unlabeled;
                        capture_rust = is_rust && !captured_any;
                        continue; // do not copy fences
                    }
                    if in_code && t.starts_with("```") {
                        in_code = false;
                        if capture_rust {
                            captured_any = true;
                            capture_rust = false;
                        }
                        continue; // do not copy fences
                    }
                    if in_code {
                        if capture_rust {
                            main_rust.push_str(line);
                            main_rust.push('\n');
                        }
                        continue; // do not copy other code content into desc
                    }
                    // non-code content within example section is dropped from desc
                }

                out.push_str(&desc_without);
                out.push('\n');

                if captured_any && !main_rust.trim().is_empty() {
                    // Use the struct name as file stem per user preference
                    let dir_slug = obj_dirname;
                    let tabs = build_tabs_for_example(
                        &lines_to_items(&main_rust),
                        cat_rel,
                        dir_slug,
                        object,
                        &root,
                        &mut ex_js,
                        &mut ex_py,
                    );
                    out.push_str("\n## Example\n\n");
                    out.push_str(&tabs);
                }

                out.push_str("\n## Methods\n\n");
                for file in method_files.iter() {
                    let md = std::fs::read_to_string(obj_dir.join(format!("{}.md", file)))
                        .unwrap_or_default();

                    // Split method content into description (pre) and the first Rust code block after '## Example'
                    let mut pre = String::new();
                    let mut rust_body = String::new();
                    let mut after_example = false;
                    let mut in_code = false;
                    let mut taking_rust = false;
                    for line in md.lines() {
                        let t = line.trim_start();
                        if !after_example && t.starts_with("## Example") {
                            after_example = true;
                            continue;
                        }
                        if after_example {
                            if !in_code && t.starts_with("```") {
                                in_code = true;
                                taking_rust = true; // assume the first block is Rust
                                continue;
                            } else if in_code && t.starts_with("```") {
                                in_code = false;
                                taking_rust = false;
                                continue;
                            }
                            if taking_rust {
                                rust_body.push_str(line);
                                rust_body.push('\n');
                            }
                        } else {
                            pre.push_str(line);
                            pre.push('\n');
                        }
                    }

                    // Separator before each method title for visual grouping
                    out.push_str("\n---\n\n");
                    // Downshift headings for the method description
                    out.push_str(&downshift_headings(pre.trim_end()));
                    out.push('\n');

                    // Build example tabs via shared helper
                    let dir_slug = obj_dir.file_name().and_then(|s| s.to_str()).unwrap_or("");
                    let items = lines_to_items(&rust_body);
                    let tabs = build_tabs_for_example(
                        &items, cat_rel, dir_slug, file, &root, &mut ex_js, &mut ex_py,
                    );
                    out.push_str("\n#### Example\n\n");
                    out.push_str(&tabs);
                }

                // Ensure exactly one trailing newline at EOF
                let mut out = out.trim_end().to_string();
                out.push('\n');

                // Normalize links in MDX to site root
                fn site_base() -> String {
                    std::env::var("DOCS_SITE_BASE").unwrap_or_else(|_| "/api".to_string())
                }

                fn rewrite_links_to_site_root(
                    mdx: &str,
                    top_categories: &std::collections::HashSet<String>,
                ) -> String {
                    let base = site_base();
                    let mut out = String::new();
                    let bytes = mdx.as_bytes();
                    let mut i = 0usize;

                    let n_https = "](https://fragmentcolor.org/api/".as_bytes();
                    let n_http = "](http://fragmentcolor.org/api/".as_bytes();
                    let n_www_https = "](https://www.fragmentcolor.org/api/".as_bytes();
                    let n_www_http = "](http://www.fragmentcolor.org/api/".as_bytes();

                    while i < bytes.len() {
                        let mut matched = None::<&[u8]>;
                        if i + n_https.len() <= bytes.len()
                            && &bytes[i..i + n_https.len()] == n_https
                        {
                            matched = Some(n_https);
                        } else if i + n_http.len() <= bytes.len()
                            && &bytes[i..i + n_http.len()] == n_http
                        {
                            matched = Some(n_http);
                        } else if i + n_www_https.len() <= bytes.len()
                            && &bytes[i..i + n_www_https.len()] == n_www_https
                        {
                            matched = Some(n_www_https);
                        } else if i + n_www_http.len() <= bytes.len()
                            && &bytes[i..i + n_www_http.len()] == n_www_http
                        {
                            matched = Some(n_www_http);
                        }

                        if let Some(m) = matched {
                            out.push_str("](");
                            i += m.len();
                            out.push_str(&base);
                            out.push('/');
                            while i < bytes.len() && bytes[i] != b')' {
                                out.push(bytes[i] as char);
                                i += 1;
                            }
                        } else if i + 2 <= bytes.len() && bytes[i] == b']' && bytes[i + 1] == b'(' {
                            out.push(']');
                            out.push('(');
                            i += 2;
                            let start = i;
                            while i < bytes.len() && bytes[i] != b')' {
                                i += 1;
                            }
                            let href = &mdx[start..i];
                            let href_trim = href.trim();
                            let lower = href_trim.to_ascii_lowercase();
                            let is_abs = lower.starts_with("http://")
                                || lower.starts_with("https://")
                                || href_trim.starts_with('/')
                                || href_trim.starts_with('#')
                                || lower.starts_with("mailto:")
                                || lower.starts_with("tel:")
                                || lower.starts_with("data:");
                            if is_abs {
                                out.push_str(href_trim);
                            } else {
                                let top = href_trim.split('/').next().unwrap_or("");
                                if top_categories.contains(top) {
                                    out.push_str(&base);
                                    if !href_trim.starts_with('/') {
                                        out.push('/');
                                    }
                                    out.push_str(href_trim);
                                } else {
                                    out.push_str(href_trim);
                                }
                            }
                        } else {
                            out.push(bytes[i] as char);
                            i += 1;
                        }
                    }
                    out
                }

                out = rewrite_links_to_site_root(&out, &top_categories);

                // Add platform WIP banner for Android/iOS pages
                out = inject_platform_banner_if_needed(object, &out);

                // Transform Rust code fences: strip hidden '#', move use-lines, add collapse ranges
                out = transform_rust_code_fences(&out);

                let site_file = if cat_rel.is_empty() {
                    site_root.join(format!("{}.mdx", object.to_lowercase()))
                } else {
                    site_root
                        .join(cat_rel)
                        .join(format!("{}.mdx", object.to_lowercase()))
                };
                if let Some(parent) = site_file.parent() {
                    std::fs::create_dir_all(parent).unwrap();
                }
                std::fs::write(&site_file, out).unwrap();

                // Return relative path for cleanup
                if cat_rel.is_empty() {
                    format!("{}.mdx", object.to_lowercase())
                } else {
                    format!("{}/{}.mdx", cat_rel, object.to_lowercase())
                }
            };

            // Iterate objects discovered from AST (base objects only)
            let objects = super::validation::base_public_objects();
            for object in objects.iter() {
                let dir_name = super::validation::object_dir_name(object);
                let obj_dir =
                    find_object_dir(&docs_root, &dir_name).unwrap_or(docs_root.join(&dir_name));
                let cat_rel = if obj_dir.exists() {
                    category_rel_from(&docs_root, &obj_dir)
                } else {
                    String::new()
                };

                // Determine method files from codegen
                let mut method_files = Vec::new();
                if let Some(methods) = api_map.get(object) {
                    for m in methods {
                        if let Some(fun) = &m.function {
                            let name = &fun.name;
                            let file = name.clone();
                            if obj_dir.join(format!("{}.md", file)).exists() {
                                method_files.push(file);
                            }
                        }
                    }
                }

                // If no methods were discovered, fall back to docs files in the folder
                if method_files.is_empty()
                    && obj_dir.exists()
                    && obj_dir.is_dir()
                    && let Ok(read_dir) = std::fs::read_dir(&obj_dir)
                {
                    let mut files: Vec<String> = read_dir
                        .filter_map(|e| e.ok())
                        .filter_map(|e| {
                            let p = e.path();
                            if p.extension()?.to_str()? == "md" {
                                let stem = p.file_stem()?.to_str()?.to_string();
                                if stem != dir_name { Some(stem) } else { None }
                            } else {
                                None
                            }
                        })
                        .collect();
                    files.sort();
                    method_files = files;
                }

                if is_hidden_category(&cat_rel) {
                    processed.insert(object.clone());
                    continue;
                }
                let rel = write_page(object, &obj_dir, &cat_rel, method_files);
                expected.insert(rel);
                groups.entry(cat_rel).or_default().push(object.clone());
                processed.insert(object.clone());
            }

            // Extras: any docs-only objects in docs/api (recursively) not already processed
            for (object, obj_dir, cat_rel) in scan_docs_objects(&docs_root) {
                if processed.contains(&object) {
                    continue;
                }
                let dir_name = obj_dir
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_string();
                let mut method_files: Vec<String> = Vec::new();
                if let Ok(files) = std::fs::read_dir(&obj_dir) {
                    method_files = files
                        .filter_map(|e| e.ok())
                        .filter_map(|e| {
                            let p = e.path();
                            if p.extension()?.to_str()? == "md" {
                                let stem = p.file_stem()?.to_str()?.to_string();
                                if stem != dir_name { Some(stem) } else { None }
                            } else {
                                None
                            }
                        })
                        .collect();
                    method_files.sort();
                }
                if is_hidden_category(&cat_rel) {
                    processed.insert(object);
                    continue;
                }
                let rel = write_page(&object, &obj_dir, &cat_rel, method_files);
                expected.insert(rel);
                groups.entry(cat_rel).or_default().push(object.clone());
                processed.insert(object);
            }

            // Generate an index.mdx grouped by category relative path (as in source)
            // groups: cat_rel -> [objects]
            for list in groups.values_mut() {
                list.sort();
            }

            let mut top = groups.remove("").unwrap_or_default();
            top.sort();

            let mut cats: Vec<String> = groups.keys().cloned().collect();
            cats.sort();

            let mut idx = String::new();
            idx.push_str("---\n");
            idx.push_str("title: API\n");
            idx.push_str("description: \"Auto-generated API index\"\n");
            idx.push_str("---\n\n");
            idx.push_str("# API\n\n");

            // Build site base for index links
            let base = std::env::var("DOCS_SITE_BASE").unwrap_or_else(|_| "/api".to_string());
            let base = base.trim_end_matches('/');

            // Top-level (no category) first
            for o in &top {
                let path = format!("{}/{}", base, o.to_lowercase());
                idx.push_str(&format!("- [{}]({})\n", o, path));
            }
            if !top.is_empty() {
                idx.push('\n');
            }

            // Then each category alphabetically
            for cat in cats {
                if let Some(list) = groups.get(&cat) {
                    let label = category_title(&cat);
                    idx.push_str(&format!("## {}\n\n", label));
                    for o in list {
                        let path = format!("{}/{}/{}", base, cat, o.to_lowercase());
                        idx.push_str(&format!("- [{}]({})\n", o, path));
                    }
                    idx.push('\n');
                }
            }

            let mut idx = idx.trim_end().to_string();
            idx.push('\n');
            std::fs::write(site_root.join("index.mdx"), idx).unwrap();

            // Cleanup: remove any stale MDX files not written in this run
            expected.insert("index.mdx".to_string());
            fn walk_and_cleanup(
                root: &std::path::Path,
                base: &std::path::Path,
                expected: &std::collections::HashSet<String>,
            ) {
                if let Ok(read_dir) = std::fs::read_dir(root) {
                    for entry in read_dir.flatten() {
                        let path = entry.path();
                        if path.is_dir() {
                            walk_and_cleanup(&path, base, expected);
                            continue;
                        }
                        if let Some(ext) = path.extension().and_then(|s| s.to_str())
                            && ext == "mdx"
                            && let Ok(rel) = path.strip_prefix(base)
                        {
                            let key = rel.to_string_lossy().replace('\\', "/");
                            if !expected.contains(&key) {
                                let _ = std::fs::remove_file(&path);
                            }
                        }
                    }
                }
            }
            walk_and_cleanup(&site_root, &site_root, &expected);

            // Write platform healthcheck aggregators to include ALL generated examples
            {
                // Sort and write JS aggregator
                let mut js_list: Vec<String> = ex_js.into_iter().collect();
                js_list.sort();
                let js_path = root.join("platforms/web/healthcheck/generated_examples.mjs");
                if let Some(parent) = js_path.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                let mut js_out = String::new();
                js_out
                    .push_str("// Auto-generated: runs all JS examples with cargo-like output.\n");
                js_out.push_str("const GREEN='\\u001b[1;32m'; const RED='\\u001b[1;31m'; const RESET='\\u001b[0m';\n");
                js_out.push_str("const EXAMPLES = [\n");
                for rel in &js_list {
                    js_out.push_str(&format!("  '../examples/{}',\n", rel));
                }
                js_out.push_str("]\n\n");
                js_out.push_str("function fq(rel){ return 'platforms.web.examples.' + rel.replace('../examples/','').replace(/\\\\.js$/, '').replaceAll('/', '.'); }\n");
                js_out.push_str("(async () => {\n  let failed = 0;\n  for (const rel of EXAMPLES) {\n    const name = fq(rel);\n    const head = `test ${name} ... `;\n    try {\n      await import(rel);\n      console.log(head + GREEN + 'OK' + RESET);\n    } catch (e) {\n      failed++;\n      console.log(head + RED + 'FAILED' + RESET);\n      console.error(e);\n    }\n  }\n  if (failed === 0) {\n    console.log('Headless JS render completed successfully');\n  } else {\n    throw new Error(`${failed} JS examples failed`);\n  }\n})();\n");
                std::fs::write(&js_path, js_out).unwrap();

                // Sort and write Python aggregator
                let mut py_list: Vec<String> = ex_py.into_iter().collect();
                py_list.sort();
                let py_path = root.join("platforms/python/examples/main.py");
                if let Some(parent) = py_path.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                let mut py_out = String::new();
                py_out.push_str(
                    "# Auto-generated: executes all Python examples with cargo-like output.\n",
                );
                py_out.push_str("import runpy, pathlib, sys, traceback\n\n");
                py_out.push_str("GREEN='\x1b[1;32m'\nRED='\x1b[1;31m'\nRESET='\x1b[0m'\n\n");
                py_out.push_str("def run_all():\n");
                py_out.push_str("    base = pathlib.Path(__file__).parent\n");
                py_out.push_str("    files = [\n");
                for rel in &py_list {
                    let rel_norm = rel.replace('\\', "/");
                    py_out.push_str(&format!("        '{}',\n", rel_norm));
                }
                py_out.push_str("    ]\n");
                py_out.push_str("    failed = 0\n");
                py_out.push_str("    for rel in files:\n");
                py_out.push_str("        name = 'platforms.python.examples.' + rel.replace('/', '.').removesuffix('.py')\n");
                py_out.push_str("        head = f'test {name} ... '\n");
                py_out.push_str("        try:\n");
                py_out
                    .push_str("            runpy.run_path(str(base / rel), run_name='__main__')\n");
                py_out.push_str("            print(head + GREEN + 'OK' + RESET)\n");
                py_out.push_str("        except Exception:\n");
                py_out.push_str("            failed += 1\n");
                py_out.push_str("            print(head + RED + 'FAILED' + RESET)\n");
                py_out.push_str("            traceback.print_exc()\n");
                py_out.push_str("    if failed:\n");
                py_out.push_str("        raise SystemExit(1)\n");
                py_out.push_str("\nif __name__ == '__main__':\n    run_all()\n");
                std::fs::write(&py_path, py_out).unwrap();
            }
        }
    }
}

mod meta {
    use std::{
        path::{Path, PathBuf},
        process::Command,
    };

    pub fn workspace_root() -> PathBuf {
        let output = Command::new(env!("CARGO"))
            .arg("locate-project")
            .arg("--workspace")
            .arg("--message-format=plain")
            .output()
            .unwrap()
            .stdout;
        let cargo_path = Path::new(std::str::from_utf8(&output).unwrap().trim());
        cargo_path.parent().unwrap().to_path_buf()
    }
}
