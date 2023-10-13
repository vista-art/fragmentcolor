extern crate proc_macro;

mod python;
mod wasm;
use proc_macro::TokenStream;
use syn::{parse::Parse, Ident};

// API map of the most recent `plrender` build
// Static map of object names to their method signatures
include!("../../../generated/api_map.rs");

const EMPTY: &'static [FunctionSignature] = &[];

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
    let struct_name = input.struct_name;
    let method_signatures = API_MAP
        .get(struct_name.to_string().as_str())
        .unwrap_or(&EMPTY);

    // let method_signatures = if method_signatures.is_none() {
    //     println!("⚠️ No API map found for struct {}", struct_name);
    //     &[].as_slice()
    // } else {
    //     &[method_signatures].as_slice()
    // };

    (struct_name, method_signatures)
}

#[proc_macro]
pub fn wrap_py(tokens: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(tokens as MacroInput);
    let (struct_name, method_signatures) = parse_input(input);
    let wrapped = python::wrap(struct_name, method_signatures);
    TokenStream::from(wrapped)
}

#[proc_macro]
pub fn wrap_wasm(tokens: TokenStream) -> TokenStream {
    //let input = syn::parse_macro_input!(tokens as MacroInput);
    //let (struct_name, method_signatures) = parse_input(input);
    //let wrapped = wasm::wrap(struct_name, method_signatures);
    //TokenStream::from(wrapped)

    // just passing it forward for now
    tokens
}
