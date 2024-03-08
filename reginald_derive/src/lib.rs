// use darling::FromDeriveInput;
use proc_macro::{self, TokenStream};
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

// #[derive(FromDeriveInput, Default, Debug)]
// #[darling(attributes(reginald))]
// struct PackOpts {
//     width_bits: usize,
// }

#[proc_macro_derive(Pack, attributes(reginald))]
pub fn derive_pack(input: TokenStream) -> TokenStream {
    let _input: DeriveInput = parse_macro_input!(input);
    // let _opts = PackOpts::from_derive_input(&input).unwrap();

    // eprintln!("{:?}", _opts);

    // let output = quote! {
    //     impl MyTrait for #ident {}
    // };

    let output = quote! {};
    output.into()
}

// #[proc_macro_derive(Unpack, attributes(reginald))]
// pub fn derive_unpack(input: TokenStream) -> TokenStream {
//     todo!()
// }
//
// #[proc_macro_derive(TryUnpack, attributes(reginald))]
// pub fn derive_try_unpack(input: TokenStream) -> TokenStream {
//     todo!()
// }
//
