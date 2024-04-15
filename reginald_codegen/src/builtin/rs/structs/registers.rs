use std::fmt::Write;

use super::*;

use crate::{builtin::rs::array_literal, error::Error, utils::Endianess, writer::indent_writer::IndentWriter};

use super::rs_pascalcase;

pub fn generate_register(out: &mut dyn Write, inp: &Input, register: &Register) -> Result<(), Error> {
    let mut out = IndentWriter::new(out, "    ");

    writeln!(&mut out)?;

    // If seperate modules are enabled, generate module doc comment
    // and open module:
    if inp.opts.split_into_modules {
        writeln!(out, "/// `{}` Register", register.name)?;
        generate_register_header(&mut out, register, "///")?;
        writeln!(&mut out, "pub mod {} {{", rs_snakecase(&register.name))?;
        out.push_indent();
    } else {
        rs_generate_header_comment(&mut out, &format!("`{}` Register", register.name))?;
        generate_register_header(&mut out, register, "//")?;
    }

    // Track if we are currently in a sub-module
    let in_module = inp.opts.split_into_modules;

    if register.layout.is_local {
        // If the layout is local to this register, generate it and associate all properties to it:
        // Layout struct:
        generate_register_struct(&mut out, inp, register, in_module)?;
    } else {
        // Otherwise generate a newtype to contain the register properties:
        generate_register_newtype(&mut out, inp, register, in_module)?;
        generate_register_impl(&mut out, inp, register, true, in_module)?;
    }

    // Close module if opened.
    if inp.opts.split_into_modules {
        out.pop_indent();
        writeln!(&mut out, "}}")?;
    }

    Ok(())
}

pub fn generate_register_struct(
    out: &mut dyn Write,
    inp: &Input,
    register: &Register,
    in_module: bool,
) -> Result<(), Error> {
    let mut out = HeaderWriter::new(out);

    // If the layout is local to this register, generate it and associate all properties to it:
    // Layout struct:
    layouts::generate_layout_struct(&mut out, inp, &register.layout, Some(register), in_module)?;

    generate_register_impl(&mut out, inp, register, false, in_module)?;

    out.push_section_with_header(&["\n", "// Register-specific enums:", "\n"]);
    for e in register.layout.nested_local_enums() {
        enums::generate_enum(&mut out, inp, e)?;
    }
    out.pop_section();

    out.push_section_with_header(&["\n", "// Register-specific sub-layouts:", "\n"]);
    for local_layout in register.layout.nested_local_layouts() {
        layouts::generate_layout_struct(&mut out, inp, local_layout, None, in_module)?;
    }
    out.pop_section();

    out.push_section_with_header(&["\n", "// Conversion functions:", "\n"]);
    layouts::generate_layout_impls(&mut out, inp, &register.layout, in_module)?;
    for e in register.layout.nested_local_enums() {
        enums::generate_enum_impls(&mut out, inp, e, in_module)?;
    }
    for layout in register.layout.nested_local_layouts() {
        layouts::generate_layout_impls(&mut out, inp, layout, in_module)?;
    }
    out.pop_section();

    Ok(())
}

pub fn generate_register_newtype(
    out: &mut dyn Write,
    inp: &Input,
    register: &Register,
    in_module: bool,
) -> Result<(), Error> {
    // Struct doc comment:
    writeln!(out)?;
    writeln!(out, "/// `{}` Register", register.name)?;
    writeln!(out, "///")?;
    writeln!(out, "/// Address: 0x{:X}", register.adr)?;
    if !register.docs.is_empty() {
        writeln!(out, "///")?;
        write!(out, "{}", register.docs.as_multiline("/// "))?;
    }
    writeln!(out, "///")?;
    writeln!(out, "/// Uses `{}` layout.", rs_pascalcase(&register.layout.name))?;

    // Register derives:
    if !inp.opts.struct_derive.is_empty() {
        let derives = inp.opts.struct_derive.join(", ");
        writeln!(out, "#[derive({derives})]")?;
    }

    let layout_name =
        prefix_with_super(inp, &rs_pascalcase(&register.layout.name), register.layout.is_local, in_module);

    // Struct proper:
    writeln!(out, "pub struct {} ({layout_name});", rs_pascalcase(&register.name))?;

    Ok(())
}

pub fn generate_register_header(out: &mut dyn Write, register: &Register, comment_str: &str) -> Result<(), Error> {
    writeln!(out, "{comment_str} `{}` Register", register.name)?;
    writeln!(out, "{comment_str}")?;
    writeln!(out, "{comment_str} Address: 0x{:X}", register.adr)?;
    if let Some(reset_val) = register.reset_val {
        writeln!(out, "{comment_str} Value: 0x{:X}", reset_val)?;
    }
    if !register.docs.is_empty() {
        writeln!(out, "{comment_str}")?;
        write!(out, "{}", register.docs.as_multiline(&(comment_str.to_string() + " ")))?;
    }
    Ok(())
}

pub fn generate_register_impl(
    out: &mut dyn Write,
    inp: &Input,
    register: &Register,
    is_newtype: bool,
    in_module: bool,
) -> Result<(), Error> {
    let reg_name = &register.name;
    let struct_name = rs_pascalcase(reg_name);
    let byte_width = register.layout.width_bytes();
    let address_type = &inp.address_type;
    let trait_prefix = trait_prefix(inp, in_module);

    // ==== Properties ====:
    writeln!(out)?;
    writeln!(out, "/// Register Properties")?;
    writeln!(out, "impl {trait_prefix}Register<{byte_width}, {address_type}> for {struct_name} {{")?;

    // Adr:
    writeln!(out, "    const ADDRESS: {address_type} = 0x{:X};", register.adr)?;

    // Reset val:
    if let Some(reset_val) = &register.reset_val {
        let val = array_literal(Endianess::Little, *reset_val, byte_width);
        writeln!(
                out,
                "    const RESET_VAL: Option<{trait_prefix}ResetVal<{byte_width}>> = Some({trait_prefix}ResetVal::LittleEndian({val}));"
            )?;
    } else {
        writeln!(out, "    const RESET_VAL: Option<{trait_prefix}ResetVal<{byte_width}>> = None;")?;
    }
    writeln!(out, "}}")?;

    // ==== Default ====:
    if let Some(reset_val) = &register.reset_val {
        let mut out = IndentWriter::new(out, "    ");

        let mut init_str = String::new();
        let mut init = IndentWriter::new(&mut init_str, "    ");

        if generate_reset_val(inp, &mut init, &register.layout, *reset_val, is_newtype, in_module, true).is_err() {
            return Ok(());
        };
        drop(init);

        writeln!(out)?;
        writeln!(out, "/// Reset Value")?;
        writeln!(out, "impl Default for {struct_name} {{")?;
        writeln!(out, "    fn default() -> Self {{")?;
        out.increase_indent(2);
        if is_newtype {
            writeln!(out, "Self ({init_str})")?;
        } else {
            writeln!(out, "{}", init_str)?;
        }
        out.decrease_indent(2);
        writeln!(out, "    }}")?;
        writeln!(out, "}}")?;
    }

    Ok(())
}

pub fn generate_register_block(out: &mut dyn Write, inp: &Input, block: &RegisterBlock) -> Result<(), Error> {
    let mut out = IndentWriter::new(out, "    ");

    writeln!(&mut out)?;

    // If seperate modules are enabled, generate module doc comment
    // and open module:
    if inp.opts.split_into_modules {
        writeln!(out, "/// `{}` Register Block", block.name)?;
        generate_register_block_header(&mut out, block, "///")?;
        writeln!(&mut out, "pub mod {} {{", rs_snakecase(&block.name))?;
        out.push_indent();
    } else {
        rs_generate_header_comment(&mut out, &format!("`{}` Register Block", block.name))?;
        generate_register_block_header(&mut out, block, "//")?;
    }

    // Track if we are currently in a sub-module
    let in_module = inp.opts.split_into_modules;

    // Shared register block properties
    generate_register_block_properties(&mut out, inp, block)?;

    // Members:
    for member in block.members.values() {
        writeln!(&mut out)?;
        rs_generate_header_comment(&mut out, &format!("`{}` Register Block Member `{}`", &block.name, &member.name))?;

        if member.layout.is_local {
            layouts::generate_layout(&mut out, inp, &member.layout, in_module)?;
        }

        if !block.instances.is_empty() {
            writeln!(&mut out)?;
            writeln!(&mut out, "// Instances:")?;
            for block_instance in block.instances.values() {
                let member_instance = &block_instance.registers[&member.name];
                generate_register_newtype(&mut out, inp, member_instance, in_module)?;
                generate_register_impl(&mut out, inp, member_instance, true, in_module)?;
            }
        }
    }

    // Close module if opened.
    if inp.opts.split_into_modules {
        out.pop_indent();
        writeln!(&mut out, "}}")?;
    }

    Ok(())
}

pub fn generate_register_block_header(
    out: &mut dyn Write,
    block: &RegisterBlock,
    comment_str: &str,
) -> Result<(), Error> {
    if !block.docs.is_empty() {
        writeln!(out, "{comment_str}")?;
        write!(out, "{}", block.docs.as_multiline(&(comment_str.to_string() + " ")))?;
    }

    if !block.members.is_empty() {
        writeln!(out, "{comment_str}")?;
        writeln!(out, "{comment_str} Contains registers:")?;
        for member in block.members.values() {
            if let Some(brief) = &member.docs.brief {
                writeln!(out, "{comment_str} - `0x{:02}` `{}`: {}", member.offset, member.name, brief)?;
            } else {
                writeln!(out, "{comment_str} - `0x{:02}` `{}`", member.offset, member.name)?;
            }
        }
    }

    if !block.instances.is_empty() {
        writeln!(out, "{comment_str}")?;
        writeln!(out, "{comment_str} Instances:")?;
        for instance in block.instances.values() {
            if let Some(brief) = &instance.docs.brief {
                writeln!(out, "{comment_str} - `0x{:02}` `{}`: {}", instance.adr, instance.name, brief)?;
            } else {
                writeln!(out, "{comment_str} - `0x{:02}` `{}`", instance.adr, instance.name)?;
            }
        }
    }

    Ok(())
}

pub fn generate_register_block_properties(
    out: &mut dyn Write,
    inp: &Input,
    block: &RegisterBlock,
) -> Result<(), Error> {
    let address_type = &inp.address_type;
    let block_name = &block.name;
    let const_block_name = rs_const(block_name);

    if !block.members.is_empty() {
        writeln!(out)?;
        writeln!(out, "// Contained registers:")?;
        for member in block.members.values() {
            let reg_name = &member.name;
            let const_reg_name = rs_const(reg_name);

            writeln!(out)?;
            writeln!(out, "/// Offset of `{reg_name}` register from `{block_name}` block start")?;
            writeln!(out, "pub const {const_reg_name}_OFFSET: {address_type} = 0x{:x};", member.offset)?;
        }
    }

    if !block.instances.is_empty() {
        writeln!(out)?;
        writeln!(out, "// Instances:")?;
        for instance in block.instances.values() {
            let instance_name = &instance.name;
            let const_instance_name = rs_const(instance_name);
            writeln!(out)?;
            writeln!(out, "/// Start of `{block_name}` instance `{instance_name}`")?;
            writeln!(
                out,
                "pub const {const_block_name}_INSTANCE_{const_instance_name}: {address_type} = 0x{:x};",
                instance.adr
            )?;
        }
    }

    Ok(())
}

fn generate_reset_val(
    inp: &Input,
    out: &mut IndentWriter,
    layout: &Layout,
    val: TypeValue,
    is_newtype: bool,
    in_module: bool,
    is_top: bool,
) -> Result<(), Error> {
    if !is_newtype && is_top {
        writeln!(out, "Self {{")?;
    } else {
        writeln!(out, "{} {{", prefix_with_super(inp, &rs_pascalcase(&layout.name), layout.is_local, in_module))?;
    }

    out.push_indent();

    for field in layout.fields_with_content() {
        let field_val = (val & field.mask) >> (lsb_pos(field.mask));

        write!(out, "{}: ", rs_snakecase(&field.name))?;

        if let FieldType::Layout(layout) = &field.accepts {
            generate_reset_val(inp, out, layout, field_val, false, in_module, false)?;
        } else {
            match field.decode_value(field_val)? {
                crate::regmap::DecodedField::UInt(val) => {
                    write!(out, "0x{:X}", val)?;
                }
                crate::regmap::DecodedField::Bool(b) => {
                    write!(out, "{}", b)?;
                }
                crate::regmap::DecodedField::EnumEntry(entry) => {
                    let FieldType::Enum(e) = &field.accepts else {
                        unreachable!()
                    };

                    // If this is a newtyp struct, the enum is defined with the layout, not with this
                    // newtype.
                    let is_local = !is_newtype && e.is_local;

                    let enum_name = prefix_with_super(inp, &rs_pascalcase(&e.name), is_local, in_module);
                    write!(out, "{}::{}", enum_name, rs_pascalcase(&entry))?;
                }
                crate::regmap::DecodedField::Fixed { .. } => unreachable!(),
            };
        }

        writeln!(out, ",")?;
    }

    out.pop_indent();
    write!(out, "}}")?;
    Ok(())
}
