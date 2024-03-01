use std::fmt::Write;

use crate::{
    error::GeneratorError,
    regmap::{
        access_string,
        bits::{bit_mask_range, lsb_pos, mask_to_bit_ranges, msb_pos},
        FieldEnum, PhysicalRegister, RegisterMap, RegisterOrigin,
    },
};

use super::md_table;

pub fn generate(out: &mut dyn Write, map: &RegisterMap) -> Result<(), GeneratorError> {
    writeln!(out, "# {}", map.map_name)?;
    writeln!(out, "")?;
    writeln!(out, "## Register Map")?;
    writeln!(out, "")?;
    generate_overview(out, map)?;

    writeln!(out, "")?;
    writeln!(out, "## Register Details")?;
    let registers = map.physical_registers();
    for register in registers {
        generate_register_infos(out, map, &register)?;
    }

    Ok(())
}

fn generate_overview(out: &mut dyn Write, map: &RegisterMap) -> Result<(), GeneratorError> {
    let mut rows = vec![];
    rows.push(vec!["*Address*".to_string(), "*Register*".to_string(), "*Brief*".to_string()]);
    let regs = map.physical_registers();
    for reg in &regs {
        let adr = if let Some(adr) = &reg.absolute_adr {
            format!("0x{:X}", adr)
        } else {
            "-".to_string()
        };
        let name = reg.name.clone();
        let brief = reg.template.docs.brief.clone().unwrap_or("-".to_string());
        rows.push(vec![adr, name, brief]);
    }
    md_table(out, &rows)?;
    Ok(())
}

fn generate_register_infos(
    out: &mut dyn Write,
    map: &RegisterMap,
    register: &PhysicalRegister,
) -> Result<(), GeneratorError> {
    // Header:
    writeln!(out, "")?;
    writeln!(out, "### {}", register.name)?;

    // Overview table:
    let ranges = register.template.split_to_bitranges();

    let mut row_bits: Vec<String> = vec!["*Bits:*".to_string()];
    let mut row_field: Vec<String> = vec!["*Field:*".to_string()];
    let mut row_access: Vec<String> = vec!["*Access:*".to_string()];

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
                row_field.push(format!("Write '0b{:b}'", val));
                row_access.push(format!("/"));
            }
        }
    }
    writeln!(out, "")?;
    writeln!(out, "#### Register:")?;
    writeln!(out, "")?;
    md_table(out, &vec![row_bits, row_field, row_access])?;

    writeln!(out, "")?;
    writeln!(out, "#### Info:")?;
    writeln!(out, "")?;
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
            let val = always_write.value & bit_mask_range(&range) >> range.end();
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
            writeln!(out, "    - Offset from block start: 0x{:X}", offset)?;
        }
        if let Some(instance_start) = instance.adr {
            writeln!(out, "    - Instance {} start address: 0x{:X}", instance.name, instance_start)?;
        }
    }

    // Field Info:
    writeln!(out, "")?;
    writeln!(out, "#### Fields:")?;
    writeln!(out, "")?;

    for field in register.template.fields.values() {
        let access = if let Some(access) = &field.access {
            format!(" [{}]", access_string(access))
        } else {
            "".to_string()
        };
        writeln!(out, "  - {}{}:", field.name, access)?;
        write!(out, "{}", field.docs.as_twoline("    - "))?;

        if let Some(field_enum) = &field.field_enum {
            writeln!(out, "    - Accepts:")?;
            let enum_entries = match field_enum {
                FieldEnum::Local(field_enum) => field_enum.entries.values(),
                FieldEnum::Shared(shared_enun) => shared_enun.entries.values(),
            };
            for entry in enum_entries {
                writeln!(out, "      - 0x{:X}: {}", entry.value, entry.name)?;
                write!(out, "{}", entry.docs.as_twoline("        - "))?;
            }
        }
    }

    Ok(())
}
