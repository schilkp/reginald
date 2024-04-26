use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use reginald_utils::{field_byte_to_packed_byte_transform, Endianess, ShiftDirection};

use crate::{
    input::{EnumDeriveInput, FieldType, StructDeriveInput},
    utils::prefix_ident,
};

pub fn derive_struct_to_bytes(input: &StructDeriveInput) -> syn::Result<TokenStream> {
    let mut lines: Vec<TokenStream> = vec![];

    let width_bytes = input.width_bytes;
    let name = &input.name;

    for field in &input.fields {
        let field_width_bytes = field.field_type.trait_width_bytes();
        let field_name = &field.name;
        let field_arr_name = prefix_ident("field_", &field.name);

        // Convert field to bits and store in new array:
        let line = if matches!(field.field_type, FieldType::Bool) {
            quote_spanned! { field.name.span() => let #field_arr_name: [u8; #field_width_bytes] = if self.#field_name {[1]} else {[0]}; }
        } else {
            quote_spanned! { field.name.span() => let #field_arr_name: [u8; #field_width_bytes] = self.#field_name.to_le_bytes(); }
        };
        lines.push(line);

        // Transfer field bits into result array:
        let field_mask_unpos = field.bits.unpositioned();
        let field_pos = field.bits.lsb_pos();

        for byte in 0..width_bytes {
            for field_byte in 0..field_width_bytes {
                // Determine required transform to put byte 'field_byte' of field into 'byte' of
                // output:
                let transform = field_byte_to_packed_byte_transform(
                    Endianess::Little,
                    &field_mask_unpos,
                    field_pos,
                    field_byte,
                    width_bytes,
                    byte,
                    width_bytes,
                );

                let Some(transform) = transform else {
                    continue;
                };

                // Grab byte from field array:
                let field_byte = quote! {#field_arr_name [#field_byte]};

                // Shift byte:
                let field_byte = match &transform.shift {
                    Some((ShiftDirection::Left, amnt)) => quote!( (#field_byte << #amnt) ),
                    Some((ShiftDirection::Right, amnt)) => quote!( (#field_byte >> #amnt) ),
                    None => field_byte,
                };

                // Mask byte:
                let field_byte = if transform.mask != 0xFF {
                    let mask = transform.mask;
                    quote!( #field_byte & #mask )
                } else {
                    field_byte
                };

                let line = quote! { result[#byte] |= #field_byte; };
                lines.push(line);
            }
        }
    }

    // Fixed bits:
    for byte in 0..width_bytes {
        let mask = input.fixed_bits.mask.get_le_byte(byte);
        let val = input.fixed_bits.value.get_le_byte(byte);

        if mask == 0 {
            continue;
        }
        lines.push(quote! { result[#byte] &= !#mask; });
        lines.push(quote! { result[#byte] |= #val; });
    }

    let out = quote! {
        impl ::reginald::ToBytes<#width_bytes> for #name {
            fn to_le_bytes(&self) -> [u8; #width_bytes] {
                use ::reginald::ToBytes;
                let mut result: [u8; #width_bytes] = [0; #width_bytes];
                #(#lines)*
                result
            }
        }
    };

    // let file: syn::File = syn::parse2(out.clone()).unwrap();
    // eprintln!("{}", prettyplease::unparse(&file));
    // panic!();

    Ok(out)
}

pub fn derive_enum_to_bytes(input: &EnumDeriveInput) -> syn::Result<TokenStream> {
    let mut match_arms: Vec<TokenStream> = vec![];

    let width_bytes = input.width_bytes;
    let name = &input.name;

    for variant in &input.variants {
        let variant_name = &variant.name;
        let variant_val = &variant.value | &input.fixed_bits.value;

        let le_bytes: Vec<u8> = (0..width_bytes).map(|x| variant_val.get_le_byte(x)).collect();

        match_arms.push(quote! { Self::#variant_name => [#(#le_bytes),*], });
    }

    let out = quote! {
        impl ::reginald::ToBytes<#width_bytes> for #name {
            fn to_le_bytes(&self) -> [u8; #width_bytes] {
                match self {
                    #(#match_arms)*
                }
            }
        }
    };

    // let file: syn::File = syn::parse2(out.clone()).unwrap();
    // eprintln!("{}", prettyplease::unparse(&file));
    // panic!();

    Ok(out)
}

pub fn derive_struct_from_bytes(input: &StructDeriveInput) -> syn::Result<TokenStream> {
    let _ = input;
    todo!()
}

pub fn derive_enum_from_bytes(input: &EnumDeriveInput) -> syn::Result<TokenStream> {
    let _ = input;
    todo!()
}
