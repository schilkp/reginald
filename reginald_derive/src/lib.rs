mod utils;

mod derive;
mod input;

use derive::{
    derive_enum_from_bytes, derive_enum_from_masked_bytes, derive_enum_to_bytes, derive_enum_try_from_bytes,
    derive_struct_from_bytes, derive_struct_to_bytes, derive_struct_try_from_bytes,
};
use proc_macro::TokenStream;
use syn::parse_macro_input;

use crate::utils::spanned_err;

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

#[proc_macro_derive(TryFromBytes, attributes(reginald))]
pub fn derive_try_from_bytes(inp: TokenStream) -> TokenStream {
    let inp = parse_macro_input!(inp as input::ReginaldDeriveInput);
    let result = match inp {
        input::ReginaldDeriveInput::Struct(struct_data) => derive_struct_try_from_bytes(&struct_data),
        input::ReginaldDeriveInput::Enum(enum_data) => derive_enum_try_from_bytes(&enum_data),
    };
    match result {
        Ok(s) => s.into(),
        Err(e) => e.into_compile_error().into(),
    }
}

#[proc_macro_derive(FromMaskedBytes, attributes(reginald))]
pub fn derive_from_masked_bytes(inp: TokenStream) -> TokenStream {
    let inp = parse_macro_input!(inp as input::ReginaldDeriveInput);
    let result = match inp {
        input::ReginaldDeriveInput::Struct(struct_data) => {
            spanned_err!(struct_data.name, "Reginald: FrommaskedBytes cannot be derived for a struct.")
        }
        input::ReginaldDeriveInput::Enum(enum_data) => derive_enum_from_masked_bytes(&enum_data),
    };
    match result {
        Ok(s) => s.into(),
        Err(e) => e.into_compile_error().into(),
    }
}
