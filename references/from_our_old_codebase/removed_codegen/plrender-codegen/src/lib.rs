extern crate proc_macro;

mod python;
mod wasm;
use proc_macro::TokenStream;
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

fn parse_input(input: MacroInput) -> (Ident, &'static [FunctionSignature]) {
    let struct_ident = input.struct_name;
    let method_signatures: &[FunctionSignature] = API_MAP
        .get(struct_ident.to_string().as_str())
        .expect(format!("🛑 No API Map found for {}", struct_ident).as_str());

    (struct_ident, method_signatures)
}

#[proc_macro]
pub fn wrap_py(tokens: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(tokens as MacroInput);
    let (struct_ident, method_signatures) = parse_input(input);
    let wrapped = python::wrap(struct_ident, method_signatures);
    TokenStream::from(wrapped)
}

#[proc_macro]
pub fn wrap_wasm(tokens: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(tokens as MacroInput);
    let (struct_ident, method_signatures) = parse_input(input);
    let wrapped = wasm::wrap(struct_ident, method_signatures);
    TokenStream::from(wrapped)
}
