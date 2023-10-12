extern crate proc_macro;

mod python;
mod wasm;

use plrender::*;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse::Parse, parse_macro_input, Ident};

// API map of the most recent `plrender` build
// Static map of object names to their method signatures
include!("../../generated/api_map.rs");

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

#[proc_macro]
pub fn wrap_py(tokens: TokenStream) -> TokenStream {
    let method_signatures: phf::Map<&'static str, FunctionSignature> = API_MAP
        .get(&struct_name.to_string())
        .expect("Unknown struct!");

    let wrapped = python::wrap(tokens, method_signatures);

    TokenStream::from(wrapped)
}
