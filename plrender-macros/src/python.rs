use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{Ident, ReturnType};

// !!!! This was copied from Xtask !!!!
// @TODO Extract the function parser to a separate module
// remove it from the xtask build system, so both the
// build system and the parser would use the same logic
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

pub fn wrap(struct_name: Ident, method_signatures: &str) -> TokenStream {
    let wrapper_name = format!("Py{}", struct_name);
    let wrapper_ident = syn::Ident::new(&wrapper_name, struct_name.span());

    let signatures = method_signatures
        .split(";")
        .filter_map(|signature| {
            if signature.is_empty() {
                None
            } else {
                let parsed = syn::parse_str::<syn::Signature>(signature);
                if parsed.is_err() {
                    None
                } else {
                    let signature = parsed.unwrap();
                    Some(extract_signature(&signature))
                }
            }
        })
        .collect::<Vec<_>>();

    // generate Pyo3 wrappers for function signatures
    let methods = signatures
        .iter()
        .map(|signature| {
            let name = &signature.name;
            // @TODO let parameters = &signature.parameters;
            let return_type = &signature.return_type;

            quote! {
                fn #name(&self) #return_type {
                    self.inner.#name()
                }
            }
        })
        .collect::<Vec<_>>();

    let expanded = quote! {
        #[pyclass(name = #struct_name)]
        pub struct #wrapper_ident {
            inner: #struct_name,
        }

        #[pymethods]
        impl #wrapper_ident {
            #[new]
            fn __new__(obj: &pyo3::PyRawObject) -> pyo3::PyResult<()> {
                obj.init(|_token| #wrapper_ident { inner: #struct_name::new() })
            }

            #(#methods)*
        }
    };

    TokenStream::from(expanded)
}

// !!!! This was copied from Xtask !!!!
// @TODO Extract the function parser to a separate module
// remove it from the xtask build system, so both the
// build system and the parser would use the same logic

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

#[cfg(test)]
mod tests {}
