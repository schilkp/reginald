use super::*;

use crate::{
    bits::bitwidth_to_width_bytes,
    builtin::rs::{array_literal, masked_array_literal},
    error::Error,
};

use super::rs_pascalcase;

/// Generate enum.
pub(super) fn generate_enum(out: &mut dyn Write, inp: &Input, e: &Enum) -> Result<(), Error> {
    writeln!(out)?;
    generate_doc_comment(out, &e.docs, "")?;

    // Enum derives:
    let derives = inp.enum_derives.join(", ");
    writeln!(out, "#[derive({derives})]")?;

    // Enum proper:
    writeln!(out, "pub enum {} {{", rs_pascalcase(&e.name))?;
    for entry in e.entries.values() {
        generate_doc_comment(out, &entry.docs, "    ")?;
        writeln!(out, "    {}, // 0x{:X}", rs_pascalcase(&entry.name), entry.value)?;
    }
    writeln!(out, "}}")?;

    Ok(())
}

/// Generate conversion impls.
pub(super) fn generate_enum_impls(out: &mut dyn Write, inp: &Input, e: &Enum) -> Result<(), Error> {
    // Smallest uint type that can be used to represent the enum's content:
    let enum_name = rs_pascalcase(&e.name);
    let width_bytes = bitwidth_to_width_bytes(e.bitwidth);
    let trait_prefix = trait_prefix(inp);

    match enum_impl(e) {
        FromBytesImpl::FromBytes => {
            writeln!(out)?;
            writeln!(out, "impl {trait_prefix}FromBytes<{width_bytes}> for {enum_name} {{")?;
            writeln!(out, "    fn from_le_bytes(val: &[u8; {width_bytes}]) -> Self {{")?;
            writeln!(out, "        match val {{")?;
            for entry in e.entries.values() {
                let entry_val = array_literal(Endianess::Little, entry.value, width_bytes);
                let entry_name = rs_pascalcase(&entry.name);
                writeln!(out, "            {entry_val} => Self::{entry_name},")?;
            }
            writeln!(out, "        }}")?;
            writeln!(out, "    }}")?;
            writeln!(out, "}}")?;
        }
        FromBytesImpl::WrappingFromBytes => {
            let mut masked_array = vec![];
            for i in 0..width_bytes {
                let mask = grab_byte(Endianess::Little, e.occupied_bits(), i, width_bytes);
                let byte = if mask == 0xFF {
                    format!("val[{i}]")
                } else if mask == 0x00 {
                    String::from("0x0")
                } else {
                    format!("val[{i}] & 0x{mask:X}")
                };
                masked_array.push(byte);
            }

            let masked_array = format!("[{}]", masked_array.join(", "));
            writeln!(out)?;
            writeln!(out, "impl {trait_prefix}WrappingFromBytes<{width_bytes}> for {enum_name} {{")?;
            writeln!(out, "    fn wrapping_from_le_bytes(val: &[u8; {width_bytes}]) -> Self {{")?;
            writeln!(out, "        match {masked_array} {{")?;
            for entry in e.entries.values() {
                let entry_val = array_literal(Endianess::Little, entry.value, width_bytes);
                let entry_name = rs_pascalcase(&entry.name);
                writeln!(out, "            {entry_val} => Self::{entry_name},")?;
            }
            writeln!(out, "            _ => unreachable!(),")?;
            writeln!(out, "        }}")?;
            writeln!(out, "    }}")?;
            writeln!(out, "}}")?;

            writeln!(out)?;
            writeln!(out, "impl {trait_prefix}TryFromBytes<{width_bytes}> for {enum_name} {{")?;
            writeln!(out, "    type Error = {trait_prefix}FromBytesError;")?;
            writeln!(out)?;
            writeln!(out, "    fn try_from_le_bytes(val: &[u8; {width_bytes}]) -> Result<Self, Self::Error> {{")?;
            if !trait_prefix.is_empty() {
                writeln!(out, "        use {trait_prefix}WrappingFromBytes;")?;
            }
            let bytes_outside = masked_array_literal(Endianess::Little, "val", !e.occupied_bits(), width_bytes);
            writeln!(out, "        let bytes_outside = {bytes_outside};")?;
            writeln!(out, "        if bytes_outside == [0; {width_bytes}] {{")?;
            writeln!(out, "            Ok(Self::wrapping_from_le_bytes(val))")?;
            writeln!(out, "        }} else {{")?;
            writeln!(out, "            Err(Self::Error {{pos: 0}})")?;
            writeln!(out, "        }}")?;
            writeln!(out, "    }}")?;
            writeln!(out, "}}")?;
        }
        FromBytesImpl::TryFromBytes => {
            writeln!(out)?;
            writeln!(out, "impl {trait_prefix}TryFromBytes<{width_bytes}> for {enum_name} {{")?;
            writeln!(out, "    type Error = {trait_prefix}FromBytesError;")?;
            // Conversion:
            writeln!(out)?;
            writeln!(out, "    fn try_from_le_bytes(val: &[u8; {width_bytes}]) -> Result<Self, Self::Error> {{")?;
            writeln!(out, "        match val {{")?;
            for entry in e.entries.values() {
                writeln!(
                    out,
                    "           {} => Ok(Self::{}),",
                    array_literal(Endianess::Little, entry.value, width_bytes),
                    rs_pascalcase(&entry.name)
                )?;
            }
            writeln!(out, "            _ => Err(Self::Error {{pos: 0}})")?;
            writeln!(out, "        }}")?;
            writeln!(out, "    }}")?;

            writeln!(out, "}}")?;
        }
    }

    writeln!(out)?;
    writeln!(out, "impl {trait_prefix}ToBytes<{width_bytes}> for {enum_name} {{")?;
    writeln!(out, "    fn to_le_bytes(&self) -> [u8; {width_bytes}] {{")?;
    writeln!(out, "        match self {{")?;
    for entry in e.entries.values() {
        let entry_val = array_literal(Endianess::Little, entry.value, width_bytes);
        let entry_name = rs_pascalcase(&entry.name);
        writeln!(out, "            Self::{entry_name} => {entry_val},")?;
    }
    writeln!(out, "        }}")?;
    writeln!(out, "    }}")?;
    writeln!(out, "}}")?;

    Ok(())
}
