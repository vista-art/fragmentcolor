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

    /// Export a newline-delimited list of public API objects relevant for JS branding.
    ///
    /// Note: we currently include only types that are wasm-bindgen-exported, because
    /// those are the ones that actually exist as JS classes at runtime. The filename
    /// is intentionally platform-agnostic to allow future reuse.
    pub fn export_api_objects() {
        let root = super::meta::workspace_root();
        let out_path = root.join("generated/api_objects.txt");
        let info = super::validation::collect_public_structs_info();
        let mut names: Vec<String> = Vec::new();
        // Helper: detect #[wasm_bindgen] directly or nested via cfg_attr(..., wasm_bindgen, ...)
        fn has_wasm_bindgen(attrs: &[syn::Attribute]) -> bool {
            attrs.iter().any(|a| {
                if a.path().is_ident("wasm_bindgen") {
                    return true;
                }
                let mut found = false;
                let _ = a.parse_nested_meta(|meta| {
                    if meta.path.is_ident("wasm_bindgen") {
                        found = true;
                    }
                    Ok(())
                });
                found
            })
        }

        for (name, attrs, _inner_types) in info {
            if has_wasm_bindgen(&attrs) {
                names.push(name);
            }
        }
        names.sort();
        names.dedup();
        // Write one per line
        let mut buf = String::new();
        for n in names {
            buf.push_str(&n);
            buf.push('\n');
        }
        if let Some(parent) = out_path.parent()
            && let Err(e) = std::fs::create_dir_all(parent) {
                eprintln!("Warning: Failed to create directory '{}': {}", parent.display(), e);
            }
        if let Err(e) = std::fs::write(&out_path, &buf) {
            eprintln!("Warning: Failed to write to '{}': {}", out_path.display(), e);
        }
        println!("cargo::rerun-if-changed={}", out_path.to_string_lossy());
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
            module_path.clone(),
            match parse_file(&content) {
                Ok(file) => file,
                Err(e) => {
                    eprintln!("Parse error in {}: {}", module_path.display(), e);
                    panic!("Failed to parse module file: {}", module_path.display())
                }
            }
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
