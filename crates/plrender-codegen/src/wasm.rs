use crate::FunctionSignature;
use proc_macro::TokenStream;
use quote::quote;
use syn::Ident;

pub(crate) fn wrap(struct_ident: Ident, method_signatures: &[FunctionSignature]) -> TokenStream {
    let struct_name = struct_ident.to_string();
    let wrapper_name = format!("Py{}", struct_name);
    let wrapper_ident = syn::Ident::new(&wrapper_name, struct_ident.span());

    // generate WASM wrappers
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

            let constructor = if returns_self && signature.name == "new" {
                quote! {
                    #[wasm_bindgen(constructor)]
                    fn _new_(#(#parameters)*) -> #wrapper_ident {
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

    println!("🔎 Generating WASM wrapper struct {}", wrapper_ident);
    let struct_name = struct_ident.to_string();
    let expanded = quote! {
        #[wasm_bindgen(js_name = #struct_name, getter_with_clone)]
        pub struct #wrapper_ident {
            inner: #struct_ident,
        }

        #[wasm_bindgen(js_class = #struct_name)]
        impl #wrapper_ident {
            #(#methods)*
        }
    };

    TokenStream::from(expanded)
}
