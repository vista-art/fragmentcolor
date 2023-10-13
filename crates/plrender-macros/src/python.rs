use crate::FunctionSignature;
use proc_macro::TokenStream;
use quote::quote;
use syn::Ident;

pub(crate) fn wrap(struct_ident: Ident, method_signatures: &[FunctionSignature]) -> TokenStream {
    let wrapper_name = format!("Py{}", struct_ident);
    let wrapper_ident = syn::Ident::new(&wrapper_name, struct_ident.span());

    // generate Pyo3 wrappers
    let methods = method_signatures
        .iter()
        .map(|signature| {
            let name = &signature.name;

            let parameters = signature
                .parameters
                .iter()
                .map(|parameter| {
                    let name = &parameter.name;
                    let type_name = &parameter.type_name;

                    quote! {
                        #name: #type_name,
                    }
                })
                .collect::<Vec<_>>();

            let return_type = if signature.return_type.is_some() {
                let return_type = signature.return_type.as_ref().unwrap();
                quote! {
                    -> #return_type
                }
            } else {
                quote! {}
            };

            let constructor = if signature.name == "new" {
                quote! {
                    #[new]
                }
            } else {
                quote! {}
            };

            // NOTE: If there is no method marked with #[new],
            // object instances can only be created from Rust,
            // but not from Python.
            quote! {
                #constructor
                fn #name(&self #(#parameters)*) #return_type {
                    self.inner.#name(#(#parameters)*)
                }
            }
        })
        .collect::<Vec<_>>();

    let expanded = quote! {
        #[pyclass(name = #struct_ident)]
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
