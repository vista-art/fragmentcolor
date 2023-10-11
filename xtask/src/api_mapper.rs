use quote::ToTokens;
use std::{
    collections::{HashMap, HashSet},
    convert::AsRef,
    fs,
    path::{Path, PathBuf},
};
use syn::{parse_file, Ident, ImplItem, Item, ItemImpl, ReturnType, Visibility};

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

pub fn extract_function_signatures_from_crate<P: AsRef<Path>>(
    crate_path: &P,
) -> HashMap<String, Vec<FunctionSignature>> {
    let mut signatures = HashMap::new();
    let (entry_path, parsed_file) = parse_crate_entry_point(crate_path.as_ref());
    traverse_and_extract(entry_path.as_ref(), parsed_file.items, &mut signatures);

    signatures
}

fn parse_crate_entry_point(file_path: &Path) -> (PathBuf, syn::File) {
    let entry_point = if file_path.join("src/lib.rs").exists() {
        file_path.join("src/lib.rs")
    } else {
        file_path.join("src/main.rs")
    };

    let content = fs::read_to_string(entry_point.as_path()).expect("Failed to read file");
    (
        entry_point,
        parse_file(&content).expect("Failed to parse file"),
    )
}

fn traverse_and_extract(
    current_path: &Path,
    items: Vec<Item>,
    signatures: &mut HashMap<String, Vec<FunctionSignature>>,
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
                    // This method returns a tuple of Idents:
                    // The first is the the exported Item as in the module.
                    // The second is Some() if the Item has been renamed.
                    let mod_structs = extract_structs_from_use_tree(&use_path.tree);
                    reexported_modules.insert(mod_name, mod_structs);
                }
            }
            _ => {}
        }
    }

    // Third pass: Process the items
    for item in items {
        match item {
            Item::Mod(item_mod) => {
                if let Visibility::Public(_) = item_mod.vis {
                    if let Some((_, mod_items)) = item_mod.content {
                        // inline module
                        traverse_and_extract(current_path, mod_items, signatures);
                    } else {
                        // external module
                        let mod_name = item_mod.ident.to_string();
                        let (next_file, parsed_module) =
                            parse_external_module(mod_name, current_path);

                        traverse_and_extract(&next_file, parsed_module.items, signatures);
                    }
                } else {
                    let mod_name = item_mod.ident.to_string();
                    let reexported = reexported_modules.get(&mod_name).unwrap();
                    reexported.iter().for_each(|name_pair| {
                        match name_pair {
                            // The Item has been renamed by the importing module
                            (Some(exported_name), Some(renamed_name)) => {
                                // @TODO
                            }
                            // The Item has been exported as-is
                            (Some(exported_name), None) => {
                                // @TODO
                            }
                            // This is a glob import
                            (None, None) => {
                                // it runs exactly the same code as the public modules above
                                // @TODO there's an opportunity here to extract this into a function
                                if let Some((_, mod_items)) = &item_mod.content {
                                    // inline reexported module
                                    traverse_and_extract(
                                        current_path,
                                        mod_items.to_vec(),
                                        signatures,
                                    );
                                } else {
                                    // external reexported module
                                    let mod_name = item_mod.ident.to_string();
                                    let (next_file, parsed_module) =
                                        parse_external_module(mod_name, current_path);

                                    traverse_and_extract(
                                        &next_file,
                                        parsed_module.items,
                                        signatures,
                                    );
                                }
                            }
                            _ => unreachable!(),
                        }
                    });
                }
            }
            Item::Impl(item_impl) => {
                extract_impl(item_impl, signatures);
            }
            _ => {}
        }
    }
}

// We get only the path names, so we can compare
// the module names with the last segment
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

// We discard the path and extract only the leaf nodes
fn extract_structs_from_use_tree(tree: &syn::UseTree) -> Vec<(Option<Ident>, Option<Ident>)> {
    match tree {
        syn::UseTree::Path(use_path) => extract_structs_from_use_tree(&use_path.tree),
        syn::UseTree::Name(use_name) => {
            vec![(Some(use_name.ident.clone()), None)]
        }
        syn::UseTree::Group(use_group) => {
            let mut structs = Vec::new();
            for use_tree in &use_group.items {
                let new_vec = extract_structs_from_use_tree(use_tree);
                structs.append(&mut Vec::from(new_vec.as_slice()));
            }
            structs
        }
        syn::UseTree::Glob(use_glob) => {
            vec![(None, None)]
        }
        syn::UseTree::Rename(use_rename) => {
            vec![(
                Some(use_rename.ident.clone()),
                Some(use_rename.rename.clone()),
            )]
        }
    }
}

fn parse_external_module(mod_name: String, file_path: &Path) -> (PathBuf, syn::File) {
    let current_dir = file_path.parent().unwrap();
    let next_file = if current_dir.join(format!("{}.rs", mod_name)).exists() {
        current_dir.join(format!("{}.rs", mod_name))
    } else {
        current_dir.join(mod_name).join("mod.rs")
    };

    let content = fs::read_to_string(&next_file).expect("Failed to read module file");
    (
        next_file,
        parse_file(&content).expect("Failed to parse module file"),
    )
}

fn extract_impl(item_impl: ItemImpl, signatures: &mut HashMap<String, Vec<FunctionSignature>>) {
    let struct_name = match *item_impl.self_ty {
        syn::Type::Path(type_path) => type_path.path.segments.last().unwrap().ident.to_string(),
        _ => {
            return;
        }
    };

    let mut methods = Vec::new();
    for impl_item in &item_impl.items {
        if let ImplItem::Fn(method) = impl_item {
            if let Visibility::Public(_) = method.vis {
                methods.push(extract_signature(&method.sig));
            }
        }
    }

    if !methods.is_empty() {
        signatures.insert(struct_name, methods);
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{create_dir_all, File};
    use std::io::Write;
    use tempfile::tempdir;

    fn create_temp_file(filename: &str, content: &str, dir: &tempfile::TempDir) {
        let file_path = dir.path().join(filename);
        if let Some(parent) = file_path.parent() {
            create_dir_all(parent).unwrap();
        }
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "{}", content).unwrap();
    }

    fn create_mock_crate(dir: &tempfile::TempDir) -> &str {
        create_temp_file(
            "src/lib.rs",
            r#"
            pub mod module;

            pub struct LibStruct;

            impl LibStruct {
                pub fn public_method(&self, arg: i32) -> String {
                    "".to_string()
                }

                fn private_method(&self) {}
            }
            "#,
            &dir,
        );

        create_temp_file(
            "src/module/mod.rs",
            r#"
            pub struct ModStruct;

            impl ModStruct {
                pub fn mod_public_method(&self) {}
                fn mod_private_method(&self) {}
            }
            "#,
            &dir,
        );

        dir.path().to_str().unwrap()
    }

    #[test]
    fn test_extract_function_signatures_from_crate() {
        let dir = tempdir().unwrap();
        let crate_path: String = create_mock_crate(&dir).to_string();

        let signatures = extract_function_signatures_from_crate(&crate_path);

        assert_eq!(signatures.len(), 2);
        assert!(signatures.contains_key("LibStruct"));
        assert!(signatures.contains_key("ModStruct"));

        let lib_methods = signatures.get("LibStruct").unwrap();
        assert_eq!(lib_methods.len(), 1);
        assert_eq!(lib_methods[0].name, "public_method");
        assert_eq!(
            lib_methods[0].parameters,
            vec![FunctionParameter {
                name: "arg".to_string(),
                type_name: "i32".to_string()
            }]
        );
        assert_eq!(lib_methods[0].return_type, Some("String".to_string()));

        let mod_methods = signatures.get("ModStruct").unwrap();
        assert_eq!(mod_methods.len(), 1);
        assert_eq!(mod_methods[0].name, "mod_public_method");
        assert_eq!(mod_methods[0].parameters.len(), 0);
        assert_eq!(mod_methods[0].return_type, None);
    }
}
