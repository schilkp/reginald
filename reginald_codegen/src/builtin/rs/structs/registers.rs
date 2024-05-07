use std::fmt::Write;

use super::*;

use crate::{builtin::rs::array_literal, error::Error, utils::Endianess, writer::indent_writer::IndentWriter};

use super::rs_pascalcase;

pub fn generate_register(out: &mut dyn Write, inp: &Input, register: &Register) -> Result<(), Error> {
    writeln!(out)?;

    rs_generate_header_comment(out, &format!("`{}` Register", register.name))?;

    if register.layout.is_local && register.from_block.is_none() {
        // If the layout is local to this register, generate it and associate all properties to it:
        layouts::generate_layout(out, inp, &register.layout, &LayoutStructKind::RegisterLayout(register))?;
        generate_register_impl(out, inp, register, false)?;
    } else {
        // Otherwise generate a newtype to contain the register properties:
        generate_register_newtype(out, inp, register)?;
        generate_register_impl(out, inp, register, true)?;
    }

    Ok(())
}

pub fn generate_register_newtype(out: &mut dyn Write, inp: &Input, register: &Register) -> Result<(), Error> {
    // Struct doc comment:
    writeln!(out)?;
    writeln!(out, "/// `{}` Register", register.name)?;
    writeln!(out, "///")?;
    writeln!(out, "/// Address: 0x{:X}", register.adr)?;
    if let Some(reset_val) = register.reset_val {
        writeln!(out, "///")?;
        writeln!(out, "/// Reset Value: 0x{:X}", reset_val)?;
    }
    writeln!(out, "///")?;
    writeln!(out, "/// Uses [`{}`] layout.", rs_pascalcase(&register.layout.name))?;

    // Register derives:
    if !inp.opts.struct_derive.is_empty() {
        let derives = inp.opts.struct_derive.join(", ");
        writeln!(out, "#[derive({derives})]")?;
    }

    let layout_name = rs_pascalcase(&register.layout.name);

    // Struct proper:
    writeln!(out, "pub struct {} (pub {layout_name});", rs_pascalcase(&register.name))?;

    Ok(())
}

pub fn generate_register_impl(
    out: &mut dyn Write,
    inp: &Input,
    register: &Register,
    is_newtype: bool,
) -> Result<(), Error> {
    let reg_name = &register.name;
    let struct_name = rs_pascalcase(reg_name);
    let byte_width = register.layout.width_bytes();
    let address_type = &inp.address_type;
    let trait_prefix = trait_prefix(inp);

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

        if generate_reset_val(&mut init, &register.layout, *reset_val, is_newtype, true).is_err() {
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
    writeln!(out)?;

    rs_generate_header_comment(out, &format!("`{}` Register Block", block.name))?;
    generate_register_block_header(out, block, "//")?;

    // Shared register block properties
    generate_register_block_properties(out, inp, block)?;

    // Members:
    for member in block.members.values() {
        writeln!(out)?;
        rs_generate_header_comment(out, &format!("`{}` Register Block Member `{}`", &block.name, &member.name))?;

        if member.layout.is_local {
            layouts::generate_layout(out, inp, &member.layout, &LayoutStructKind::RegisterBlockMemberStruct(member))?;
        }
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
    out: &mut IndentWriter,
    layout: &Layout,
    val: TypeValue,
    is_newtype: bool,
    is_top: bool,
) -> Result<(), Error> {
    if !is_newtype && is_top {
        writeln!(out, "Self {{")?;
    } else {
        writeln!(out, "{} {{", rs_pascalcase(&layout.name))?;
    }

    out.push_indent();

    for field in layout.fields_with_content() {
        let field_val = (val & field.mask) >> (lsb_pos(field.mask));

        write!(out, "{}: ", rs_snakecase(&field.name))?;

        if let FieldType::Layout(layout) = &field.accepts {
            generate_reset_val(out, layout, field_val, false, false)?;
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

                    let enum_name = rs_pascalcase(&e.name);
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
