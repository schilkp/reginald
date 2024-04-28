use std::ops::RangeInclusive;

use proc_macro2::Ident;
use reginald_utils::Bits;
use syn::{punctuated::Punctuated, DeriveInput, Expr, Field, Lit, Meta, Token, Variant};

use crate::utils::{attach_spanned_error, spanned_err, spanned_error, WithTokens};

use super::{check_for_bits_overlap, FixedBits, UInt};

// ==== Struct =================================================================

#[derive(Debug)]
pub struct StructInfo {
    pub name: Ident,
    pub width_bytes_attr: Option<WithTokens<usize>>,
    pub fixed_bits_attr: FixedBits,
    pub fixed_bits_attr_orig: Vec<WithTokens<Bits>>,
}

pub fn parse_struct(inp: &DeriveInput) -> syn::Result<StructInfo> {
    // ==== Attributes ====
    let mut width_bytes_attr: Option<WithTokens<usize>> = None;
    let mut fixed_bits_attrs: Vec<WithTokens<FixedBits>> = vec![];

    for attr in inp.attrs.iter().filter(|x| x.path().is_ident("reginald")) {
        let nested = attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;
        for meta in nested {
            if meta.path().is_ident("width_bytes") {
                // #[reginald(width_bytes = 5)]
                let name_value = meta.require_name_value()?;
                let new = WithTokens::new(Box::new(name_value.clone()), parse_usize(&name_value.value)?);
                check_for_repeated_attr(&width_bytes_attr, &new)?;
                width_bytes_attr = Some(new);
            } else if meta.path().is_ident("fixed_bits") {
                // #[reginald(fixed_bits = (1, 0))]
                // #[reginald(fixed_bits = ([1, 4..=20], 0))]
                let name_value = meta.require_name_value()?;
                let fixed_bits_attr = parse_fixed_bit_tuple(&name_value.value)?;
                fixed_bits_attrs.push(WithTokens::new(Box::new(name_value.clone()), fixed_bits_attr));
            } else {
                return spanned_err!(&meta.path(), "Reginald: Unknown struct attribute.");
            };
        }
    }

    check_for_bits_overlap(&Vec::from_iter(fixed_bits_attrs.iter().map(|x| x.map(|x| x.mask.clone()))), "Fixed bits")?;
    let mut mask = Bits::new();
    let mut value = Bits::new();
    let mut fixed_bits_orig = vec![];
    for piece in fixed_bits_attrs {
        mask |= &piece.inner.mask;
        value |= &piece.inner.value;
        fixed_bits_orig.push(piece.map(|x| x.mask.clone()));
    }
    let fixed_bits = FixedBits { value, mask };

    // === Info ====

    let name = inp.ident.clone();

    Ok(StructInfo {
        name,
        width_bytes_attr,
        fixed_bits_attr: fixed_bits,
        fixed_bits_attr_orig: fixed_bits_orig,
    })
}

// ==== Struct Field ===========================================================

#[derive(Debug)]
pub struct StructFieldInfo {
    pub name: Ident,
    pub field_type: WithTokens<FieldTypeInfo>,
    pub field_type_name: Ident,
    pub bits_attr: Bits,
    pub bits_attr_orig: Vec<WithTokens<Bits>>,
    pub trait_width_bytes_attr: Option<WithTokens<usize>>,
    pub type_attr: Option<WithTokens<FieldTypeAttr>>,
}

#[derive(Debug)]
pub enum FieldTypeAttr {
    UInt(UInt),
    Bool,
    Trait,
    TraitMasked,
}

#[derive(Debug)]
pub enum FieldTypeInfo {
    UInt(UInt),
    Trait,
    Bool,
}

pub fn parse_struct_field(inp: &Field) -> syn::Result<StructFieldInfo> {
    // === Attributes ====

    let mut bits_attrs_orig: Vec<WithTokens<Bits>> = vec![];
    let mut trait_width_bytes_attr: Option<WithTokens<usize>> = None;
    let mut type_attr: Option<WithTokens<FieldTypeAttr>> = None;

    for attr in inp.attrs.iter().filter(|x| x.path().is_ident("reginald")) {
        let nested = attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;
        for meta in nested {
            if meta.path().is_ident("bits") {
                // #[reginald(bits = 1)]
                // #[reginald(bits = 1..=3)]
                // #[reginald(bits = [1..=3, 10])]
                let name_value = meta.require_name_value()?;
                let bits_attr = parse_bits(&name_value.value)?;
                bits_attrs_orig.push(WithTokens::new(Box::new(name_value.clone()), bits_attr));
            } else if meta.path().is_ident("trait_width_bytes") {
                // #[reginald(trait_width_bytes = 1)]
                let name_value = meta.require_name_value()?;
                let val = WithTokens::new(Box::new(name_value.clone()), parse_usize(&name_value.value)?);
                check_for_repeated_attr(&trait_width_bytes_attr, &val)?;
                trait_width_bytes_attr = Some(val);
            } else if meta.path().is_ident("is") {
                // #[reginald(is=u8)]
                let name_value = meta.require_name_value()?;
                let val = WithTokens::new(Box::new(name_value.clone()), parse_type_attr(&name_value.value)?);
                check_for_repeated_attr(&type_attr, &val)?;
                type_attr = Some(val);
            } else {
                return spanned_err!(&meta.path(), "Reginald: Unknown field attribute.");
            };
        }
    }

    if bits_attrs_orig.is_empty() {
        return spanned_err!(inp.ident.clone(), "Reginald: Field is missing bit position attribute.");
    }

    check_for_bits_overlap(&bits_attrs_orig, "Field bit position")?;
    let mut bits = Bits::new();
    for bit in &bits_attrs_orig {
        bits |= &bit.inner;
    }

    // === Field Info ====

    let syn::Type::Path(ty) = &inp.ty else {
        return spanned_err!(inp.ty.clone(), "Reginald: Unsupported type.");
    };

    let field_type = if ty.path.is_ident("u8") {
        FieldTypeInfo::UInt(UInt::U8)
    } else if ty.path.is_ident("u16") {
        FieldTypeInfo::UInt(UInt::U16)
    } else if ty.path.is_ident("u32") {
        FieldTypeInfo::UInt(UInt::U32)
    } else if ty.path.is_ident("u64") {
        FieldTypeInfo::UInt(UInt::U64)
    } else if ty.path.is_ident("u128") {
        FieldTypeInfo::UInt(UInt::U128)
    } else if ty.path.is_ident("bool") {
        FieldTypeInfo::Bool
    } else {
        FieldTypeInfo::Trait
    };

    let field_type = WithTokens::new(Box::new(ty.path.clone()), field_type);

    let Some(field_type_name) = ty.path.get_ident().cloned() else {
        return spanned_err!(ty, "Reginald: Field has no type.");
    };

    let name: Ident = inp
        .ident
        .clone()
        .ok_or_else(|| spanned_error!(&inp, "Reginald: Field has no name."))?;

    Ok(StructFieldInfo {
        name,
        field_type,
        field_type_name,
        bits_attr: bits,
        bits_attr_orig: bits_attrs_orig,
        trait_width_bytes_attr,
        type_attr,
    })
}

// ==== Enum ===================================================================

#[derive(Debug)]
pub struct EnumInfo {
    pub name: Ident,
    pub width_bytes_attr: Option<WithTokens<usize>>,
}

pub fn parse_enum(inp: &DeriveInput) -> syn::Result<EnumInfo> {
    // ==== Attributes ====
    let mut width_bytes_attr: Option<WithTokens<usize>> = None;

    for attr in inp.attrs.iter().filter(|x| x.path().is_ident("reginald")) {
        let nested = attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;
        for meta in nested {
            if meta.path().is_ident("width_bytes") {
                // #[reginald(width_bytes = 5)]
                let name_value = meta.require_name_value()?;
                let new = WithTokens::new(Box::new(name_value.clone()), parse_usize(&name_value.value)?);
                check_for_repeated_attr(&width_bytes_attr, &new)?;
                width_bytes_attr = Some(new);
            } else {
                return spanned_err!(&meta.path(), "Reginald: Unknown enum attribute.");
            };
        }
    }

    // === Info ====

    let name = inp.ident.clone();

    Ok(EnumInfo { name, width_bytes_attr })
}

// ==== Enum Variant ===========================================================

#[derive(Debug)]
pub struct EnumVariantInfo {
    pub name: Ident,
    pub discriminant: Option<WithTokens<Bits>>,
    pub val_attr: Option<WithTokens<Bits>>,
}

pub fn parse_enum_variant(inp: &Variant) -> syn::Result<EnumVariantInfo> {
    // === Attributes ====

    let mut val_attr: Option<WithTokens<Bits>> = None;

    for attr in inp.attrs.iter().filter(|x| x.path().is_ident("reginald")) {
        let nested = attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;
        for meta in nested {
            if meta.path().is_ident("value") {
                // #[reginald(value = 1)]
                // FIXME: Need scheme for arbitrary size value input
                let name_value = meta.require_name_value()?;
                let new = parse_u128(&name_value.value)?;
                let new = Bits::from_uint(new);
                let new = WithTokens::new(Box::new(name_value.clone()), new);
                check_for_repeated_attr(&val_attr, &new)?;
                val_attr = Some(new);
            } else {
                return spanned_err!(&meta.path(), "Reginald: Unknown variant attribute.");
            };
        }
    }

    // === Info ====
    let name: Ident = inp.ident.clone();

    let mut discriminant = None;
    if let Some(explicit_discriminanbt) = &inp.discriminant {
        let value = parse_u128(&explicit_discriminanbt.1)?;
        let bits = Bits::from_uint(value);
        discriminant = Some(WithTokens::new(Box::new(explicit_discriminanbt.0), bits));
    }

    if !inp.fields.is_empty() {
        return spanned_err!(&inp.fields, "Reginald: Variants with data are not supported.");
    }

    Ok(EnumVariantInfo {
        name,
        val_attr,
        discriminant,
    })
}

// ==== Parsing Funcs ==========================================================

fn parse_usize(inp: &Expr) -> syn::Result<usize> {
    let Expr::Lit(lit) = inp else {
        return spanned_err!(inp, "Reginald: Expected integer literal.");
    };

    let Lit::Int(lit) = &lit.lit else {
        return spanned_err!(inp, "Reginald: Expected integer literal.");
    };

    lit.base10_parse()
}

fn parse_u128(inp: &Expr) -> syn::Result<u128> {
    let Expr::Lit(lit) = inp else {
        return spanned_err!(inp, "Reginald: Expected integer literal.");
    };

    let Lit::Int(lit) = &lit.lit else {
        return spanned_err!(inp, "Reginald: Expected integer literal.");
    };

    lit.base10_parse()
}

fn parse_range(inp: &Expr) -> syn::Result<RangeInclusive<usize>> {
    let Expr::Range(range) = inp else {
        return spanned_err!(inp, "Reginald: Expected range.");
    };

    let Some(Expr::Lit(start)) = range.start.as_deref() else {
        return spanned_err!(inp, "Reginald: Range must have integer lower bound.");
    };
    let Lit::Int(start) = &start.lit else {
        return spanned_err!(start, "Reginald: Range must have integer lower bound.");
    };
    let start: usize = start.base10_parse()?;

    let Some(Expr::Lit(end)) = range.end.as_deref() else {
        return spanned_err!(inp, "Reginald: Range must have integer upper bound.");
    };
    let Lit::Int(end) = &end.lit else {
        return spanned_err!(start, "Reginald: Range must have integer upper bound.");
    };
    let end: usize = end.base10_parse()?;

    match range.limits {
        syn::RangeLimits::HalfOpen(_) => {
            if end <= start {
                spanned_err!(start, "Reginald: Range end must be greater than start. Range may not be empty.")
            } else {
                Ok(start..=(end - 1))
            }
        }
        syn::RangeLimits::Closed(_) => {
            if end < start {
                spanned_err!(start, "Reginald: Range end must be greater than start.")
            } else {
                Ok(start..=end)
            }
        }
    }
}

fn parse_bits(inp: &Expr) -> syn::Result<Bits> {
    // Valid bits:
    //  - `1`
    //  - `1..2`
    //  - `1..=2`
    //  - `[1, 2..3, 4..=6]`

    match inp {
        Expr::Lit(_) => {
            let pos = parse_usize(inp)?;
            Ok(Bits::from_bitpos(pos))
        }
        Expr::Range(_) => {
            let range = parse_range(inp)?;
            Ok(Bits::from_range(range))
        }
        Expr::Array(array) => {
            let mut bits = Bits::new();

            if array.elems.is_empty() {
                return spanned_err!(array, "Reginald: Bits may not be empty.");
            }

            for piece in &array.elems {
                match piece {
                    Expr::Lit(lit) => {
                        let pos = parse_usize(piece)?;
                        let additional_bits = &Bits::from_bitpos(pos);
                        if bits.overlaps_with(additional_bits) {
                            return spanned_err!(lit, "Reginald: Bits overlapping with themselves.");
                        }
                        bits |= additional_bits;
                    }
                    Expr::Range(range_token) => {
                        let range = parse_range(piece)?;
                        let additional_bits = &Bits::from_range(range);
                        if bits.overlaps_with(additional_bits) {
                            return spanned_err!(range_token, "Reginald: Bits overlapping with themselves.");
                        }
                        bits |= additional_bits;
                    }
                    _ => {
                        return spanned_err!(piece, "Reginald: Expected integer literal or range.");
                    }
                }
            }

            Ok(bits)
        }
        _ => {
            spanned_err!(inp, "Reginald: Expected integer literal, range, or array.")
        }
    }
}

fn parse_fixed_bit_tuple(inp: &Expr) -> syn::Result<FixedBits> {
    let Expr::Tuple(tup) = inp else {
        return spanned_err!(inp, "Reginald: Expected tuple.");
    };

    if tup.elems.len() != 2 {
        return spanned_err!(inp, "Reginald: Tuple must be of length two.");
    }

    let bits_expr = &tup.elems[0];
    let val_expr = &tup.elems[1];

    let bits = parse_bits(bits_expr)?;
    let val = Bits::from_uint(parse_u128(val_expr)?) << bits.lsb_pos();

    if val.clear_mask(&bits).is_nonzero() {
        return spanned_err!(inp, "Reginald: Fixed value does not fit into fixed bits.");
    }

    Ok(FixedBits { mask: bits, value: val })
}

fn parse_type_attr(inp: &Expr) -> syn::Result<FieldTypeAttr> {
    let Expr::Path(path) = inp else {
        return spanned_err!(inp, "Reginald: Expected identifier.");
    };

    if path.path.is_ident("u8") {
        Ok(FieldTypeAttr::UInt(UInt::U8))
    } else if path.path.is_ident("u16") {
        Ok(FieldTypeAttr::UInt(UInt::U16))
    } else if path.path.is_ident("u32") {
        Ok(FieldTypeAttr::UInt(UInt::U32))
    } else if path.path.is_ident("u64") {
        Ok(FieldTypeAttr::UInt(UInt::U64))
    } else if path.path.is_ident("u128") {
        Ok(FieldTypeAttr::UInt(UInt::U128))
    } else if path.path.is_ident("Bool") {
        Ok(FieldTypeAttr::Bool)
    } else if path.path.is_ident("trait") {
        Ok(FieldTypeAttr::Trait)
    } else if path.path.is_ident("trait_masked") {
        Ok(FieldTypeAttr::TraitMasked)
    } else {
        spanned_err!(inp, "Reginald: Unknown type.")
    }
}

fn check_for_repeated_attr<T>(is: &Option<WithTokens<T>>, new: &WithTokens<T>) -> syn::Result<()> {
    if is.is_some() {
        spanned_err!(is, "Reginald: Repeat attribute (1/2).")
            .map_err(|mut x| attach_spanned_error!(x, new, "Reginald: Repeated attribute (2/2).",))
    } else {
        Ok(())
    }
}
