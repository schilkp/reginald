use crate::{error::ListingError, regmap::validate::validate_docs, regmap::AccessMode};
use lazy_static::lazy_static;
use regex::Regex;
use std::{collections::BTreeMap, path::PathBuf, rc::Rc};

use super::{
    listing::{self},
    validate::{validate_bitpos, validate_register_template},
    Access, AlwaysWrite, Docs, Enum, EnumEntry, Field, FieldEnum, Instance, Register, RegisterBlock, RegisterMap,
    TypeAdr, TypeBitwidth, TypeValue,
};

pub fn convert_map(m: &listing::RegisterMap, input_file: &Option<PathBuf>) -> Result<RegisterMap, ListingError> {
    let bt = &m.map_name;

    let map_name = m.map_name.clone();
    let docs = convert_docs(&m.brief, &m.doc, bt)?;
    let shared_enums = convert_shared_enums(m, bt)?;
    let default_bitwidth = m.default_register_bitwidth;
    let register_blocks = convert_registers(m, default_bitwidth, &shared_enums, bt)?;

    Ok(RegisterMap {
        from_file: input_file.clone(),
        map_name,
        docs,
        register_blocks,
        shared_enums,
    })
}

fn convert_always_write(always_write: &Option<listing::AlwaysWrite>, _bt: &str) -> Option<AlwaysWrite> {
    always_write.as_ref().map(|always_write| AlwaysWrite {
        mask: always_write.mask,
        value: always_write.val,
    })
}

fn convert_bits(bits: &listing::Bits, bt: &str) -> Result<TypeValue, ListingError> {
    let bt = bt.to_owned() + ".bits";

    let mut result: TypeValue = 0;

    for piece in bits {
        let mask = match piece {
            listing::BitRange::Bit(bitpos) => convert_bitpos(*bitpos, &bt)?,
            listing::BitRange::Range(range) => convert_bitrange(range, &bt)?,
        };

        if mask & result != 0 {
            return Err(ListingError::ConversionError {
                bt,
                msg: format!("Bitranges in list overlap (mask: 0x{:x})", mask & result,),
            });
        };

        result |= mask;
    }
    if result == 0 {
        return Err(ListingError::ConversionError {
            bt,
            msg: "Bits cannot be zero.".into(),
        });
    };

    Ok(result)
}

fn convert_bitpos(bitpos: TypeBitwidth, bt: &str) -> Result<TypeValue, ListingError> {
    validate_bitpos(bitpos, bt)?;
    Ok(1 << bitpos)
}

lazy_static! {
    static ref BITRANGE_RE: Regex = Regex::new(r"[^_a-zA-Z0-9]").unwrap();
}

fn convert_bitrange(bitrange: &str, bt: &str) -> Result<TypeValue, ListingError> {
    if !BITRANGE_RE.is_match(bitrange) {
        return Err(ListingError::ConversionError {
            bt: bt.to_string(),
            msg: format!("Malformed bit range '{}'", bitrange),
        });
    };

    let limit_strs: Vec<&str> = bitrange.splitn(2, '-').collect();
    let mut limits = vec![];
    for limit in limit_strs {
        let limit: TypeBitwidth = limit.parse().map_err(|_| ListingError::ConversionError {
            bt: bt.to_owned(),
            msg: format!("Malformed bit range '{}'", bitrange),
        })?;

        validate_bitpos(limit, bt)?;

        limits.push(limit);
    }

    let mut mask = 0;
    let min = *limits.iter().min().unwrap();
    let max = *limits.iter().max().unwrap();
    for pos in min..=max {
        mask |= 1 << pos;
    }

    Ok(mask)
}

fn convert_access(access: &listing::Access, _bt: &str) -> Option<Access> {
    let result: Vec<AccessMode> = access
        .iter()
        .map(|x| match x {
            listing::AccessMode::R => AccessMode::R,
            listing::AccessMode::W => AccessMode::W,
        })
        .collect();

    if result.is_empty() {
        None
    } else {
        Some(result)
    }
}

fn convert_docs(brief: &Option<String>, doc: &Option<String>, bt: &str) -> Result<Docs, ListingError> {
    let brief = brief.clone().map(|x| x.trim_end().to_owned());
    let doc = doc.clone().map(|x| x.trim_end().to_owned());
    let docs = Docs { brief, doc };
    validate_docs(docs, bt)
}

fn convert_shared_enums(m: &listing::RegisterMap, bt: &str) -> Result<BTreeMap<String, Rc<Enum>>, ListingError> {
    let mut result = BTreeMap::new();

    let bt = bt.to_owned() + ".enums";

    for (shared_enum_name, shared_enum) in &m.enums {
        let bt = bt.clone() + shared_enum_name;
        result.insert(
            shared_enum_name.to_owned(),
            Rc::new(Enum {
                name: shared_enum_name.to_owned(),
                is_shared: true,
                docs: convert_docs(&shared_enum.brief, &shared_enum.doc, &bt)?,
                entries: convert_enum_entries(&shared_enum.entries, &bt)?,
            }),
        );
    }

    Ok(result)
}

fn convert_enum_entries(entries: &listing::EnumEntries, bt: &str) -> Result<BTreeMap<String, EnumEntry>, ListingError> {
    let mut result: BTreeMap<String, EnumEntry> = BTreeMap::new();

    for (entry_name, entry) in entries {
        let bt = bt.to_owned() + "." + entry_name;
        result.insert(
            entry_name.clone(),
            EnumEntry {
                name: entry_name.to_owned(),
                value: entry.val,
                docs: convert_docs(&entry.brief, &entry.doc, &bt)?,
            },
        );
    }

    Ok(result)
}

fn convert_local_field_enum(
    field: &listing::Field,
    field_name: &str,
    local_enum: &listing::EnumEntries,
    bt: &str,
) -> Result<Option<FieldEnum>, ListingError> {
    Ok(Some(FieldEnum::Local(Enum {
        name: field_name.to_owned(),
        is_shared: false,
        docs: convert_docs(&field.brief, &field.doc, bt)?,
        entries: convert_enum_entries(local_enum, bt)?,
    })))
}

fn convert_shared_field_enum(
    name: &str,
    shared_enums: &BTreeMap<String, Rc<Enum>>,
    bt: &str,
) -> Result<Option<FieldEnum>, ListingError> {
    let shared_enum = shared_enums.get(name).ok_or(ListingError::ConversionError {
        bt: bt.to_string(),
        msg: format!("Shared enum '{}' not found.", name),
    })?;
    Ok(Some(FieldEnum::Shared(shared_enum.clone())))
}

fn convert_field_enum(
    field: &listing::Field,
    field_name: &str,
    shared_enums: &BTreeMap<String, Rc<Enum>>,
    bt: &str,
) -> Result<Option<FieldEnum>, ListingError> {
    let bt = bt.to_owned() + ".enum";

    match &field.field_enum {
        Some(listing::FieldEnum::Enum(e)) => convert_local_field_enum(field, field_name, e, &bt),
        Some(listing::FieldEnum::SharedEnum(name)) => convert_shared_field_enum(name, shared_enums, &bt),
        None => Ok(None),
    }
}

fn convert_fields(
    reg: &listing::Register,
    shared_enums: &BTreeMap<String, Rc<Enum>>,
    bt: &str,
) -> Result<BTreeMap<String, Field>, ListingError> {
    let mut result = BTreeMap::new();
    let bt = bt.to_owned() + ".fields";

    for (field_name, field) in &reg.fields {
        result.insert(field_name.clone(), convert_field(field_name, field, shared_enums, &bt)?);
    }

    Ok(result)
}

fn convert_field(
    field_name: &str,
    field: &listing::Field,
    shared_enums: &BTreeMap<String, Rc<Enum>>,
    bt: &str,
) -> Result<Field, ListingError> {
    let bt = bt.to_owned() + "." + field_name;

    Ok(Field {
        name: field_name.to_owned(),
        mask: convert_bits(&field.bits, &bt)?,
        access: convert_access(&field.access, &bt),
        docs: convert_docs(&field.brief, &field.doc, &bt)?,
        field_enum: convert_field_enum(field, field_name, shared_enums, &bt)?,
    })
}

fn convert_registers(
    map: &listing::RegisterMap,
    default_bitwidth: TypeBitwidth,
    shared_enums: &BTreeMap<String, Rc<Enum>>,
    bt: &str,
) -> Result<BTreeMap<String, RegisterBlock>, ListingError> {
    let bt = bt.to_owned() + ".registers";
    let mut result: BTreeMap<String, RegisterBlock> = BTreeMap::new();

    for (physreg_name, physreg) in &map.registers {
        let block = match physreg {
            listing::PhysicalRegister::Register(reg) => {
                convert_register(physreg_name, reg, default_bitwidth, shared_enums, &bt)?
            }
            listing::PhysicalRegister::RegisterBlock(regblock) => {
                convert_register_block(physreg_name, regblock, default_bitwidth, shared_enums, &bt)?
            }
        };
        result.insert(physreg_name.clone(), block);
    }

    Ok(result)
}

fn convert_register(
    reg_name: &str,
    reg: &listing::Register,
    default_bitwidth: TypeBitwidth,
    shared_enums: &BTreeMap<String, Rc<Enum>>,
    bt: &str,
) -> Result<RegisterBlock, ListingError> {
    let bt = bt.to_owned() + reg_name;

    let docs = convert_docs(&reg.brief, &reg.doc, &bt)?;

    // Register template:
    // Single register gets converted into a block at the register
    // with a single unnamed register template at offset 0.
    let template = Register {
        name: String::new(), // Template unnamed.
        bitwidth: reg.bitwidth.unwrap_or(default_bitwidth),
        is_block_template: false,
        adr: Some(0), // Offset
        reset_val: reg.reset_val,
        always_write: convert_always_write(&reg.always_write, &bt),
        fields: convert_fields(reg, shared_enums, &bt)?,
        docs: docs.clone(),
    };

    let template = validate_register_template(template, &bt)?;

    let block = RegisterBlock {
        name: reg_name.to_owned(),
        instances: BTreeMap::from([(
            reg_name.to_owned(),
            Instance {
                name: reg_name.to_owned(),
                adr: reg.adr,
            },
        )]),
        docs,
        register_templates: BTreeMap::from([(String::new(), template)]),
    };

    Ok(block)
}

fn convert_register_block_templates(
    block: &listing::RegisterBlock,
    default_bitwidth: TypeBitwidth,
    shared_enums: &BTreeMap<String, Rc<Enum>>,
    bt: &str,
) -> Result<BTreeMap<String, Register>, ListingError> {
    let mut result = BTreeMap::new();

    for (template_name, template) in &block.registers {
        let bt = bt.to_owned() + "." + template_name;
        let template = Register {
            name: template_name.to_string(),
            bitwidth: template.bitwidth.unwrap_or(default_bitwidth),
            is_block_template: true,
            adr: template.adr,
            reset_val: template.reset_val,
            always_write: convert_always_write(&template.always_write, &bt),
            fields: convert_fields(template, shared_enums, &bt)?,
            docs: convert_docs(&template.brief, &template.doc, &bt)?,
        };
        result.insert(template_name.clone(), validate_register_template(template, &bt)?);
    }

    Ok(result)
}

fn convert_instances(instances: &BTreeMap<String, Option<TypeAdr>>) -> BTreeMap<String, Instance> {
    instances
        .iter()
        .map(|(name, adr)| {
            (
                name.to_owned(),
                Instance {
                    name: name.clone(),
                    adr: *adr,
                },
            )
        })
        .collect()
}

fn convert_register_block(
    block_name: &str,
    block: &listing::RegisterBlock,
    default_bitwidth: TypeBitwidth,
    shared_enums: &BTreeMap<String, Rc<Enum>>,
    bt: &str,
) -> Result<RegisterBlock, ListingError> {
    let bt = bt.to_owned() + block_name;
    Ok(RegisterBlock {
        name: block_name.to_string(),
        instances: convert_instances(&block.instances),
        docs: convert_docs(&block.brief, &block.doc, &bt)?,
        register_templates: convert_register_block_templates(block, default_bitwidth, shared_enums, &bt)?,
    })
}

#[cfg(test)]
mod tests {
    use crate::regmap::assert_regmap_eq;

    use super::*;
    use pretty_assertions::assert_eq;
    use std::path::PathBuf;

    fn convert_yaml_example(file: &str) -> RegisterMap {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("../examples/maps/");
        path.push(file);
        let reader = std::fs::File::open(path).unwrap();
        RegisterMap::from_yaml(reader).unwrap()
    }

    fn convert_hjson_example(file: &str) -> RegisterMap {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("../examples/maps/");
        path.push(file);
        let reader = std::fs::File::open(path).unwrap();
        RegisterMap::from_hjson(reader).unwrap()
    }

    #[test]
    fn convert_examples_dummy() {
        let map_yaml = convert_yaml_example("dummy.yaml");
        let map_hjson = convert_hjson_example("dummy.hjson");
        assert_regmap_eq(map_yaml, map_hjson);
    }

    #[test]
    fn convert_examples_lsm6dsv16bx() {
        let map_yaml = convert_yaml_example("lsm6dsv16bx.yaml");
        let map_hjson = convert_hjson_example("lsm6dsv16bx.hjson");
        assert_regmap_eq(map_yaml, map_hjson);
    }

    #[test]
    fn convert_examples_max77654() {
        let map_yaml = convert_yaml_example("max77654.yaml");
        let map_hjson = convert_hjson_example("max77654.hjson");
        assert_regmap_eq(map_yaml, map_hjson);
    }

    #[test]
    fn test_convert_bits() {
        assert_eq!(convert_bits(&vec![listing::BitRange::Bit(0)], "").unwrap(), 0b1 << 0,);

        assert_eq!(convert_bits(&vec![listing::BitRange::Bit(8)], "").unwrap(), 0b1 << 8,);

        assert_eq!(convert_bits(&vec![listing::BitRange::Bit(0), listing::BitRange::Bit(1)], "").unwrap(), 0b11,);

        assert_eq!(
            convert_bits(&vec![listing::BitRange::Range("3-4".into()), listing::BitRange::Bit(0)], "").unwrap(),
            0b11001,
        );
    }

    #[test]
    fn test_catch_empty_bits() {
        convert_bits(&vec![], "").unwrap_err();
    }

    #[test]
    fn test_catch_overlapping_bits() {
        convert_bits(&vec![listing::BitRange::Range("3-4".into()), listing::BitRange::Bit(3)], "").unwrap_err();

        convert_bits(&vec![listing::BitRange::Range("3".into()), listing::BitRange::Bit(3)], "").unwrap_err();
    }

    #[test]
    fn test_catch_malformed_range() {
        convert_bits(&vec![listing::BitRange::Range("3- 4".into())], "").unwrap_err();
        convert_bits(&vec![listing::BitRange::Range("4".into())], "").unwrap_err();
        convert_bits(&vec![listing::BitRange::Range("a-b".into())], "").unwrap_err();
        convert_bits(
            &vec![listing::BitRange::Range(
                "0-999999999999999999999999999999999999999999999999999999".into(),
            )],
            "",
        )
        .unwrap_err();
    }
}