use crate::FunctionSignature;
use proc_macro::TokenStream;
use quote::quote;
use syn::Ident;

// use phf::phf_map;
// static TYPE_CONVERSIONS: phf::Map<&'static str, &'static str> = phf_map! {
//     "impl AsRef < Path >" => "String",
// };

pub(crate) fn wrap(struct_ident: Ident, method_signatures: &[FunctionSignature]) -> TokenStream {
    let struct_name = struct_ident.to_string();
    let wrapper_name = format!("Py{}", struct_name);
    let wrapper_ident = syn::Ident::new(&wrapper_name, struct_ident.span());

    // generate Pyo3 wrappers
    let methods = method_signatures
        .iter()
        .map(|signature| {
            let name = syn::parse_str::<syn::Ident>(&signature.name).unwrap();

            let parameters = signature
                .parameters
                .iter()
                .map(|parameter| {
                    let name = syn::parse_str::<syn::Ident>(&parameter.name).unwrap();
                    let type_name = syn::parse_str::<syn::Type>(&parameter.type_name).unwrap();

                    // @TODO match trait bounds or skip them
                    // see https://pyo3.rs/v0.19.2/trait_bounds
                    match type_name {
                        _ => (),
                    }

                    quote! {
                        #name: #type_name,
                    }
                })
                .collect::<Vec<_>>();

            let parameter_names = signature
                .parameters
                .iter()
                .map(|parameter| {
                    let name = syn::parse_str::<syn::Ident>(&parameter.name).unwrap();
                    quote! {
                        #name,
                    }
                })
                .collect::<Vec<_>>();

            let mut returns_self = false;
            let return_type = if signature.return_type.is_some() {
                let return_type =
                    syn::parse_str::<syn::Type>(&signature.return_type.as_ref().unwrap()).unwrap();

                let type_name = format!("{:?}", return_type);
                returns_self = type_name == "Self" || type_name == struct_name;

                quote! {
                    -> #return_type
                }
            } else {
                quote! {}
            };

            // NOTE: If there is no method marked with #[new],
            // object instances can only be created from Rust,
            // but not from Python.
            //
            // We create an additional constructor, so that
            // pyo3 won't override the default constructor.
            let constructor = if returns_self && signature.name == "new" {
                quote! {
                    #[new]
                    fn __new__(#(#parameters)*) -> #wrapper_ident {
                        #wrapper_ident {
                            inner: #struct_ident::new(#(#parameter_names)*)
                        }
                    }
                }
            } else {
                quote! {}
            };

            if returns_self {
                quote! {
                    #constructor

                    #[staticmethod]
                    fn #name(#(#parameters)*) -> #wrapper_ident {
                        #wrapper_ident {
                            inner: #struct_ident::#name(#(#parameter_names)*)
                        }
                    }
                }
            } else {
                quote! {
                    #constructor

                    fn #name(&self, #(#parameters)*) #return_type {
                        self.inner.#name(#(#parameter_names)*)
                    }
                }
            }
        })
        .collect::<Vec<_>>();

    println!("🔎 Generating Pyo3 wrapper struct {}", wrapper_ident);
    let expanded = quote! {
        #[pyclass(name = #struct_name)]
        pub struct #wrapper_ident {
            inner: #struct_ident,
        }

        #[pymethods]
        impl #wrapper_ident {
            #(#methods)*
        }
    };

    TokenStream::from(expanded)
}
