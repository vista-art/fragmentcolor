use crate::ObjectProperty;
use proc_macro::TokenStream;
use quote::quote;
use syn::Ident;

pub(crate) fn wrap(struct_ident: Ident, object_property: &[ObjectProperty]) -> TokenStream {
    let struct_name = struct_ident.to_string();
    let wrapper_name = format!("Py{}", struct_name);
    let wrapper_ident = syn::Ident::new(&wrapper_name, struct_ident.span());

    // generate Pyo3 wrappers
    let methods = object_property
        .iter()
        .map(|property| {
            let name = syn::parse_str::<syn::Ident>(&property.name).expect("1");
            let type_name = syn::parse_str::<syn::Type>(&property.type_name).expect("2");

            if let Some(signature) = &property.function {
                let parameters = signature
                    .parameters
                    .iter()
                    .map(|parameter| {
                        let name = syn::parse_str::<syn::Ident>(&parameter.name).expect("3");
                        let type_name = if let Some(_) = super::API_MAP.get(&parameter.type_name) {
                            // Parameter is a custom type from our API.
                            // We assume all of our custom objects will
                            // be wrapped by a "Py" prefix in the caller
                            let type_wrapper = format!("Py{}", &parameter.type_name);
                            let _type_wrapper_ident = syn::Ident::new(&type_wrapper, name.span());

                            syn::parse_str::<syn::Type>(type_wrapper.as_str()).expect("4")
                        } else {
                            // parameter is a native Rust type
                            syn::parse_str::<syn::Type>(&parameter.type_name).expect("5")
                        }; // @TODO deal with generics

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
                        let name = syn::parse_str::<syn::Ident>(&parameter.name).expect("6");
                        quote! {
                            #name,
                        }
                    })
                    .collect::<Vec<_>>();

                let mut returns_self = false;
                let return_type = if signature.return_type.is_some() {
                    let return_type =
                        syn::parse_str::<syn::Type>(&signature.return_type.as_ref().expect("7"))
                            .expect("8");

                    let type_name = format!("{:?}", return_type);
                    returns_self = type_name == "Self" || type_name == struct_name;

                    quote! {
                        -> #return_type
                    }
                } else {
                    quote! {}
                };

                let constructor = if signature.name == "new" {
                    quote! {
                        #[new]
                        fn new(#(#parameters)*) -> #wrapper_ident {
                            #wrapper_ident {
                                inner: #struct_ident::new(#(#parameter_names)*)
                            }
                        }
                    }
                } else {
                    quote! {}
                };

                if returns_self && signature.name != "new" {
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
            } else {
                quote! {
                    #[getter]
                    fn #name(&self) -> #type_name {
                        self.inner.#name
                    }

                    #[setter]
                    fn #name(&mut self, value: #type_name) {
                        self.inner.#name = value;
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
