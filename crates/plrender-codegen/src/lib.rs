extern crate proc_macro;

mod python;
mod wasm;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse::Parse, Ident};

// API map of the most recent `plrender` build
// Static map of object names to their method signatures
include!("../../../generated/api_map.rs");

pub(crate) struct MacroInput {
    struct_name: Ident,
}

impl Parse for MacroInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            struct_name: input.parse()?,
        })
    }
}

fn parse_input(input: MacroInput) -> (Ident, &'static [ObjectProperty]) {
    let struct_ident = input.struct_name;
    let object_properties: &[ObjectProperty] = API_MAP
        .get(struct_ident.to_string().as_str())
        .expect(format!("🛑 No API Map found for {}", struct_ident).as_str());

    (struct_ident, object_properties)
}

#[proc_macro]
pub fn py_module(_: TokenStream) -> TokenStream {
    let mut objects = Vec::new();
    let mut functions = Vec::new();

    for (struct_name, object_properties) in API_MAP.entries {
        if struct_name.ends_with(".rs") {
            let name = object_properties[0].name;
            let tokens = quote! {
                m.add_function(wrap_pyfunction!(#name, m)?)?;
            };

            functions.push(tokens);
        } else {
            let wrapper_name = format!("Py{}", struct_name);
            let tokens = quote! {
                m.add_class::<#wrapper_name>()?;
            };

            objects.push(tokens);
        }
    }

    let expanded = quote! {
        #[pymodule]
        fn plrender(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
            #(#functions)*
            #(#objects)*
            Ok(())
        }
    };
    TokenStream::from(expanded)
}

#[proc_macro]
pub fn wrap_py(tokens: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(tokens as MacroInput);
    let (struct_ident, object_properties) = parse_input(input);
    let wrapped = python::wrap(struct_ident, object_properties);
    TokenStream::from(wrapped)
}

#[proc_macro]
pub fn wrap_wasm(tokens: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(tokens as MacroInput);
    let (struct_ident, object_properties) = parse_input(input);
    let wrapped = wasm::wrap(struct_ident, object_properties);
    TokenStream::from(wrapped)
}
