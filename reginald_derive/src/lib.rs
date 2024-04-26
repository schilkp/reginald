mod utils;

mod derive;
mod input;

use derive::{derive_enum_from_bytes, derive_enum_to_bytes, derive_struct_from_bytes, derive_struct_to_bytes};
use proc_macro::TokenStream;
use syn::parse_macro_input;

#[proc_macro_derive(ToBytes, attributes(reginald))]
pub fn to_bytes(inp: TokenStream) -> proc_macro::TokenStream {
    let inp = parse_macro_input!(inp as input::ReginaldDeriveInput);
    let result = match inp {
        input::ReginaldDeriveInput::Struct(struct_data) => derive_struct_to_bytes(&struct_data),
        input::ReginaldDeriveInput::Enum(enum_data) => derive_enum_to_bytes(&enum_data),
    };
    match result {
        Ok(s) => s.into(),
        Err(e) => e.into_compile_error().into(),
    }
}

#[proc_macro_derive(FromBytes, attributes(reginald))]
pub fn from_bytes(inp: TokenStream) -> proc_macro::TokenStream {
    let inp = parse_macro_input!(inp as input::ReginaldDeriveInput);
    let result = match inp {
        input::ReginaldDeriveInput::Struct(struct_data) => derive_struct_from_bytes(&struct_data),
        input::ReginaldDeriveInput::Enum(enum_data) => derive_enum_from_bytes(&enum_data),
    };
    match result {
        Ok(s) => s.into(),
        Err(e) => e.into_compile_error().into(),
    }
}

// #[proc_macro_derive(TryFromBytes, attributes(reginald))]
// pub fn derive_try_from_bytes(_inp: TokenStream) -> TokenStream {
//     TokenStream::new()
// }

// #[proc_macro_derive(FromBytesMasked, attributes(reginald))]
// pub fn derive_from_bytes_masked(_inp: TokenStream) -> TokenStream {
//     TokenStream::new()
// }
