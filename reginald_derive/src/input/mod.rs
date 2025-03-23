mod parse;

use std::{collections::HashSet, fmt::Display};

use proc_macro2::Ident;
use reginald_utils::{RangeStyle, bits::Bits};
use syn::parse::{Parse, ParseStream};

use crate::{
    input::parse::{parse_enum_variant, parse_struct, parse_struct_field},
    utils::{WithTokens, attach_spanned_error, spanned_err},
};

use self::parse::{FieldTypeInfo, StructFieldInfo, parse_enum};

// ==== Derive Input ===========================================================

#[derive(Debug)]
pub enum ReginaldDeriveInput {
    Struct(StructDeriveInput),
    Enum(EnumDeriveInput),
}

impl Parse for ReginaldDeriveInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let input = syn::DeriveInput::parse(input)?;
        match &input.data {
            syn::Data::Struct(struct_data) => parse_struct_derive_input(&input, struct_data),
            syn::Data::Enum(enum_data) => parse_enum_derive_input(&input, enum_data),
            syn::Data::Union(union_data) => {
                spanned_err!(union_data.union_token, "Reginald: Cannot derive for union.")
            }
        }
    }
}

// ==== Struct =================================================================

#[derive(Debug)]
pub struct StructDeriveInput {
    pub name: Ident,
    pub width_bytes: usize,
    pub fixed_bits: FixedBits,
    pub fields: Vec<Field>,
}

#[derive(Debug, Default, Clone)]
pub struct FixedBits {
    pub value: Bits,
    pub mask: Bits,
}

#[derive(Debug)]
pub struct Field {
    pub name: Ident,
    pub bits: Bits,
    pub field_type: FieldType,
    pub field_type_name: Ident,
}

#[derive(Debug, Clone)]
pub enum FieldType {
    UInt(UInt),
    Trait(TraitField),
    Bool,
}

#[derive(Debug, Clone)]
pub enum UInt {
    U8,
    U16,
    U32,
    U64,
    U128,
}

#[derive(Debug, Clone)]
pub struct TraitField {
    pub trait_width_bytes: usize,
    pub masked: bool,
}

fn parse_struct_derive_input(
    input: &syn::DeriveInput,
    struct_data: &syn::DataStruct,
) -> syn::Result<ReginaldDeriveInput> {
    // Parse struct and fields:
    let struct_info = parse_struct(input)?;

    let syn::Fields::Named(fields_data) = &struct_data.fields else {
        return spanned_err!(input, "Reginald: Tuple and Newtype structs are not supported.");
    };
    let mut field_infos = vec![];
    for field in &fields_data.named {
        field_infos.push(parse_struct_field(field)?);
    }
    let mut fields: Vec<Field> = vec![];
    for info in field_infos.iter() {
        fields.push(convert_field(info)?);
    }

    // Determine struct width:
    let min_width_bytes_fields = fields.iter().map(|x| x.bits.width_bytes()).max().unwrap_or(0);
    let min_width_bytes_fixed = struct_info.fixed_bits_attr.mask.width_bytes();
    let min_width_bytes = usize::max(min_width_bytes_fields, min_width_bytes_fixed);
    let width_bytes = if let Some(width_bytes) = struct_info.width_bytes_attr {
        if width_bytes.inner < min_width_bytes {
            return spanned_err!(
                width_bytes,
                "Reginald: Given width is too small to contain all fields and fixed bits."
            );
        }
        width_bytes.inner
    } else {
        min_width_bytes
    };

    // Check for overlap between fields and fields/fixed bits:
    let mut all_bits = vec![];
    for field in &field_infos {
        for bits_orig in &field.bits_attr_orig {
            all_bits.push(bits_orig.clone());
        }
    }
    for bit_orig in &struct_info.fixed_bits_attr_orig {
        all_bits.push(bit_orig.clone());
    }
    check_for_bits_overlap(&all_bits, "Field bits")?;

    Ok(ReginaldDeriveInput::Struct(StructDeriveInput {
        name: input.ident.clone(),
        fixed_bits: struct_info.fixed_bits_attr.clone(),
        width_bytes,
        fields,
    }))
}

fn convert_field(field_info: &StructFieldInfo) -> syn::Result<Field> {
    // Determine field type:
    let trait_width_bytes = field_info
        .trait_width_bytes_attr
        .clone()
        .map(|x| x.inner)
        .unwrap_or(field_info.bits_attr.positioned_width_bytes());

    let field_type = if let Some(type_attr) = &field_info.type_attr {
        match &type_attr.inner {
            parse::FieldTypeAttr::UInt(uint) => FieldType::UInt(uint.clone()),
            parse::FieldTypeAttr::Bool => FieldType::Bool,
            parse::FieldTypeAttr::Trait => FieldType::Trait(TraitField {
                trait_width_bytes,
                masked: false,
            }),
            parse::FieldTypeAttr::TraitMasked => FieldType::Trait(TraitField {
                trait_width_bytes,
                masked: true,
            }),
        }
    } else {
        match &field_info.field_type.inner {
            FieldTypeInfo::UInt(uint) => FieldType::UInt(uint.clone()),
            FieldTypeInfo::Bool => FieldType::Bool,
            FieldTypeInfo::Trait => FieldType::Trait(TraitField {
                trait_width_bytes,
                masked: false,
            }),
        }
    };

    let field_type_tokens = if let Some(type_attr) = &field_info.type_attr {
        &type_attr.tokens
    } else {
        &field_info.field_type.tokens
    };

    // Error if specified bits is incompatible with field type:
    let bitwidth = field_info.bits_attr.bitwidth();
    match &field_type {
        FieldType::Bool => {
            if bitwidth > 1 {
                return spanned_err!(
                    field_type_tokens,
                    "Reginald: A boolean field can only be 1 bit wide, but this field is {bitwidth} bits wide.",
                );
            }
        }
        FieldType::UInt(u) => {
            if bitwidth > u.width_bytes() * 8 {
                return spanned_err!(
                    field_type_tokens,
                    "Reginald: A {u} field can only be 1 bit wide. but this field is {bitwidth} bits wide.",
                );
            }
        }
        FieldType::Trait(_) => (),
    }

    // Error if trait width is specified but field is not a trait field:
    if let Some(width_attr) = &field_info.trait_width_bytes_attr {
        if !matches!(field_type, FieldType::Trait(_)) {
            return spanned_err!(width_attr, "Reginald: Trait width specified but field is a primitive.");
        }
    }

    Ok(Field {
        name: field_info.name.clone(),
        bits: field_info.bits_attr.clone(),
        field_type_name: field_info.field_type_name.clone(),
        field_type,
    })
}

impl FieldType {
    pub fn trait_width_bytes(&self) -> usize {
        match self {
            FieldType::UInt(u) => u.width_bytes(),
            FieldType::Trait(t) => t.trait_width_bytes,
            FieldType::Bool => 1,
        }
    }
}

impl UInt {
    pub fn width_bytes(&self) -> usize {
        match self {
            UInt::U8 => 1,
            UInt::U16 => 2,
            UInt::U32 => 4,
            UInt::U64 => 8,
            UInt::U128 => 16,
        }
    }
}

impl Display for UInt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            UInt::U8 => "u8",
            UInt::U16 => "u16",
            UInt::U32 => "u32",
            UInt::U64 => "u64",
            UInt::U128 => "u128",
        };
        f.write_str(s)
    }
}

// ==== Enum ===================================================================

#[derive(Debug)]
pub struct EnumDeriveInput {
    pub name: Ident,
    pub width_bytes: usize,
    pub variants: Vec<Variant>,
}

#[derive(Debug)]
pub struct Variant {
    pub name: Ident,
    pub value: Bits,
}

fn parse_enum_derive_input(input: &syn::DeriveInput, enum_data: &syn::DataEnum) -> syn::Result<ReginaldDeriveInput> {
    // Parse enum and variants:
    let enum_info = parse_enum(input)?;

    let mut variant_infos = vec![];
    for variant in &enum_data.variants {
        variant_infos.push(parse_enum_variant(variant)?);
    }

    let mut variant_vals: Vec<WithTokens<Bits>> = vec![];
    for variant_info in &variant_infos {
        let value = match (&variant_info.val_attr, &variant_info.discriminant) {
            (Some(attr), _) => attr,
            (None, Some(discr)) => discr,
            (None, None) => {
                return spanned_err!(
                    &variant_info.name,
                    "Reginald: Variant missing value. Add attribute or discriminant."
                );
            }
        };

        // Check for collision with other variants:
        for other_var in &variant_vals {
            if other_var.inner == value.inner {
                return spanned_err!(other_var, "Reginald: Variants with same value (1/2).")
                    .map_err(|mut x| attach_spanned_error!(x, value, "Reginald: Variants with same value (2/2)."));
            }
        }

        variant_vals.push(value.clone());
    }

    let mut variants = vec![];
    for (info, val) in std::iter::zip(variant_infos, variant_vals) {
        variants.push(Variant {
            name: info.name,
            value: val.inner,
        })
    }

    let min_width_bytes = variants.iter().map(|x| x.value.width_bytes()).max().unwrap_or(0);
    let width_bytes = if let Some(width_bytes) = enum_info.width_bytes_attr {
        if width_bytes.inner < min_width_bytes {
            return spanned_err!(width_bytes, "Reginald: Given width is too small to contain all variants.");
        }
        width_bytes.inner
    } else {
        min_width_bytes
    };

    Ok(ReginaldDeriveInput::Enum(EnumDeriveInput {
        name: enum_info.name,
        width_bytes,
        variants,
    }))
}

impl EnumDeriveInput {
    pub fn can_always_unpack_mask(&self, unpos_mask: &Bits) -> bool {
        // All enum values that fit into the mask:
        let enum_vals: HashSet<Bits> = self
            .variants
            .iter()
            .map(|x| x.value.clone())
            .filter(|x| x.clear_mask(unpos_mask).is_zero())
            .collect();

        // Number of values the mask can represent:
        let mask_bit_count = unpos_mask.to_bit_positions().len();
        let mask_vals_count = 2_u128.pow(mask_bit_count.try_into().unwrap());

        let enum_vals_count: u128 = enum_vals.len().try_into().unwrap();

        mask_vals_count == enum_vals_count
    }
}

// ==== Utils ==================================================================

fn check_for_bits_overlap(bits: &[WithTokens<Bits>], kind_msg: &str) -> syn::Result<()> {
    for idx_a in 0..bits.len() {
        for idx_b in (idx_a + 1)..bits.len() {
            let a = &bits[idx_a];
            let b = &bits[idx_b];

            if a.inner.overlaps_with(&b.inner) {
                let overlap = (&a.inner & &b.inner).to_bit_ranges_str(RangeStyle::RustInclusive);

                return spanned_err!(a, "Reginald: {} overlapping (1/2). Overlapping bits: [{}]", kind_msg, overlap)
                    .map_err(|mut x| {
                        attach_spanned_error!(
                            x,
                            b,
                            "Reginald: {} overlapping (2/2). Overlapng bits: [{}]",
                            kind_msg,
                            overlap
                        )
                    });
            }
        }
    }
    Ok(())
}
