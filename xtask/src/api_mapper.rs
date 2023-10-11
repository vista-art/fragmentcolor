use quote::ToTokens;
use std::{collections::HashMap, fs, path::Path};
use syn::{parse_file, ImplItem, Item, ItemImpl, ReturnType, Visibility};

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

pub fn extract_function_signatures_from_crate(
    crate_path: &str,
) -> HashMap<String, Vec<FunctionSignature>> {
    let mut signatures = HashMap::new();
    let entry_path = Path::new(crate_path).join("src/lib.rs");
    traverse_and_extract(&entry_path, &mut signatures);
    signatures
}

fn traverse_and_extract(
    file_path: &Path,
    signatures: &mut HashMap<String, Vec<FunctionSignature>>,
) {
    let content = fs::read_to_string(file_path).expect("Failed to read file");
    let parsed_file = parse_file(&content).expect("Failed to parse file");

    for item in parsed_file.items {
        match item {
            Item::Mod(item_mod) => {
                if let Visibility::Public(_) = item_mod.vis {
                    if let Some((_, mod_items)) = item_mod.content {
                        for mod_item in mod_items {
                            if let Item::Impl(item_impl) = mod_item {
                                extract_impl(item_impl, signatures);
                            }
                        }
                    } else {
                        let mod_name = item_mod.ident.to_string();
                        let next_file = if file_path
                            .parent()
                            .unwrap()
                            .join(format!("{}.rs", mod_name))
                            .exists()
                        {
                            file_path.parent().unwrap().join(format!("{}.rs", mod_name))
                        } else {
                            file_path.parent().unwrap().join(mod_name).join("mod.rs")
                        };
                        traverse_and_extract(&next_file, signatures);
                    }
                }
            }
            Item::Impl(item_impl) => {
                extract_impl(item_impl, signatures);
            }
            _ => {}
        }
    }
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
        let crate_path = create_mock_crate(&dir);

        let signatures = extract_function_signatures_from_crate(crate_path);

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
