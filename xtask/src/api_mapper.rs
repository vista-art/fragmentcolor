use crate::meta;
use quote::ToTokens;
use std::{
    collections::{hash_map::Entry, HashMap, HashSet},
    convert::AsRef,
    fs,
    hash::Hash,
    io::Write,
    path::{Path, PathBuf},
};
use syn::{parse_file, Ident, ImplItem, Item, ItemFn, ItemImpl, ReturnType, Visibility};

pub const API_MAP_KEYWORD: &str = "API_MAP";
pub const API_MAP_FILE: &str = "generated/api_map.rs";
pub const FUNCTION_SIGNATURE_STRUCT_NAME: &str = "FunctionSignature";
pub const FUNCTION_SIGNATURE_STRUCT_DEFINITION: &str = "
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
";

#[derive(Clone, Debug, PartialEq)]
pub struct FunctionParameter {
    pub name: String,
    pub type_name: String,
}
#[derive(Clone, Debug)]
pub struct FunctionSignature {
    pub name: String,
    pub parameters: Vec<FunctionParameter>,
    pub return_type: Option<String>,
}

impl Eq for FunctionSignature {}
impl PartialEq for FunctionSignature {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Hash for FunctionSignature {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

pub type ApiMap = HashMap<String, HashSet<FunctionSignature>>;

#[derive(Clone, Debug, PartialEq)]
enum NameFilter {
    Global,
    Specific(String),
    Rename(String, String),
}

pub fn map_public_api(crate_name: &str) {
    println!();
    println!("🗺️ Generating API map...");

    generate_api_map(crate_name);

    println!("✅ API map successfully generated!");
    println!();
}

fn generate_api_map(crate_name: &str) {
    let crate_root = meta::crate_root(crate_name);
    let api_map_file = meta::workspace_root().join(API_MAP_FILE);
    let api_map = extract_public_functions(crate_root.as_ref());

    export_api_map(api_map, api_map_file.as_ref())
}

/// Traverses a Rust library `/src` directory and returns
/// a HashMap of its public functions and their signatures
fn extract_public_functions(crate_path: &Path) -> ApiMap {
    let mut signatures = ApiMap::new();
    let (entry_path, parsed_file) = parse_lib_entry_point(crate_path.as_ref());

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
        match &item_use.tree {
            syn::UseTree::Path(use_path) => {
                let full_path = extract_full_path_from_use_tree(&item_use.tree);
                let last_segment = full_path.last().unwrap();
                let mod_name = last_segment.to_string();

                if private_modules.get(&mod_name).is_some() {
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
            _ => {}
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
            // If the item is a struct, we will extract its public methods
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

    let content = fs::read_to_string(&module_path).expect("Failed to read module file");
    (
        module_path,
        parse_file(&content)
            .expect("Failed to parse module file")
            .items,
    )
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
        if let ImplItem::Fn(method) = impl_item {
            if let Visibility::Public(_) = method.vis {
                methods.push(extract_signature(&method.sig));
            }
        }
    }

    match signatures.entry(struct_name) {
        Entry::Vacant(entry) => {
            entry.insert(HashSet::from_iter(methods));
        }
        Entry::Occupied(mut entry) => {
            entry.get_mut().extend(methods);
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
            .find_map(|ancestor| {
                if ancestor.ends_with("src") {
                    Some(ancestor)
                } else {
                    None
                }
            })
            .expect("Couldn't find parent /src directory");

        let key = path
            .strip_prefix(ancestor)
            .unwrap()
            .to_str()
            .unwrap()
            .replace("/", "_");

        match signatures.entry(key) {
            Entry::Vacant(entry) => {
                entry.insert(HashSet::from([signature]));
            }
            Entry::Occupied(mut entry) => {
                entry.get_mut().insert(signature);
            }
        }
    }
}

/// Extracts the name, parameters and return type of a function
fn extract_signature(method: &syn::Signature) -> FunctionSignature {
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

    FunctionSignature {
        name,
        parameters,
        return_type,
    }
}

/// Exports the generated API map to a static Rust file
fn export_api_map(api_map: ApiMap, target_file: &Path) {
    let mut static_map_builder = phf_codegen::Map::new();
    let mut target_file = fs::File::create(&target_file).unwrap();
    let mut writer = std::io::BufWriter::new(&mut target_file);

    for (struct_name, functions) in api_map {
        static_map_builder.entry(
            struct_name.clone(),
            &format!(
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
        "{}\n\nstatic {}: phf::Map<&'static str, &[FunctionSignature]> = {};\n",
        FUNCTION_SIGNATURE_STRUCT_DEFINITION,
        API_MAP_KEYWORD,
        static_map_builder.build()
    )
    .unwrap();
}
