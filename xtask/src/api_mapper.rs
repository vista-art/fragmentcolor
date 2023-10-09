use syn::{parse_file, Item, ItemImpl, ItemTrait, ReturnType, Type};

#[derive(Clone)]
pub struct FunctionSignature {
    pub name: &'static str,
    pub parameters: Vec<&'static str>,
    pub return_type: Option<&'static str>,
}

pub fn extract_function_signatures(file_path: &str) -> Vec<FunctionSignature> {
    let content = std::fs::read_to_string(file_path).expect("Failed to read file");
    let parsed_file = parse_file(&content).expect("Failed to parse file");

    let mut signatures = Vec::new();

    for item in parsed_file.items {
        match item {
            Item::Trait(item_trait) => {
                for trait_item in &item_trait.items {
                    if let syn::TraitItem::Method(method) = trait_item {
                        if method.vis == syn::Visibility::Public {
                            signatures.push(extract_signature(&method.sig));
                        }
                    }
                }
            }
            Item::Impl(ItemImpl { items, .. }) => {
                for impl_item in items {
                    if let syn::ImplItem::Method(method) = impl_item {
                        if method.vis == syn::Visibility::Public {
                            signatures.push(extract_signature(&method.sig));
                        }
                    }
                }
            }
            _ => {}
        }
    }

    signatures
}

fn extract_signature(method: &syn::Signature) -> FunctionSignature {
    let name = method.ident.to_string();
    let parameters = method
        .inputs
        .iter()
        .filter_map(|arg| {
            if let syn::FnArg::Typed(pat_type) = arg {
                Some(pat_type.ty.to_token_stream().to_string())
            } else {
                None
            }
        })
        .collect();

    let return_type = match &method.output {
        ReturnType::Default => None,
        ReturnType::Type(_, ty) => Some(ty.to_token_stream().to_string()),
    };

    FunctionSignature {
        name: Box::leak(name.into_boxed_str()),
        parameters: parameters
            .into_iter()
            .map(|param| Box::leak(param.into_boxed_str()))
            .collect(),
        return_type: return_type.map(|ret| Box::leak(ret.into_boxed_str())),
    }
}
