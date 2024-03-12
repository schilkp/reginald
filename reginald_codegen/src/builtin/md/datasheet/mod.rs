pub mod regdump;

use std::fmt::Write;

use crate::{
    bits::{bitmask_from_range, lsb_pos, mask_to_bit_ranges, msb_pos},
    error::Error,
    regmap::{access_string, FieldType, PhysicalRegister, RegisterBitrange, RegisterMap, RegisterOrigin, TypeValue},
    utils::filename,
};

use super::md_table;

pub fn generate(out: &mut dyn Write, map: &RegisterMap) -> Result<(), Error> {
    writeln!(out, "# {}", map.map_name)?;
    writeln!(out)?;
    writeln!(out, "## Register Map")?;
    generate_overview(out, map)?;

    writeln!(out)?;
    writeln!(out, "## Register Details")?;
    let registers = map.physical_registers();
    for register in registers {
        generate_register_infos(out, &register, None)?;
    }

    Ok(())
}

fn generate_overview(out: &mut dyn Write, map: &RegisterMap) -> Result<(), Error> {
    if let Some(input_file) = &map.from_file {
        writeln!(out)?;
        writeln!(out, "Generated from listing file: {}.", filename(input_file)?)?;
    }
    if let Some(author) = &map.author {
        writeln!(out)?;
        writeln!(out, "Listing file author: {author}.")?;
    }
    if let Some(note) = &map.note {
        writeln!(out,)?;
        writeln!(out, "Listing file note:")?;
        writeln!(out, "```")?;
        for line in note.lines() {
            writeln!(out, "  {line}")?;
        }
        writeln!(out, "```")?;
    }

    let mut rows = vec![];
    rows.push(vec![
        "**Address**".to_string(),
        "**Register**".to_string(),
        "**Brief**".to_string(),
    ]);
    let regs = map.physical_registers();
    for reg in &regs {
        let adr = if let Some(adr) = &reg.absolute_adr {
            format!("0x{adr:X}")
        } else {
            "-".to_string()
        };
        let name = reg.name.clone();
        let brief = reg.template.docs.brief.clone().unwrap_or("-".to_string());
        rows.push(vec![adr, name, brief]);
    }
    writeln!(out)?;
    md_table(out, &rows)?;

    Ok(())
}

fn generate_register_infos(
    out: &mut dyn Write,
    register: &PhysicalRegister,
    value: Option<TypeValue>,
) -> Result<(), Error> {
    // Header:
    writeln!(out)?;
    writeln!(out, "### {}", register.name)?;

    // Overview table:
    let ranges = register.template.split_to_bitranges();

    let mut row_bits: Vec<String> = vec!["**Bits:**".to_string()];
    let mut row_field: Vec<String> = vec!["**Field:**".to_string()];
    let mut row_access: Vec<String> = vec!["**Access:**".to_string()];
    let mut row_state: Vec<String> = vec!["**State:**".to_string()];
    let mut row_decode: Vec<String> = vec!["**Decode:**".to_string()];

    for range in ranges.iter().rev() {
        if range.bits.start() == range.bits.end() {
            row_bits.push(format!("{}", range.bits.end()));
        } else {
            row_bits.push(format!("{}:{}", range.bits.end(), range.bits.start()));
        }

        match range.content {
            crate::regmap::RegisterBitrangeContent::Empty => {
                row_field.push("/".into());
                row_access.push("/".into());
            }
            crate::regmap::RegisterBitrangeContent::Field {
                field,
                is_split,
                subfield_mask,
            } => {
                if let Some(access) = &field.access {
                    row_access.push(access_string(access));
                } else {
                    row_access.push("?".into());
                }

                if is_split {
                    let lsb = lsb_pos(subfield_mask);
                    let msb = msb_pos(subfield_mask);
                    if lsb == msb {
                        row_field.push(format!("{}[{}]", field.name, msb));
                    } else {
                        row_field.push(format!("{}[{}:{}]", field.name, msb, lsb));
                    }
                } else {
                    row_field.push(field.name.clone());
                }
            }
            crate::regmap::RegisterBitrangeContent::AlwaysWrite { val } => {
                row_field.push(format!("Write '0b{val:b}'"));
                row_access.push("/".to_string());
            }
        }

        if let Some(value) = value {
            let value_range = (value & bitmask_from_range(&range.bits)) >> range.bits.start();
            row_state.push(format!("**0b{value_range:b}**"));
            row_decode.push(decode_bit_range(&value, range));
        }
    }

    writeln!(out)?;
    writeln!(out, "#### Register:")?;
    writeln!(out)?;
    if value.is_some() {
        md_table(out, &vec![row_bits, row_field, row_access, row_state, row_decode])?;
    } else {
        md_table(out, &vec![row_bits, row_field, row_access])?;
    }

    writeln!(out)?;
    writeln!(out, "#### Info:")?;
    writeln!(out)?;
    if let Some(value) = value {
        writeln!(out, "  - **Current Value: 0x{value:X}**")?;
    }
    if let Some(adr) = register.absolute_adr {
        writeln!(out, "  - Address: 0x{adr:X}")?;
    }
    if let Some(reset_val) = register.template.reset_val {
        writeln!(out, "  - Reset: 0x{reset_val:X}")?;
    }
    if let Some(always_write) = &register.template.always_write {
        let ranges = mask_to_bit_ranges(always_write.mask);
        writeln!(out, "  - Always write:")?;
        for range in ranges {
            let val = (always_write.value & bitmask_from_range(&range)) >> range.end();
            let bits = if range.end() == range.start() {
                format!("bit {}", range.end())
            } else {
                format!("bits [{}:{}]", range.end(), range.start())
            };
            writeln!(out, "    - 0b{val:b} to {bits}")?;
        }
    }

    write!(out, "{}", register.template.docs.as_twoline("  - "))?;
    if let RegisterOrigin::RegisterBlockInstance {
        block,
        instance,
        offset_from_block_start,
    } = &register.origin
    {
        writeln!(out, "  - In '{}' instance of '{}' block", instance.name, block.name)?;
        if let Some(offset) = offset_from_block_start {
            writeln!(out, "    - Offset from block start: 0x{offset:X}")?;
        }
        if let Some(instance_start) = instance.adr {
            writeln!(out, "    - Instance {} start address: 0x{:X}", instance.name, instance_start)?;
        }
    }

    // Field Info:
    writeln!(out)?;
    writeln!(out, "#### Fields:")?;
    writeln!(out)?;

    for field in register.template.fields.values() {
        let value_field = value.map(|x| (x & field.mask) >> lsb_pos(field.mask));

        let access = if let Some(access) = &field.access {
            format!(" [{}]", access_string(access))
        } else {
            String::new()
        };

        let value_string = value_field.map(|x| format!("0x{x:X}")).unwrap_or_default();

        writeln!(out, "  - {}{}: {}", field.name, access, value_string)?;
        write!(out, "{}", field.docs.as_twoline("    - "))?;

        if let Some(enum_entries) = field.enum_entries() {
            writeln!(out, "    - Accepts:")?;
            for entry in enum_entries {
                match value_field {
                    Some(val_field) if val_field == entry.value => {
                        writeln!(out, "      - **0x{:X}: {} (SELECTED)**", entry.value, entry.name)?;
                    }
                    _ => {
                        writeln!(out, "      - 0x{:X}: {}", entry.value, entry.name)?;
                    }
                }
                write!(out, "{}", entry.docs.as_twoline("        - "))?;
            }
        }
    }

    Ok(())
}

fn decode_bit_range(value: &TypeValue, range: &RegisterBitrange) -> String {
    let value_range = (value & bitmask_from_range(&range.bits)) >> range.bits.end();

    match range.content {
        crate::regmap::RegisterBitrangeContent::Field { field, .. } => {
            let field_value = (value & field.mask) >> lsb_pos(field.mask);
            if let Some(mut enum_entries) = field.enum_entries() {
                if let Some(entry) = enum_entries.find(|x| x.value == field_value) {
                    return format!("**{}**", entry.name);
                } else {
                    return "**UNKNOWN**".to_string();
                }
            } else if matches!(field.accepts, FieldType::Bool) {
                if field_value == 0 {
                    return "**false**".to_string();
                } else {
                    return "**true**".to_string();
                }
            }
        }
        crate::regmap::RegisterBitrangeContent::AlwaysWrite { val } => {
            if value_range == val {
                return "**OK**".to_string();
            } else {
                return "**ERROR**".to_string();
            }
        }
        _ => (),
    }

    String::new()
}
