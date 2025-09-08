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

    map_public_api();
}

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
pub const OBJECT_PROPERTY_STRUCT_NAME: &str = "ObjectProperty";
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

pub fn map_public_api() {
    println!();
    println!("🗺️ Generating API map...");

    generate_api_map();

    println!("✅ API map successfully generated!");
    println!();
}

fn generate_api_map() {
    let crate_root = meta::workspace_root();
    let api_map_file = meta::workspace_root().join(API_MAP_FILE);

    // Extract functions from source
    let mut api_map = extract_public_functions(crate_root.as_ref());

    // Derive canonical public objects from AST: all top-level pub structs excluding #[doc(hidden)]
    let objects = validation::public_structs_excluding_hidden();

    // Keep only objects discovered in code (exclude file-key entries and hidden/internal types)
    api_map.retain(|k, _| objects.contains(k));

    // Validate docs, enforce lsp_doc coverage, and update website
    validation::validate_and_update_website(&api_map);

    export_api_map(api_map, api_map_file.as_ref())
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
fn parse_lib_entry_point(file_path: &Path) -> (PathBuf, syn::File) {
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
                        traverse_and_extract(&mod_path, mod_items, signatures, name_filter.clone());
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
fn parse_module(current_path: &Path, current_module: &syn::ItemMod) -> (PathBuf, Vec<Item>) {
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
fn export_api_map(api_map: ApiMap, target_file: &Path) {
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

mod validation {
    use super::*;
    use std::fs;

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
        }
        if !content.contains("## Example") {
            problems.push(format!("{}: '## Example' section missing", path.display()));
        }
        if content.contains("fragmentcolor.com") {
            problems.push(format!("{}: contains fragmentcolor.com", path.display()));
        }
    }

    fn healthcheck_has_block(lang: &str, key: &str, content: &str) -> bool {
        let begin = format!(
            "{} DOC: {} (begin)",
            match lang {
                "py" => "#",
                _ => "//",
            },
            key
        );
        let end = format!(
            "{} DOC: (end)",
            match lang {
                "py" => "#",
                _ => "//",
            }
        );
        content.contains(&begin) && content.contains(&end)
    }

    fn validate_healthchecks(_api_map: &ApiMap, problems: &mut Vec<String>) {
        let py_path = meta::workspace_root().join("platforms/python/healthcheck.py");
        let js_path = meta::workspace_root().join("platforms/web/healthcheck/main.js");
        let py = fs::read_to_string(&py_path).unwrap_or_default();
        let js = fs::read_to_string(&js_path).unwrap_or_default();

        // Only enforce healthcheck markers for a curated subset to keep the healthchecks practical
        let required_keys = vec![
            "Renderer.constructor",
            "Renderer.create_texture_target",
            "Renderer.render",
            "Pass.constructor",
            "Pass.add_shader",
            "Frame.constructor",
            "Frame.add_pass",
            "Shader.constructor",
            "Shader.set",
            "Shader.get",
            "Shader.list_uniforms",
            "Shader.list_keys",
        ];

        for key in required_keys {
            if !healthcheck_has_block("py", key, &py) {
                problems.push(format!(
                    "Missing Python DOC block: {} in {}",
                    key,
                    py_path.display()
                ));
            }
            if !healthcheck_has_block("js", key, &js) {
                problems.push(format!(
                    "Missing JS DOC block: {} in {}",
                    key,
                    js_path.display()
                ));
            }
        }
    }

    pub fn validate_and_update_website(api_map: &ApiMap) {
        let mut problems = Vec::new();
        let root = meta::workspace_root();
        let docs_root = root.join("docs/api");

        // Enforce documentation for ALL public objects (including wrappers)
        let objects = public_structs_excluding_hidden();
        let all_objects = objects.clone();

        // Validate objects and their methods
        for object in objects.iter() {
            let methods_vec = api_map.get(object).cloned().unwrap_or_default();
            let dir = object_dir_name(object);
            let object_md = docs_root.join(&dir).join(format!("{}.md", dir));
            ensure_object_md_ok(object, &object_md, &mut problems);
            enforce_links_in_file(&object_md, &all_objects, &mut problems);

            for m in &methods_vec {
                if let Some(fun) = &m.function {
                    let name = &fun.name;
                    // Skip platform-specific wrapper variants and internal helpers
                    let skip = name.ends_with("_js")
                        || name.ends_with("_py")
                        || name.ends_with("_ios")
                        || name.ends_with("_android")
                        || name == "headless"
                        || name == "render_bitmap"
                        || (object == "TextureTarget" && name == "new");
                    if skip {
                        continue;
                    }

                    let file = if name == "new" {
                        "constructor".to_string()
                    } else {
                        to_snake_case(name)
                    };
                    let path = docs_root.join(&dir).join(format!("{}.md", file));
                    ensure_method_md_ok(object, name, &path, &mut problems);
                    enforce_links_in_file(&path, &all_objects, &mut problems);
                }
            }
        }

        // Also validate any docs-only objects under docs/api not present in allowed
        let docs_root = root.join("docs/api");
        if let Ok(read_dir) = std::fs::read_dir(&docs_root) {
            for entry in read_dir.flatten() {
                if entry.path().is_dir() {
                    let dir_name = entry.file_name().to_string_lossy().to_string();
                    let object = dir_to_object_name(&dir_name);
                    if objects.iter().any(|o| o == &object) {
                        continue;
                    }
                    let object_md = docs_root.join(&dir_name).join(format!("{}.md", dir_name));
                    ensure_object_md_ok(&object, &object_md, &mut problems);
                    enforce_links_in_file(&object_md, &all_objects, &mut problems);
                }
            }
        }

        validate_healthchecks(api_map, &mut problems);

        if !problems.is_empty() {
            eprintln!("\nDocumentation validation failed with the following issues:\n");
            for p in &problems {
                eprintln!("- {}", p);
            }
            panic!("documentation incomplete");
        }

        // Enforce that all public structs and their public methods have #[lsp_doc]
        enforce_lsp_doc_coverage(&objects, &mut problems);

        // Update website if everything is valid
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
        let entry = super::parse_lib_entry_point(&meta::workspace_root());
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
                            let (mod_path, mod_items) = super::parse_module(path, &m);
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
        let entry = super::parse_lib_entry_point(&meta::workspace_root());
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
                            let (mod_path, mod_items) = super::parse_module(path, &m);
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

    fn object_url(name: &str) -> String {
        format!("https://fragmentcolor.org/api/{}", object_dir_name(name))
    }

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
        let entry = super::parse_lib_entry_point(&meta::workspace_root());
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
                            let (mod_path, mod_items) = super::parse_module(path, &m);
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
        let api_map_file = meta::workspace_root().join(super::API_MAP_FILE);
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
                            // Skip platform wrapper variants
                            if name.ends_with("_js")
                                || name.ends_with("_py")
                                || name.ends_with("_ios")
                                || name.ends_with("_android")
                                || name == "headless"
                                || name == "render_bitmap"
                            {
                                continue;
                            }
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

        fn escape_angle_in_heading_text(s: &str) -> String {
            // Escape raw angle brackets in headings so MDX doesn't parse them as JSX tags.
            // We keep inline code spans (`...`) intact, but escape < and > elsewhere.
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
                // Escape < and > in headings only (outside code blocks) to avoid MDX parsing errors.
                if shifted.trim_start().starts_with('#') {
                    shifted = escape_angle_in_heading_text(&shifted);
                }
                out.push_str(&shifted);
                out.push('\n');
            }
            out.trim_end().to_string()
        }

        fn collect_health_example(lang: &str, key: &str, content: &str) -> Option<String> {
            let (start_token, end_token) = match lang {
                "py" => ("#", "#"),
                _ => ("//", "//"),
            };
            let begin = format!("{} DOC: {} (begin)", start_token, key);
            let end = format!("{} DOC: (end)", end_token);
            if let Some(b) = content.find(&begin) {
                let from = b + begin.len();
                if let Some(e_rel) = content[from..].find(&end) {
                    let e = from + e_rel;
                    let snippet = &content[from..e];
                    return Some(snippet.trim().to_string());
                }
            }
            None
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

        pub fn update(api_map: &ApiMap) {
            use std::collections::HashSet;
            let root = meta::workspace_root();
            let docs_root = root.join("docs/api");
            let site_root = root.join("docs/website/src/content/docs/api");
            let py = std::fs::read_to_string(root.join("platforms/python/healthcheck.py"))
                .unwrap_or_default();
            let js = std::fs::read_to_string(root.join("platforms/web/healthcheck/main.js"))
                .unwrap_or_default();

            // Track which objects were written via reflection
            let mut written: HashSet<String> = HashSet::new();

            // Helper to write a page given an object name and an ordered list of method files (without extension)
            let write_page = |object: &str, method_files: Vec<String>| {
                let dir = super::validation::object_dir_name(object);
                let obj_dir = docs_root.join(&dir);
                let obj_md = std::fs::read_to_string(obj_dir.join(format!("{}.md", dir)))
                    .unwrap_or_default();
                let description = first_paragraph(&obj_md);
                let body = strip_after_methods(&strip_leading_h1(&obj_md));

                let mut out = String::new();
                out.push_str("---\n");
                out.push_str(&format!("title: {}\n", object));
                let desc = description.replace('\n', " ").replace('"', "\\\"");
                out.push_str(&format!("description: \"{}\"\n", desc));
                out.push_str("---\n\n");

                out.push_str("## Description\n\n");
                out.push_str(&body);
                out.push('\n');

                out.push_str("\n## Methods\n\n");
                for file in method_files {
                    let md = std::fs::read_to_string(obj_dir.join(format!("{}.md", file)))
                        .unwrap_or_default();
                    out.push_str(&downshift_headings(&md));
                    out.push('\n');

                    // Examples: add Python and JS blocks if present
                    let key = if file == "constructor" {
                        format!("{}.constructor", object)
                    } else {
                        format!("{}.{}", object, file)
                    };
                    if let Some(py_ex) = collect_health_example("py", &key, &py) {
                        out.push_str("\n### Python\n\n```python\n");
                        out.push_str(&py_ex);
                        out.push_str("\n```\n");
                    }
                    if let Some(js_ex) = collect_health_example("js", &key, &js) {
                        out.push_str("\n### Javascript\n\n```js\n");
                        out.push_str(&js_ex);
                        out.push_str("\n```\n");
                    }

                    // Ensure a blank line after each method section for spacing
                    out.push('\n');
                }

                // Ensure exactly one trailing newline at EOF
                let mut out = out.trim_end().to_string();
                out.push('\n');
                let site_file = site_root.join(format!("{}.mdx", object.to_lowercase()));
                std::fs::write(site_file, out).unwrap();
            };

            // Iterate objects discovered from AST (base objects only)
            let objects = super::validation::base_public_objects();
            for object in objects.iter() {
                let dir = super::validation::object_dir_name(object);
                let obj_dir = docs_root.join(&dir);

                // Determine method files from reflection, skipping platform variants
                let mut method_files = Vec::new();
                if let Some(methods) = api_map.get(object) {
                    for m in methods {
                        if let Some(fun) = &m.function {
                            let name = &fun.name;
                            let skip = name.ends_with("_js")
                                || name.ends_with("_py")
                                || name.ends_with("_ios")
                                || name.ends_with("_android")
                                || name == "headless"
                                || name == "render_bitmap";
                            if skip {
                                continue;
                            }
                            let file = if name == "new" {
                                "constructor".to_string()
                            } else {
                                name.clone()
                            };
                            if obj_dir.join(format!("{}.md", file)).exists() {
                                method_files.push(file);
                            }
                        }
                    }
                }

                // If no methods were discovered, fall back to docs files in the folder
                if method_files.is_empty()
                    && let Ok(read_dir) = std::fs::read_dir(&obj_dir)
                {
                    let mut files: Vec<String> = read_dir
                        .filter_map(|e| e.ok())
                        .filter_map(|e| {
                            let p = e.path();
                            if p.extension()?.to_str()? == "md" {
                                let stem = p.file_stem()?.to_str()?.to_string();
                                if stem != dir { Some(stem) } else { None }
                            } else {
                                None
                            }
                        })
                        .collect();
                    files.sort();
                    // If reflection found no public methods, avoid showing private constructors
                    if let Some(pos) = files.iter().position(|s| s == "constructor") {
                        files.remove(pos);
                    }
                    method_files = files;
                }

                write_page(object, method_files);
                written.insert(object.to_string());
            }

            // Extras: any docs-only objects in docs/api not in allowed
            if let Ok(read_dir) = std::fs::read_dir(&docs_root) {
                for entry in read_dir.flatten() {
                    if entry.path().is_dir() {
                        let dir_name = entry.file_name().to_string_lossy().to_string();
                        let object = super::validation::dir_to_object_name(&dir_name);
                        if written.contains(&object) {
                            continue;
                        }
                        let obj_dir = docs_root.join(&dir_name);
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
                            // If reflection found no public methods, avoid showing private constructors
                            if let Some(pos) = method_files.iter().position(|s| s == "constructor")
                            {
                                method_files.remove(pos);
                            }
                        }
                        write_page(&object, method_files);
                        written.insert(object);
                    }
                }
            }

            // Generate an index.mdx listing all API objects (objects first, then extras)
            let mut all_objects: Vec<String> = Vec::new();
            let objects = super::validation::public_structs_excluding_hidden();
            for o in objects.iter() {
                all_objects.push(o.clone());
            }
            if let Ok(read_dir) = std::fs::read_dir(&docs_root) {
                for entry in read_dir.flatten() {
                    if entry.path().is_dir() {
                        let dir_name = entry.file_name().to_string_lossy().to_string();
                        let object = super::validation::dir_to_object_name(&dir_name);
                        if !all_objects.iter().any(|o| o == &object) {
                            all_objects.push(object);
                        }
                    }
                }
            }

            let mut idx = String::new();
            idx.push_str("---\n");
            idx.push_str("title: API\n");
            idx.push_str("description: \"Auto-generated API index\"\n");
            idx.push_str("---\n\n");
            idx.push_str("# API\n\n");
            for o in &all_objects {
                let path = format!("/docs/api/{}.mdx", o.to_lowercase());
                idx.push_str(&format!("- [{}]({})\n", o, path));
            }
            let mut idx = idx.trim_end().to_string();
            idx.push('\n');
            std::fs::write(site_root.join("index.mdx"), idx).unwrap();

            // Cleanup: remove stale top-level MDX files that were not generated in this run.
            // Keep index.mdx and any subdirectories (e.g., versioned folders like v1.2.3).
            let mut expected: HashSet<String> = HashSet::new();
            for o in &written {
                expected.insert(format!("{}.mdx", o.to_lowercase()));
            }
            expected.insert("index.mdx".to_string());
            if let Ok(read_dir) = std::fs::read_dir(&site_root) {
                for entry in read_dir.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        continue;
                    }
                    if let (Some(ext), Some(name)) = (
                        path.extension().and_then(|s| s.to_str()),
                        path.file_name().and_then(|s| s.to_str()),
                    ) {
                        if ext == "mdx" && !expected.contains(name) {
                            let _ = std::fs::remove_file(&path);
                        }
                    }
                }
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
