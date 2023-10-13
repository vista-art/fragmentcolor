use crate::FunctionSignature;
use proc_macro::TokenStream;
//use quote::quote;
use syn::Ident;

pub(crate) fn _wrap(_struct_ident: Ident, _method_signatures: &[FunctionSignature]) -> TokenStream {
    unimplemented!()
}
