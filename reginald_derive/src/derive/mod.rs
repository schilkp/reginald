use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use reginald_utils::{
    field_byte_to_packed_byte_transform, packed_byte_to_field_byte_transform, Bits, Endianess, ShiftDirection,
};

use crate::{
    input::{EnumDeriveInput, FieldType, StructDeriveInput, UInt},
    utils::{prefix_ident, spanned_err},
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
    let width_bytes = input.width_bytes;
    let name = &input.name;

    let mut match_arms: Vec<TokenStream> = vec![];
    for variant in &input.variants {
        let variant_name = &variant.name;
        let variant_val = &variant.value;

        let le_bytes: Vec<u8> = (0..width_bytes).map(|x| variant_val.get_le_byte(x)).collect();

        match_arms.push(quote! { Self::#variant_name => [#(#le_bytes),*] ,});
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
    let width_bytes = input.width_bytes;
    let name = &input.name;

    // Extract field bytes into seperate arrays:
    let mut lines: Vec<TokenStream> = vec![];
    for field in &input.fields {
        let field_width_bytes = field.field_type.trait_width_bytes();
        let field_arr_name = prefix_ident("field_", &field.name);
        let field_mask_unpos = field.bits.unpositioned();
        let field_pos = field.bits.lsb_pos();

        // Create array:
        lines.push(quote! { let mut #field_arr_name : [u8; #field_width_bytes] = [0; #field_width_bytes]; });

        // Extract field bytes into new array:
        for byte in 0..width_bytes {
            for field_byte in 0..field_width_bytes {
                // Determine required transform to put byte 'byte' of packed input into 'field_byte' of
                // field:
                let transform = packed_byte_to_field_byte_transform(
                    Endianess::Little,
                    &field_mask_unpos,
                    field_pos,
                    field_byte,
                    field_width_bytes,
                    byte,
                    width_bytes,
                );

                let Some(transform) = transform else {
                    continue;
                };

                // Grab byte from input array:
                let byte = quote! {val [#byte]};

                // Mask byte:
                let byte = if transform.mask != 0xFF {
                    let mask = transform.mask;
                    quote!( (#byte & #mask) )
                } else {
                    byte
                };

                // Shift byte:
                let byte = match &transform.shift {
                    Some((ShiftDirection::Left, amnt)) => quote!( #byte << #amnt ),
                    Some((ShiftDirection::Right, amnt)) => quote!( #byte >> #amnt ),
                    None => byte,
                };

                let line = quote! { #field_arr_name[#field_byte] |= #byte; };
                lines.push(line);
            }
        }
    }

    let mut result_fields: Vec<TokenStream> = vec![];
    for field in &input.fields {
        let field_name = &field.name;
        let field_arr_name = prefix_ident("field_", &field.name);

        let result_field = match &field.field_type {
            FieldType::UInt(UInt::U8) => {
                quote_spanned! { field.name.span() => #field_name: u8::from_le_bytes(#field_arr_name), }
            }
            FieldType::UInt(UInt::U16) => {
                quote_spanned! { field.name.span() => #field_name: u16::from_le_bytes(#field_arr_name), }
            }
            FieldType::UInt(UInt::U32) => {
                quote_spanned! { field.name.span() => #field_name: u32::from_le_bytes(#field_arr_name), }
            }
            FieldType::UInt(UInt::U64) => {
                quote_spanned! { field.name.span() => #field_name: u64::from_le_bytes(#field_arr_name), }
            }
            FieldType::UInt(UInt::U128) => {
                quote_spanned! { field.name.span() => #field_name: u128::from_le_bytes(#field_arr_name), }
            }
            FieldType::Trait(t) => {
                let field_type_name = &field.field_type_name;
                if t.masked {
                    quote_spanned! { field.field_type_name.span() => #field_name: #field_type_name::from_masked_le_bytes(&#field_arr_name), }
                } else {
                    quote_spanned! { field.field_type_name.span() => #field_name: #field_type_name::from_le_bytes(&#field_arr_name), }
                }
            }
            FieldType::Bool => quote_spanned! { field.name.span() => #field_name: #field_arr_name[0] != 0, },
        };

        result_fields.push(result_field);
    }

    let out = quote! {
        impl ::reginald::FromBytes<#width_bytes> for #name {
            fn from_le_bytes(val: &[u8; #width_bytes]) -> Self {
                use ::reginald::FromBytes;
                use ::reginald::FromMaskedBytes;
                #(#lines)*
                Self {
                    #(#result_fields)*
                }
            }
        }
    };

    // let file: syn::File = syn::parse2(out.clone()).unwrap();
    // eprintln!("{}", prettyplease::unparse(&file));
    // panic!();

    Ok(out)
}

pub fn derive_enum_from_bytes(input: &EnumDeriveInput) -> syn::Result<TokenStream> {
    let width_bytes = input.width_bytes;
    let name = &input.name;

    // Validate that this enum can cover all width_bytes values:
    let mask = if width_bytes == 0 {
        Bits::from_uint(0)
    } else {
        Bits::from_range(0..=(width_bytes * 8 - 1))
    };
    if !input.can_always_unpack_mask(&mask) {
        return spanned_err!(
            &input.name,
            "Reginald: Cannot derive FromBytes because enum does not accept all {width_bytes}-byte values."
        );
    }

    let mut match_arms: Vec<TokenStream> = vec![];
    for variant in &input.variants {
        let variant_name = &variant.name;
        let variant_bytes: Vec<u8> = (0..width_bytes).map(|x| variant.value.get_le_byte(x)).collect();
        match_arms.push(quote! {[#(#variant_bytes),*] => Self::#variant_name,});
    }

    let out = quote! {
        impl ::reginald::FromBytes<#width_bytes> for #name {
            fn from_le_bytes(val: &[u8; #width_bytes]) -> Self {
                match val {
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

pub fn derive_struct_try_from_bytes(input: &StructDeriveInput) -> syn::Result<TokenStream> {
    let _ = input;
    todo!()
}

pub fn derive_enum_try_from_bytes(input: &EnumDeriveInput) -> syn::Result<TokenStream> {
    let _ = input;
    todo!()
}

pub fn derive_enum_from_masked_bytes(input: &EnumDeriveInput) -> syn::Result<TokenStream> {
    let width_bytes = input.width_bytes;
    let name = &input.name;

    // Calculate covered bits and ensure that all values that fit that mask can be converted:
    let mut mask = Bits::new();
    for variant in &input.variants {
        mask |= &variant.value;
    }
    if !input.can_always_unpack_mask(&mask) {
        return spanned_err!(&input.name, "Reginald: Cannot derive FromMaskedBytes because enum does not accept all values that fit into occupied bits mask.");
    }

    let masked_input_bytes: Vec<TokenStream> = (0..width_bytes)
        .map(|x| {
            let byte_mask = mask.get_le_byte(x);
            quote! (val[#x] & #byte_mask)
        })
        .collect();

    let mut match_arms: Vec<TokenStream> = vec![];
    for variant in &input.variants {
        let variant_name = &variant.name;
        let variant_bytes: Vec<u8> = (0..width_bytes).map(|x| variant.value.get_le_byte(x)).collect();
        match_arms.push(quote! {[#(#variant_bytes),*] => Self::#variant_name,});
    }

    let out = quote! {
        impl ::reginald::FromMaskedBytes<#width_bytes> for #name {
            fn from_masked_le_bytes(val: &[u8; #width_bytes]) -> Self {
                match [#(#masked_input_bytes),*] {
                    #(#match_arms)*
                    _ => unreachable!(),
                }
            }
        }
    };

    // let file: syn::File = syn::parse2(out.clone()).unwrap();
    // eprintln!("{}", prettyplease::unparse(&file));
    // panic!();

    Ok(out)
}
