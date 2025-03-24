use std::fmt::Write;

use reginald_utils::str_table;

use crate::{
    error::Error,
    regmap::{Register, RegisterBlock, RegisterBlockMember},
    writer::header_writer::HeaderWriter,
};

use super::{
    Element, Input, c_code, c_generate_header_comment, c_generate_section_header_comment, c_layout_overview_comment,
    c_macro, layouts, to_array_init,
};

/// Generate register section header comment
pub fn generate_register(out: &mut dyn Write, inp: &Input, register: &Register) -> Result<(), Error> {
    let mut out = HeaderWriter::new(out);

    let mut header = String::new();
    generate_register_header(&mut header, inp, register)?;
    out.push_section_with_header(&[&header]);

    generate_register_properties(&mut out, inp, register)?;

    // If the layout is local to this register, generate it:
    if register.layout.is_local {
        layouts::generate_layout(&mut out, inp, &register.layout)?;
    } else if inp.opts.is_enabled(Element::Structs) {
        writeln!(&mut out)?;
        writeln!(
            out,
            "// Register uses the {}_{} struct and conversion funcs defined above.",
            c_code(&inp.map.name),
            c_code(&register.layout.name)
        )?;
    }

    out.pop_section();
    Ok(())
}

/// Generate register section header comment
fn generate_register_header(out: &mut dyn Write, inp: &Input, register: &Register) -> Result<(), Error> {
    let name = &register.name;
    writeln!(out)?;
    c_generate_section_header_comment(out, &format!("{name} Register"))?;
    if !register.docs.is_empty() {
        write!(out, "{}", register.docs.as_multiline("// "))?;
    }
    if inp.opts.is_enabled(Element::Structs) {
        writeln!(out, "// Fields:")?;
        writeln!(out, "{}", c_layout_overview_comment(&register.layout))?;
    }
    Ok(())
}

fn generate_register_properties(out: &mut dyn Write, inp: &Input, register: &Register) -> Result<(), Error> {
    if !inp.opts.is_enabled(Element::RegisterProperties) {
        return Ok(());
    }

    let mut defines: Vec<Vec<String>> = vec![];

    let macro_prefix = c_macro(&inp.map.name);
    let reg_macro_prefix = format!("{macro_prefix}_{}", c_macro(&register.name));

    // Address:
    defines.push(vec![
        format!("#define {}_ADDRESS", reg_macro_prefix),
        format!("(0x{:X}U)", register.adr),
        format!("//!< {} register address", register.name),
    ]);

    for endian in &inp.opts.endian {
        // Reset value:
        if let Some(reset_val) = &register.reset_val {
            defines.push(vec![
                format!("#define {}_RESET_{}", reg_macro_prefix, &endian.short().to_uppercase()),
                to_array_init(*reset_val, register.layout.width_bytes(), *endian),
                format!("//!< {} register reset value", register.name),
            ]);
        }
    }

    if !defines.is_empty() {
        writeln!(out)?;
        write!(out, "{}", str_table(&defines, "", " "))?;
    }

    Ok(())
}

pub fn generate_register_block(out: &mut dyn Write, inp: &Input, block: &RegisterBlock) -> Result<(), Error> {
    let mut out = HeaderWriter::new(out);

    let mut header = String::new();
    generate_register_block_header(&mut header, block)?;
    out.push_section_with_header(&[&header]);

    generate_register_block_properties(&mut out, inp, block)?;

    for member in block.members.values() {
        let mut header = String::new();
        generate_register_block_member_header(&mut header, inp, member)?;
        out.push_section_with_header(&[&header]);

        generate_register_block_member_properties(&mut out, inp, member, block)?;
        if member.layout.is_local {
            layouts::generate_layout(&mut out, inp, &member.layout)?;
        } else if inp.opts.is_enabled(Element::Structs) {
            writeln!(&mut out)?;
            writeln!(
                out,
                "// Register uses the {}_{} struct and conversion funcs defined above.",
                c_code(&inp.map.name),
                c_code(&member.layout.name)
            )?;
        }

        out.pop_section();
    }

    out.pop_section();
    Ok(())
}

fn generate_register_block_header(out: &mut dyn Write, block: &RegisterBlock) -> Result<(), Error> {
    let name = &block.name;
    writeln!(out)?;
    c_generate_section_header_comment(out, &format!("{name} Register Block"))?;
    if !block.docs.is_empty() {
        write!(out, "{}", block.docs.as_multiline("// "))?;
    }

    if !block.members.is_empty() {
        writeln!(out, "//")?;
        writeln!(out, "// Contains registers:")?;
        for member in block.members.values() {
            if let Some(brief) = &member.docs.brief {
                writeln!(out, "// - [0x{:02}] {}: {}", member.offset, member.name, brief)?;
            } else {
                writeln!(out, "// - [0x{:02}] {}", member.offset, member.name)?;
            }
        }
    }

    if !block.instances.is_empty() {
        writeln!(out, "//")?;
        writeln!(out, "// Instances:")?;
        for instance in block.instances.values() {
            if let Some(brief) = &instance.docs.brief {
                writeln!(out, "// - [0x{:02}] {}: {}", instance.adr, instance.name, brief)?;
            } else {
                writeln!(out, "// - [0x{:02}] {}", instance.adr, instance.name)?;
            }
        }
    }

    Ok(())
}

fn generate_register_block_properties(out: &mut dyn Write, inp: &Input, block: &RegisterBlock) -> Result<(), Error> {
    if !inp.opts.is_enabled(Element::RegisterProperties) {
        return Ok(());
    }

    let macro_prefix = c_macro(&inp.map.name);
    let macro_block_name = c_macro(&block.name);

    let mut defines = vec![];
    for member in block.members.values() {
        let macro_member_name = c_macro(&member.name);
        defines.push(vec![
            format!("#define {}_{}_OFFSET", macro_prefix, macro_member_name),
            format!("(0x{:X}U)", member.offset),
            format!("//!< Offset of {} register from {} block start", member.name, block.name),
        ]);
    }

    if !defines.is_empty() {
        writeln!(out)?;
        writeln!(out, "// Contained registers:")?;
        write!(out, "{}", str_table(&defines, "", " "))?;
    }

    let mut defines = vec![];
    for instance in block.instances.values() {
        let macro_instance_name = c_macro(&instance.name);
        defines.push(vec![
            format!("#define {}_{}_INSTANCE_{}", macro_prefix, macro_block_name, macro_instance_name),
            format!("(0x{:X}U)", instance.adr),
            format!("//!< Start of {} instance {}", block.name, instance.name),
        ]);
    }

    if !defines.is_empty() {
        writeln!(out)?;
        writeln!(out, "// Instances:")?;
        write!(out, "{}", str_table(&defines, "", " "))?;
    }

    Ok(())
}

fn generate_register_block_member_header(
    out: &mut dyn Write,
    inp: &Input,
    member: &RegisterBlockMember,
) -> Result<(), Error> {
    let name = &member.name;
    writeln!(out)?;
    c_generate_header_comment(out, &format!("{name} Register Block Member "))?;

    if !member.docs.is_empty() {
        write!(out, "{}", member.docs.as_multiline("// "))?;
    }

    if inp.opts.is_enabled(Element::Structs) {
        writeln!(out, "// Fields:")?;
        writeln!(out, "{}", c_layout_overview_comment(&member.layout))?;
    }
    Ok(())
}

fn generate_register_block_member_properties(
    out: &mut dyn Write,
    inp: &Input,
    member: &RegisterBlockMember,
    block: &RegisterBlock,
) -> Result<(), Error> {
    if !inp.opts.is_enabled(Element::RegisterProperties) {
        return Ok(());
    }

    let macro_prefix = c_macro(&inp.map.name);

    let mut defines = vec![];
    for block_instance in block.instances.values() {
        let member_instance = &block_instance.registers[&member.name];
        let reg_macro_prefix = format!("{macro_prefix}_{}", c_macro(&member_instance.name));

        // Address:
        defines.push(vec![
            format!("#define {}_ADDRESS", reg_macro_prefix),
            format!("(0x{:X}U)", member_instance.adr),
            format!("//!< {} register address", member_instance.name),
        ]);

        for endian in &inp.opts.endian {
            // Reset value:
            if let Some(reset_val) = &member_instance.reset_val {
                defines.push(vec![
                    format!("#define {}_RESET_{}", reg_macro_prefix, endian.short().to_uppercase()),
                    to_array_init(*reset_val, member_instance.layout.width_bytes(), *endian),
                    format!("//!< {} register reset value", member_instance.name),
                ]);
            }
        }
    }

    if !defines.is_empty() {
        writeln!(out)?;
        write!(out, "{}", str_table(&defines, "", " "))?;
    }

    Ok(())
}
