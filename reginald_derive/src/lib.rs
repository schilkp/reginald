extern crate proc_macro;
use proc_macro::TokenStream;

#[proc_macro_derive(Pack, attributes(reginald))]
pub fn derive_pack(_inp: TokenStream) -> TokenStream {
    TokenStream::new()
}

#[proc_macro_derive(TryUnpack, attributes(reginald))]
pub fn derive_try_unpack(_inp: TokenStream) -> TokenStream {
    TokenStream::new()
}

#[proc_macro_derive(Unpack, attributes(reginald))]
pub fn derive_unpack(_inp: TokenStream) -> TokenStream {
    TokenStream::new()
}
