use crate::FunctionSignature;
use plrender::*;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse::Parse, parse_macro_input, Ident};

pub fn wrap(tokens: TokenStream, method_signatures: FunctionSignature) -> TokenStream {
    let input = parse_macro_input!(tokens as MacroInput);
    let struct_name = &input.struct_name;

    let wrapper_ident = utility::get_reflect_ident(&struct_name.to_string());
    let wrapper_name = format!("Py{}", struct_name);
    let wrapper_ident = syn::Ident::new(&wrapper_name, struct_name.span());

    let methods: Vec<_> = method_signatures
        .iter()
        .map(|sig| {
            let method_name = syn::Ident::new(sig.name, struct_name.span());
            quote! {
                fn #method_name(&self) {
                    self.inner.#method_name()
                }
            }
        })
        .collect();

    // @FIXME it would be safer if the wrapper also implemented the trait
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

            // Generate wrapper methods based on the struct's methods
            #(#methods)*
        }
    };

    TokenStream::from(expanded)
}
