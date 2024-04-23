use std::fmt::Write;

use super::*;

use crate::{
    bits::{lsb_pos, mask_to_bit_ranges_str, unpositioned_mask},
    error::Error,
    regmap::{FieldType, Layout},
    utils::{
        field_byte_to_packed_byte_transform, field_to_packed_byte_transform, grab_byte,
        packed_byte_to_field_byte_transform, remove_wrapping_parens, Endianess, ShiftDirection,
    },
    writer::indent_writer::IndentWriter,
};

use super::{rs_pascalcase, rs_snakecase};

// Generate content for a layout struct
pub fn generate_layout(out: &mut dyn Write, inp: &Input, layout: &Layout, in_module: bool) -> Result<(), Error> {
    let mut out = HeaderWriter::new(out);

    // Layout struct:
    generate_layout_struct(&mut out, inp, layout, None, in_module)?;

    out.push_section_with_header(&["\n", "// Layout-specific enums and sub-layouts:", "\n"]);

    for e in layout.nested_local_enums() {
        enums::generate_enum(&mut out, inp, e)?;
    }

    for local_layout in layout.nested_local_layouts() {
        generate_layout_struct(&mut out, inp, local_layout, None, in_module)?;
    }

    out.pop_section();
    out.push_section_with_header(&["\n", "// Conversion functions:", "\n"]);

    generate_layout_impls(&mut out, inp, layout, in_module)?;

    for e in layout.nested_local_enums() {
        enums::generate_enum_impls(&mut out, inp, e, in_module)?;
    }

    for layout in layout.nested_local_layouts() {
        generate_layout_impls(&mut out, inp, layout, in_module)?;
    }

    out.pop_section();

    Ok(())
}

/// Generate a layout struct (which may possiblly serve double-duty
/// as a register struct).
pub fn generate_layout_struct(
    out: &mut dyn Write,
    inp: &Input,
    layout: &Layout,
    for_register: Option<&Register>,
    in_module: bool,
) -> Result<(), Error> {
    // Struct doc comment:
    writeln!(out)?;
    if let Some(reg) = for_register {
        writeln!(out, "/// `{}` Register", reg.name)?;
        writeln!(out, "///")?;
        writeln!(out, "/// Address: 0x{:X}", reg.adr)?;
        if let Some(reset_val) = reg.reset_val {
            writeln!(out, "/// Reset Value: 0x{:X}", reset_val)?;
        }
    } else {
        writeln!(out, "/// `{}`", layout.name)?;
    }
    if !layout.docs.is_empty() {
        writeln!(out, "///")?;
        write!(out, "{}", layout.docs.as_multiline("/// "))?;
    }
    if for_register.is_some() {
        writeln!(out, "///")?;
        writeln!(out, "/// Fields:")?;
        writeln!(out, "{}", rs_layout_overview_comment(layout, "/// "))?;
    }

    // Struct derives:
    if !inp.opts.struct_derive.is_empty() {
        let derives = inp.opts.struct_derive.join(", ");
        writeln!(out, "#[derive({derives})]")?;
    }

    // Struct proper:
    writeln!(out, "pub struct {} {{", rs_pascalcase(&layout.name))?;

    for field in layout.fields_with_content() {
        let field_type = register_layout_member_type(inp, field, in_module)?;
        let field_name = rs_snakecase(&field.name);
        generate_doc_comment(out, &field.docs, "    ")?;
        writeln!(out, "    pub {field_name}: {field_type},")?;
    }

    writeln!(out, "}}")?;

    Ok(())
}

/// Type of a field inside a register struct.
pub fn register_layout_member_type(inp: &Input, field: &LayoutField, in_module: bool) -> Result<String, Error> {
    match &field.accepts {
        FieldType::UInt => rs_fitting_unsigned_type(mask_width(field.mask)),
        FieldType::Bool => Ok("bool".to_string()),
        FieldType::Enum(e) => Ok(prefix_with_super(inp, &rs_pascalcase(&e.name), e.is_local, in_module)),
        FieldType::Layout(l) => Ok(prefix_with_super(inp, &rs_pascalcase(&l.name), l.is_local, in_module)),
        FieldType::Fixed(_) => panic!("Fixed layout field has no type"),
    }
}

pub fn generate_layout_impls(out: &mut dyn Write, inp: &Input, layout: &Layout, in_module: bool) -> Result<(), Error> {
    generate_layout_impl_to_bytes(inp, out, layout, in_module)?;
    generate_layout_impl_from_bytes(inp, out, layout, in_module)?;
    if inp.opts.generate_uint_conversion {
        generate_layout_impl_uint_conv(inp, out, layout, in_module)?;
    }
    Ok(())
}

pub fn generate_layout_impl_to_bytes(
    inp: &Input,
    out: &mut dyn Write,
    layout: &Layout,
    in_module: bool,
) -> Result<(), Error> {
    let struct_name = rs_pascalcase(&layout.name);
    let width_bytes = layout.width_bytes();
    let trait_prefix = trait_prefix(inp, in_module);

    let mut out = IndentWriter::new(out, "    ");

    // Impl block and function signature:
    writeln!(out)?;
    writeln!(out, "impl {trait_prefix}ToBytes<{width_bytes}> for {struct_name} {{")?;
    writeln!(out, "    #[allow(clippy::cast_possible_truncation)]")?;
    writeln!(out, "    fn to_le_bytes(&self) -> [u8; {width_bytes}] {{")?;

    if layout.fields.is_empty() {
        writeln!(out, "        [0; {width_bytes}]")?;
        writeln!(out, "    }}")?;
        writeln!(out, "}}")?;
        return Ok(());
    }

    out.increase_indent(2);

    // Variable to hold result:
    writeln!(out, "let mut val: [u8; {width_bytes}] = [0; {width_bytes}];")?;

    // Insert each field:
    for field in layout.fields.values() {
        let field_name = rs_snakecase(&field.name);

        writeln!(out, "// {} @ {struct_name}[{}]:", field.name, mask_to_bit_ranges_str(field.mask))?;

        match &field.accepts {
            FieldType::UInt | FieldType::Bool => {
                // Numeric field that can be directly converted:
                for byte in 0..width_bytes {
                    let Some(transform) = field_to_packed_byte_transform(
                        Endianess::Little,
                        unpositioned_mask(field.mask),
                        lsb_pos(field.mask),
                        byte,
                        width_bytes,
                    ) else {
                        continue;
                    };

                    // Convert the field to some unsigned integer that can be shifted:
                    let field_value = match &field.accepts {
                        FieldType::UInt => format!("self.{field_name}"),
                        FieldType::Bool => format!("u8::from(self.{field_name})"),
                        FieldType::Enum(_) => unreachable!(),
                        FieldType::Fixed(_) => unreachable!(),
                        FieldType::Layout(_) => unreachable!(),
                    };

                    // The byte of interest:
                    let field_byte = match &transform.shift {
                        Some((ShiftDirection::Left, amnt)) => format!("({field_value} << {amnt})"),
                        Some((ShiftDirection::Right, amnt)) => format!("({field_value} >> {amnt})"),
                        None => field_value,
                    };

                    let masked_field_byte = if transform.mask == 0xFF {
                        field_byte
                    } else {
                        format!("({field_byte} & 0x{:X})", transform.mask)
                    };

                    writeln!(out, "val[{byte}] |= {masked_field_byte} as u8;")?;
                }
            }

            FieldType::Fixed(fixed) => {
                // Fixed value:
                for byte in 0..width_bytes {
                    let mask_byte = grab_byte(Endianess::Little, field.mask, byte, width_bytes);
                    let value_byte = grab_byte(Endianess::Little, *fixed << lsb_pos(field.mask), byte, width_bytes);
                    if mask_byte == 0 {
                        continue;
                    };

                    writeln!(out, "val[{byte}] |= 0x{value_byte:x}; // Fixed value.")?;
                }
            }

            FieldType::Layout(_) | FieldType::Enum(_) => {
                // Sub-layout has to delegate to other pack function:
                let array_name = rs_snakecase(&field.name);
                let array_len = match &field.accepts {
                    FieldType::Enum(e) => e.min_width_bytes(),
                    FieldType::Layout(l) => l.width_bytes(),
                    _ => unreachable!(),
                };
                let occupied_mask = match &field.accepts {
                    FieldType::Enum(e) => e.occupied_bits(),
                    FieldType::Layout(l) => l.occupied_mask(),
                    _ => unreachable!(),
                };

                if let FieldType::Layout(l) = &field.accepts {
                    if l.fields.is_empty() {
                        writeln!(out, "// No fields.")?;
                        continue;
                    }
                }

                writeln!(out, "let {array_name}: [u8; {array_len}] = self.{field_name}.to_le_bytes();")?;

                for byte in 0..width_bytes {
                    for field_byte in 0..array_len {
                        // Determine required transform to put byte 'field_byte' of field into 'byte' of
                        // output:
                        let transform = field_byte_to_packed_byte_transform(
                            Endianess::Little,
                            occupied_mask,
                            lsb_pos(field.mask),
                            field_byte,
                            width_bytes,
                            byte,
                            width_bytes,
                        );

                        let Some(transform) = transform else {
                            continue;
                        };

                        let field_byte = format!("{array_name}[{field_byte}]");
                        let field_byte = match &transform.shift {
                            Some((ShiftDirection::Left, amnt)) => format!("({field_byte} << {amnt})"),
                            Some((ShiftDirection::Right, amnt)) => format!("({field_byte} >> {amnt})"),
                            None => field_byte,
                        };

                        let masked = if transform.mask != 0xFF {
                            format!("{field_byte} & 0x{:X}", transform.mask)
                        } else {
                            field_byte
                        };

                        writeln!(out, "val[{byte}] |= {masked};")?;
                    }
                }
            }
        }
    }

    // Return result:
    writeln!(out, "val")?;

    // End of impl block/signature:
    out.decrease_indent(2);
    writeln!(out, "    }}")?;
    writeln!(out, "}}")?;

    Ok(())
}

fn generate_layout_impl_from_bytes(
    inp: &Input,
    out: &mut dyn Write,
    layout: &Layout,
    in_module: bool,
) -> Result<(), Error> {
    let struct_name = rs_pascalcase(&layout.name);
    let width_bytes = layout.width_bytes();
    let trait_prefix = trait_prefix(inp, in_module);

    let error_type = if inp.opts.unpacking_error_msg {
        "&'static str"
    } else {
        "()"
    };

    let mut out = IndentWriter::new(out, "    ");

    // Prevent unused var warnings:
    let val_in_sig = if layout.fields_with_content().count() != 0 {
        "val"
    } else {
        "_val"
    };

    if layout.can_always_unpack() {
        writeln!(out)?;
        writeln!(out, "impl {trait_prefix}FromBytes<{width_bytes}> for {struct_name} {{")?;
        writeln!(out, "    fn from_le_bytes({val_in_sig}: &[u8; {width_bytes}]) -> Self {{")?;
        if !trait_prefix.is_empty() {
            writeln!(out, "        use {trait_prefix}FromBytes;")?;
            writeln!(out, "        use {trait_prefix}FromMaskedBytes;")?;
        }
    } else {
        writeln!(out)?;
        writeln!(out, "impl {trait_prefix}TryFromBytes<{width_bytes}> for {struct_name} {{")?;
        writeln!(out, "    type Error = {error_type};")?;
        writeln!(out, "    fn try_from_le_bytes({val_in_sig}: &[u8; {width_bytes}]) -> Result<Self, Self::Error> {{")?;
        if !trait_prefix.is_empty() {
            writeln!(out, "        use {trait_prefix}TryFromBytes;")?;
            writeln!(out, "        use {trait_prefix}FromBytes;")?;
            writeln!(out, "        use {trait_prefix}FromMaskedBytes;")?;
        }
    }

    out.increase_indent(2);

    // Sublayouts and enums require a bunch of array wrangling, which is done before the struct initialiser:
    for field in layout.fields_with_content() {
        let array_len = match &field.accepts {
            FieldType::Enum(e) => e.min_width_bytes(),
            FieldType::Layout(l) => l.width_bytes(),
            _ => continue,
        };
        let array_name = rs_snakecase(&field.name);
        let occupied_mask = match &field.accepts {
            FieldType::Enum(e) => e.occupied_bits(),
            FieldType::Layout(l) => l.occupied_mask(),
            _ => unreachable!(),
        };

        writeln!(out, "// {} @ {struct_name}[{}]:", field.name, mask_to_bit_ranges_str(field.mask))?;

        // Assemble field bytes into array:

        if let FieldType::Layout(l) = &field.accepts {
            if l.fields.is_empty() {
                writeln!(out, "let {array_name}: [u8; {array_len}] = [0; {array_len}];")?;
                continue;
            }
        }

        writeln!(out, "let mut {array_name}: [u8; {array_len}] = [0; {array_len}];")?;

        for byte in 0..width_bytes {
            for field_byte in 0..array_len {
                // Determine required transform to put byte 'byte' of packed input into 'field_byte' of
                // field:
                let transform = packed_byte_to_field_byte_transform(
                    Endianess::Little,
                    occupied_mask,
                    lsb_pos(field.mask),
                    field_byte,
                    array_len,
                    byte,
                    width_bytes,
                );

                let Some(transform) = transform else {
                    continue;
                };

                let masked = if transform.mask != 0xFF {
                    format!("(val[{byte}] & 0x{:X})", transform.mask)
                } else {
                    format!("val[{byte}]")
                };
                let shifted = match &transform.shift {
                    Some((ShiftDirection::Left, amnt)) => format!("{masked} << {amnt}"),
                    Some((ShiftDirection::Right, amnt)) => format!("{masked} >> {amnt}"),
                    None => masked,
                };

                writeln!(out, "{array_name}[{field_byte}] |= {};", remove_wrapping_parens(&shifted))?;
            }
        }
    }

    // Struct initialiser to return:
    if layout.can_always_unpack() {
        writeln!(out, "Self {{")?;
    } else {
        writeln!(out, "Ok(Self {{")?;
    }

    for field in layout.fields_with_content() {
        let field_name = rs_snakecase(&field.name);
        writeln!(out, "  // {} @ {struct_name}[{}]:", field.name, mask_to_bit_ranges_str(field.mask))?;

        match &field.accepts {
            FieldType::UInt => {
                // Numeric fields can be directly converted:
                let numeric_value = assemble_numeric_field(layout, field)?;
                writeln!(out, "  {field_name}: {numeric_value},")?;
            }
            FieldType::Bool => {
                // Bools require a simple conversion:
                let numeric_value = assemble_numeric_field(layout, field)?;
                writeln!(out, "  {field_name}: {numeric_value} != 0,")?;
            }
            FieldType::Enum(e) => {
                let enum_name = prefix_with_super(inp, &rs_pascalcase(&e.name), e.is_local, in_module);
                let array_name = rs_snakecase(&field.name);

                match enum_impl(e) {
                    FromBytesImpl::FromBytes => {
                        writeln!(out, "  {field_name}: {enum_name}::from_le_bytes(&{array_name}),")?;
                    }
                    FromBytesImpl::FromMaskedBytes => {
                        writeln!(out, "  {field_name}: {enum_name}::from_masked_le_bytes(&{array_name}),")?;
                    }
                    FromBytesImpl::TryFromBytes => {
                        writeln!(out, "  {field_name}: {enum_name}::try_from_le_bytes(&{array_name})?,")?;
                    }
                }
            }
            FieldType::Layout(l) => {
                let layout_name = prefix_with_super(inp, &rs_pascalcase(&l.name), l.is_local, in_module);
                let array_name = rs_snakecase(&field.name);
                if l.can_always_unpack() {
                    writeln!(out, "  {field_name}: {layout_name}::from_le_bytes(&{array_name}),")?;
                } else {
                    writeln!(out, "  {field_name}: {layout_name}::try_from_le_bytes(&{array_name})?,")?;
                }
            }
            FieldType::Fixed(_) => unreachable!(),
        }
    }

    out.decrease_indent(2);
    // Close struct, function and impl:
    if layout.can_always_unpack() {
        writeln!(out, "        }}")?;
    } else {
        writeln!(out, "        }})")?;
    }
    writeln!(out, "    }}")?;
    writeln!(out, "}}")?;

    Ok(())
}

fn generate_layout_impl_uint_conv(
    inp: &Input,
    out: &mut dyn Write,
    layout: &Layout,
    in_module: bool,
) -> Result<(), Error> {
    let struct_name = rs_pascalcase(&layout.name);
    let trait_prefix = trait_prefix(inp, in_module);

    let (uint_type, uint_width_bytes) = match layout.width_bytes() {
        1 => ("u8", 1),
        2 => ("u16", 2),
        3..=4 => ("u32", 4),
        5..=8 => ("u64", 8),
        9..=16 => ("u128", 16),
        _ => return Ok(()),
    };

    let mut out = IndentWriter::new(out, "    ");

    // Struct -> Bytes:

    writeln!(out)?;
    writeln!(out, "impl From<{struct_name}> for {uint_type} {{")?;
    writeln!(out, "    fn from(value: {struct_name}) -> Self {{")?;
    out.increase_indent(2);

    if !trait_prefix.is_empty() {
        writeln!(out, "use {trait_prefix}ToBytes;")?;
    }
    if uint_width_bytes == layout.width_bytes() {
        writeln!(out, "Self::from_le_bytes(value.to_le_bytes())")?;
    } else {
        writeln!(out, "let mut bytes = [0; {uint_width_bytes}];")?;
        writeln!(out, "bytes[0..{}].copy_from_slice(&value.to_le_bytes());", layout.width_bytes())?;
        writeln!(out, "Self::from_le_bytes(bytes)")?;
    }

    out.decrease_indent(2);
    writeln!(out, "    }}")?;
    writeln!(out, "}}")?;

    // Bytes -> Struct:

    if layout.can_always_unpack() {
        writeln!(out)?;
        writeln!(out, "impl From<{uint_type}> for {struct_name} {{")?;
        writeln!(out, "    fn from(value: {uint_type}) -> Self {{")?;
        if !trait_prefix.is_empty() {
            writeln!(out, "        use {trait_prefix}FromBytes;")?;
        }
        if uint_width_bytes == layout.width_bytes() {
            writeln!(out, "        Self::from_le_bytes(&value.to_le_bytes())")?;
        } else {
            writeln!(out, "        let mut bytes = [0; {}];", layout.width_bytes())?;
            writeln!(out, "        bytes.copy_from_slice(&(value.to_le_bytes()[0..{}]));", layout.width_bytes())?;
            writeln!(out, "        Self::from_le_bytes(bytes)")?;
        }
        writeln!(out, "    }}")?;
        writeln!(out, "}}")?;
    } else {
        writeln!(out)?;
        writeln!(out, "impl TryFrom<{uint_type}> for {struct_name} {{")?;
        if inp.opts.unpacking_error_msg {
            writeln!(out, "    type Error = &'static str;")?;
        } else {
            writeln!(out, "    type Error = ();")?;
        }
        writeln!(out, "    fn try_from(value: {uint_type}) -> Result<Self, Self::Error> {{")?;
        if !trait_prefix.is_empty() {
            writeln!(out, "        use {trait_prefix}TryFromBytes;")?;
        }
        if uint_width_bytes == layout.width_bytes() {
            writeln!(out, "        Self::try_from_le_bytes(&value.to_le_bytes())")?;
        } else {
            writeln!(out, "        let mut bytes = [0; {}];", layout.width_bytes())?;
            writeln!(out, "        bytes.copy_from_slice(&(value.to_le_bytes()[0..{}]));", layout.width_bytes())?;
            writeln!(out, "        Self::try_from_le_bytes(&bytes)")?;
        }
        writeln!(out, "    }}")?;
        writeln!(out, "}}")?;
    }
    Ok(())
}

fn assemble_numeric_field(layout: &Layout, field: &LayoutField) -> Result<String, Error> {
    let field_raw_type = match &field.accepts {
        FieldType::UInt => rs_fitting_unsigned_type(mask_width(field.mask))?,
        FieldType::Bool => "u8".to_string(),
        FieldType::Enum(e) => rs_fitting_unsigned_type(e.min_bitdwith())?,
        FieldType::Fixed(_) => unreachable!(),
        FieldType::Layout(_) => unreachable!(),
    };

    let mut unpacked_value_parts: Vec<String> = vec![];

    for byte in 0..layout.width_bytes() {
        let Some(transform) = packed_byte_to_field_transform(
            Endianess::Little,
            unpositioned_mask(field.mask),
            lsb_pos(field.mask),
            byte,
            layout.width_bytes(),
        ) else {
            continue;
        };

        let casted_value = if field_raw_type == "u8" {
            format!("val[{byte}]")
        } else {
            format!("{field_raw_type}::from(val[{byte}])")
        };

        let masked = if transform.mask == 0xFF {
            casted_value
        } else {
            format!("({casted_value} & 0x{:X})", transform.mask)
        };

        match &transform.shift {
            Some((ShiftDirection::Left, amnt)) => unpacked_value_parts.push(format!("{masked} << {amnt}")),
            Some((ShiftDirection::Right, amnt)) => unpacked_value_parts.push(format!("{masked} >> {amnt}")),
            None => unpacked_value_parts.push(masked),
        };
    }
    assert!(!unpacked_value_parts.is_empty());

    Ok(remove_wrapping_parens(&unpacked_value_parts.join(" | ")))
}
