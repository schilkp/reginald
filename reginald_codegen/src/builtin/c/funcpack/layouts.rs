use std::fmt::Write;

use reginald_utils::RangeStyle;

use crate::{
    error::Error,
    regmap::{Docs, FieldType, Layout, LayoutField},
    utils::{
        Endianess, ShiftDirection, field_byte_to_packed_byte_transform, field_to_packed_byte_transform, grab_byte,
        packed_byte_to_field_byte_transform,
    },
    writer::header_writer::HeaderWriter,
};

use super::{
    Element, Input, assemble_numeric_field, c_code, c_fitting_unsigned_type, c_generate_doxy_comment, c_macro, enums,
    func_prefix, is_enabled, swap_loop,
};

pub fn generate_layout(out: &mut dyn Write, inp: &Input, layout: &Layout) -> Result<(), Error> {
    let mut out = HeaderWriter::new(out);

    if layout.is_local {
        out.push_section_with_header(&["\n", "// Register-specific enums and sub-layouts:", "\n"]);
    } else {
        out.push_section_with_header(&["\n", "// Layout-specific enums and sub-layouts:", "\n"]);
    }

    for e in layout.nested_local_enums() {
        enums::generate_enum(&mut out, inp, e)?;
    }

    for local_layout in layout.nested_local_layouts() {
        generate_layout_struct(&mut out, inp, local_layout)?;
    }

    out.pop_section();

    if layout.is_local {
        out.push_section_with_header(&["\n", "// Register Layout Struct:", "\n"]);
    } else {
        out.push_section_with_header(&["\n", "// Layout Struct:", "\n"]);
    }
    generate_layout_struct(&mut out, inp, layout)?;
    out.pop_section();

    out.push_section_with_header(&["\n", "// Enum validation functions:", "\n"]);
    for e in layout.nested_local_enums() {
        enums::generate_enum_validation_macro(&mut out, inp, e)?;
    }
    out.pop_section();

    out.push_section_with_header(&["\n", "// Layout struct conversion functions:", "\n"]);
    for layout in layout.nested_local_layouts() {
        generate_layout_funcs(&mut out, inp, layout)?;
    }
    generate_layout_funcs(&mut out, inp, layout)?;
    out.pop_section();

    Ok(())
}

fn generate_layout_struct(out: &mut dyn Write, inp: &Input, layout: &Layout) -> Result<(), Error> {
    if !is_enabled(inp, Element::Structs) {
        return Ok(());
    }

    let struct_name = format!("{}_{}", c_code(&inp.map.name), c_code(&layout.name));

    // doxy comment
    writeln!(out)?;
    c_generate_doxy_comment(
        out,
        &layout.docs,
        "",
        vec![(
            String::from("note"),
            String::from("use pack/unpack functions for conversion to/from packed binary value"),
        )],
    )?;

    // Struct proper:
    writeln!(out, "struct {struct_name} {{")?;
    for field in layout.fields_with_content() {
        let field_type = struct_field_type(inp, field)?;
        let field_name = c_code(&field.name);
        c_generate_doxy_comment(out, &field.docs, "  ", vec![])?;

        // Members are bitifields, if configured:
        let bitfield_str = if inp.opts.registers_as_bitfields {
            match &field.accepts {
                FieldType::Enum(_) | FieldType::Bool | FieldType::UInt => {
                    format!(": {}", field.bits.width())
                }
                FieldType::Layout(_) => String::new(),
                FieldType::Fixed(_) => unreachable!(),
            }
        } else {
            String::new()
        };
        writeln!(out, "  {field_type} {field_name}{bitfield_str};",)?;
    }
    if layout.fields_with_content().count() == 0 {
        writeln!(out, "  int dummy; // Register contains no variable fields.",)?;
    }

    writeln!(out, "}};")?;
    Ok(())
}

/// Generate register packing/unpacking funcs
fn generate_layout_funcs(out: &mut dyn Write, inp: &Input, layout: &Layout) -> Result<(), Error> {
    if !is_enabled(inp, Element::StructConversionFuncs) {
        return Ok(());
    }

    for endian in &inp.endian {
        generate_layout_pack_func(out, inp, layout, *endian)?;
    }
    for endian in &inp.endian {
        generate_layout_unpack_func(out, inp, layout, *endian)?;
    }
    generate_layout_validation_func(out, inp, layout)?;
    for endian in &inp.endian {
        generate_layout_try_unpack_func(out, inp, layout, *endian)?;
    }

    Ok(())
}

fn generate_layout_pack_func(
    out: &mut dyn Write,
    inp: &Input,
    layout: &Layout,
    endian: Endianess,
) -> Result<(), Error> {
    // Strings/Properties:
    let code_prefix = c_code(&inp.map.name);
    let code_name = c_code(&layout.name);
    let func_prefix = func_prefix(inp);
    let width_bytes = layout.width_bytes();

    // Doxy comment:
    writeln!(out)?;
    let docs = Docs {
        brief: Some(format!("Convert @ref struct {code_prefix}_{code_name} struct to packed {endian} value.")),
        doc: None,
    };

    c_generate_doxy_comment(
        out,
        &docs,
        "",
        vec![(
            String::from("note"),
            String::from("use pack/unpack functions for conversion to/from packed binary value"),
        )],
    )?;

    // Function:
    let func_sig = format!(
        "{}void {}_{}_pack_{}(const struct {}_{} *r, uint8_t val[{}])",
        func_prefix,
        code_prefix,
        code_name,
        endian.short(),
        code_prefix,
        code_name,
        width_bytes
    );

    if inp.opts.funcs_as_prototypes {
        writeln!(out, "{func_sig};")?;
        return Ok(());
    }

    writeln!(out, "{func_sig} {{")?;

    if let Some(defer_to) = inp.opts.defer_to_endian.filter(|x| *x != endian) {
        // The implementaiton for this endianess defers to the other endianess:
        let defer_arr = format!("val_{}", defer_to.short());
        writeln!(out, "  uint8_t {defer_arr}[{width_bytes}] = {{0}};")?;
        writeln!(out, "  {}", swap_loop("val", &defer_arr, width_bytes))?;
        writeln!(out, "  {}_{}_pack_{}(r, {defer_arr});", code_prefix, code_name, defer_to.short())?;
        writeln!(out, "  {}", swap_loop(&defer_arr, "val", width_bytes))?;
        writeln!(out, "}}")?;
        return Ok(());
    }

    // Pack each field:
    for field in layout.fields.values() {
        let field_name = c_code(&field.name);

        let bit_str = field.bits.to_string(RangeStyle::Verilog);
        writeln!(out, "  // {} @ {code_name}[{bit_str}]:", field.name)?;

        match &field.accepts {
            FieldType::UInt | FieldType::Bool | FieldType::Enum(_) => {
                // Numeric field that can be directly converted:
                for byte in 0..width_bytes {
                    let Some(transform) = field_to_packed_byte_transform(
                        endian,
                        field.bits.unpositioned_mask(),
                        field.bits.lsb_pos(),
                        byte,
                        width_bytes,
                    ) else {
                        continue;
                    };

                    let field_byte = match &transform.shift {
                        Some((ShiftDirection::Left, amnt)) => format!("(r->{field_name} << {amnt})"),
                        Some((ShiftDirection::Right, amnt)) => format!("(r->{field_name} >> {amnt})"),
                        None => format!("r->{field_name}"),
                    };

                    writeln!(out, "  val[{byte}] &= (uint8_t)~0x{:X}U;", transform.mask)?;
                    writeln!(out, "  val[{byte}] |= (uint8_t)(((uint8_t){field_byte}) & 0x{:X}U);", transform.mask)?;
                }
            }

            FieldType::Fixed(fixed) => {
                // Fixed value:
                for byte in 0..width_bytes {
                    let mask_byte = grab_byte(endian, field.bits.mask(), byte, width_bytes);
                    let value_byte = grab_byte(endian, *fixed << field.bits.lsb_pos(), byte, width_bytes);
                    if mask_byte == 0 {
                        continue;
                    };

                    writeln!(out, "  val[{byte}] &= (uint8_t)~0x{:X}U;", mask_byte)?;
                    writeln!(out, "  val[{byte}] |= (uint8_t)0x{value_byte:x}; // Fixed value.")?;
                }
            }

            FieldType::Layout(sublayout) => {
                // Sub-layout has to delegate to other pack function:

                let array_name = c_code(&field.name);
                let array_len = sublayout.width_bytes();
                let code_sublayout_name = c_code(&sublayout.name);
                let function_prefix = format!("{code_prefix}_{code_sublayout_name}");

                writeln!(out, "  uint8_t {array_name}[{array_len}] = {{0}};")?;
                writeln!(
                    out,
                    "  {function_prefix}_pack_{}(&r->{field_name}, {});",
                    endian.short(),
                    c_code(&field.name)
                )?;

                for byte in 0..width_bytes {
                    for field_byte in 0..array_len {
                        // Determine required transform to put byte 'field_byte' of field into 'byte' of
                        // output:
                        let transform = field_byte_to_packed_byte_transform(
                            endian,
                            sublayout.occupied_mask(),
                            field.bits.lsb_pos(),
                            field_byte,
                            sublayout.width_bytes(),
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

                        writeln!(out, "  val[{byte}] &= (uint8_t)~0x{:X}U;", transform.mask)?;
                        writeln!(out, "  val[{byte}] |= (uint8_t)((uint8_t){field_byte} & 0x{:X}U);", transform.mask)?;
                    }
                }
            }
        }
    }

    // Prevent unused args warnings:
    if layout.fields.is_empty() {
        writeln!(out, "  (void)val;")?;
    }
    if layout.fields_with_content().count() == 0 {
        writeln!(out, "  (void)r;")?;
    }

    writeln!(out, "}}",)?;

    Ok(())
}

fn generate_layout_unpack_func(
    out: &mut dyn Write,
    inp: &Input,
    layout: &Layout,
    endian: Endianess,
) -> Result<(), Error> {
    // Strings:
    let struct_name = format!("{}_{}", c_code(&inp.map.name), c_code(&layout.name));
    let func_prefix = func_prefix(inp);
    let code_prefix = c_code(&inp.map.name);
    let code_name = c_code(&layout.name);

    let width_bytes = layout.width_bytes();

    // doxy comment:
    writeln!(out)?;
    let docs = Docs {
        brief: Some(format!("Convert packed {endian} binary value to struct.")),
        doc: None,
    };
    c_generate_doxy_comment(out, &docs, "", vec![])?;

    // Function signature
    let func_sig = format!(
        "{}struct {} {}_unpack_{}(const uint8_t val[{}])",
        func_prefix,
        struct_name,
        struct_name,
        endian.short(),
        width_bytes,
    );

    if inp.opts.funcs_as_prototypes {
        writeln!(out, "{func_sig};")?;
        return Ok(());
    }

    writeln!(out, "{func_sig} {{")?;

    if let Some(defer_to) = inp.opts.defer_to_endian.filter(|x| *x != endian) {
        // The implementaiton for this endianess defers to the other endianess:
        let defer_arr = format!("val_{}", defer_to.short());
        writeln!(out, "  uint8_t {defer_arr}[{width_bytes}] = {{0}};")?;
        writeln!(out, "  {}", swap_loop("val", &defer_arr, width_bytes))?;
        writeln!(out, "  return {}_unpack_{}({defer_arr});", struct_name, defer_to.short())?;
        writeln!(out, "}}")?;
        return Ok(());
    }

    writeln!(out, "  struct {struct_name} r = {{0}};")?;

    // Unpack each field:
    for field in layout.fields_with_content() {
        let code_field_name = c_code(&field.name);

        let bit_str = field.bits.to_string(RangeStyle::Verilog);
        writeln!(out, "  // {} @ {code_name}[{bit_str}]:", field.name)?;

        match &field.accepts {
            FieldType::UInt | FieldType::Bool => {
                // Numeric fields can be directly converted:
                let numeric_value = assemble_numeric_field(layout, field, endian)?;

                writeln!(out, "  r.{code_field_name} = {numeric_value};")?;
            }
            FieldType::Enum(e) => {
                // Enums may need different casting:
                let enum_name = c_code(&e.name);
                let numeric_value = assemble_numeric_field(layout, field, endian)?;

                if field.bits.width() <= inp.opts.max_enum_bitwidth {
                    writeln!(out, "  r.{code_field_name} = (enum {code_prefix}_{enum_name})({numeric_value});")?;
                } else {
                    writeln!(out, "  r.{code_field_name} = {numeric_value};")?;
                }
            }
            FieldType::Layout(sublayout) => {
                // Sub-layout has to delegate to other unpack function:
                let array_len = sublayout.width_bytes();
                let code_sublayout_name = c_code(&sublayout.name);

                // Array to contain unpacked/binary sublayout:
                writeln!(out, "  uint8_t {code_field_name}[{array_len}] = {{0}};")?;

                for byte in 0..width_bytes {
                    for field_byte in 0..array_len {
                        // Determine required transform to put byte 'byte' of packed input into 'field_byte' of
                        // field:
                        let transform = packed_byte_to_field_byte_transform(
                            endian,
                            sublayout.occupied_mask(),
                            field.bits.lsb_pos(),
                            field_byte,
                            array_len,
                            byte,
                            width_bytes,
                        );

                        let Some(transform) = transform else {
                            continue;
                        };

                        let masked = format!("(val[{byte}] & 0x{:X}U)", transform.mask);
                        let shifted = match &transform.shift {
                            Some((ShiftDirection::Left, amnt)) => format!("{masked} << {amnt}"),
                            Some((ShiftDirection::Right, amnt)) => format!("{masked} >> {amnt}"),
                            None => masked,
                        };

                        writeln!(out, "  {code_field_name}[{field_byte}] |= (uint8_t)({shifted});")?;
                    }
                }

                let function_prefix = format!("{code_prefix}_{code_sublayout_name}");
                writeln!(
                    out,
                    "  r.{code_field_name} = {function_prefix}_unpack_{}({code_field_name});",
                    endian.short()
                )?;
            }
            FieldType::Fixed(_) => unreachable!(),
        }
    }

    // Prevent unused args warnings:
    if layout.fields_with_content().count() == 0 {
        writeln!(out, "  (void)val;")?;
        writeln!(out, "  (void)r;")?;
    }

    writeln!(out, "  return r;")?;
    writeln!(out, "}}")?;
    Ok(())
}

fn generate_layout_validation_func(out: &mut dyn Write, inp: &Input, layout: &Layout) -> Result<(), Error> {
    // Strings:
    let code_prefix = c_code(&inp.map.name);
    let code_name = c_code(&layout.name);
    let struct_name = format!("{code_prefix}_{code_name}");
    let func_prefix = func_prefix(inp);
    let macro_prefix = c_macro(&inp.map.name);

    // doxy comment:
    writeln!(out)?;
    let docs = Docs {
        brief: Some("Validate struct".to_string()),
        doc: Some("Confirms that all enums are valid, and all values fit into respective fields".to_string()),
    };
    c_generate_doxy_comment(
        out,
        &docs,
        "",
        vec![
            (String::from("returns"), String::from("0 if valid.")),
            (String::from("returns"), String::from("the position of the first invalid field if invalid.")),
        ],
    )?;

    // Function signature
    let func_sig =
        format!("{}int {}_validate_{}(const struct {} *r)", func_prefix, code_prefix, code_name, struct_name);

    if inp.opts.funcs_as_prototypes {
        writeln!(out, "{func_sig};")?;
        return Ok(());
    }
    writeln!(out, "{func_sig} {{")?;
    for field in layout.fields_with_content() {
        let error_code = field.bits.lsb_pos() + 1;
        let field_name = c_code(&field.name);
        let uint_type = c_fitting_unsigned_type(field.bits.width())?;
        let unpos_mask = field.bits.unpositioned_mask();

        match &field.accepts {
            FieldType::UInt => {
                writeln!(out, "  if ((r->{field_name} & ~({uint_type})0x{unpos_mask:X}) != 0) return {error_code};")?;
            }
            FieldType::Enum(e) => {
                let macro_name = c_macro(&e.name);

                writeln!(out, "  if (!({macro_prefix}_IS_VALID_{macro_name}(r->{field_name}))) return {error_code};")?;
            }
            FieldType::Layout(l) => {
                let layout_name = c_code(&l.name);
                writeln!(out, "  if ({code_prefix}_validate_{layout_name}(&r->{field_name})) return {error_code};")?;
            }
            FieldType::Bool => continue,
            FieldType::Fixed(_) => unreachable!(),
        }
    }

    // Prevent unused args warnings:
    if layout.fields_with_content().count() == 0 {
        writeln!(out, "  (void)r;")?;
    }

    writeln!(out, "  return 0;")?;
    writeln!(out, "}}")?;
    Ok(())
}

fn generate_layout_try_unpack_func(
    out: &mut dyn Write,
    inp: &Input,
    layout: &Layout,
    endian: Endianess,
) -> Result<(), Error> {
    // Strings:
    let struct_name = format!("{}_{}", c_code(&inp.map.name), c_code(&layout.name));
    let func_prefix = func_prefix(inp);
    let code_prefix = c_code(&inp.map.name);
    let code_name = c_code(&layout.name);

    let width_bytes = layout.width_bytes();

    // doxy comment:
    writeln!(out)?;
    let docs = Docs {
        brief: Some(format!("Attempt to convert packed {endian} binary value to struct.")),
        doc: None,
    };
    c_generate_doxy_comment(
        out,
        &docs,
        "",
        vec![
            (String::from("returns"), String::from("0 if valid.")),
            (String::from("returns"), String::from("the position of the first invalid field if invalid.")),
        ],
    )?;

    // Function signature
    let func_sig = format!(
        "{}int {}_try_unpack_{}(const uint8_t val[{}], struct {} *r)",
        func_prefix,
        struct_name,
        endian.short(),
        width_bytes,
        struct_name
    );

    if inp.opts.funcs_as_prototypes {
        writeln!(out, "{func_sig};")?;
        return Ok(());
    }

    writeln!(out, "{func_sig} {{")?;
    writeln!(out, "  *r = {struct_name}_unpack_{}(val);", endian.short())?;
    writeln!(out, "  return {code_prefix}_validate_{code_name}(r);")?;
    writeln!(out, "}}")?;
    Ok(())
}

fn struct_field_type(inp: &Input, field: &LayoutField) -> Result<String, Error> {
    let code_prefix = c_code(&inp.map.name);
    let field_width = field.bits.width();

    Ok(match &field.accepts {
        FieldType::Enum(e) => {
            let name = c_code(&e.name);
            if field_width <= inp.opts.max_enum_bitwidth {
                format!("enum {code_prefix}_{name}")
            } else {
                c_fitting_unsigned_type(field_width)?
            }
        }
        FieldType::UInt => c_fitting_unsigned_type(field_width)?,
        FieldType::Bool => "bool".to_string(),
        FieldType::Layout(layout) => {
            let name = c_code(&layout.name);
            format!("struct {code_prefix}_{name}")
        }
        FieldType::Fixed(_) => panic!("Fixed field has no struct type"),
    })
}
