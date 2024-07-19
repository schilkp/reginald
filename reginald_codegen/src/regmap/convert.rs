use crate::{
    bits::mask_width,
    error::Error,
    regmap::{validate::validate_docs, Layout},
};

use reginald_utils::join_with_underscore;

use lazy_static::lazy_static;
use regex::Regex;
use std::{collections::BTreeMap, path::PathBuf, rc::Rc};

use super::{
    listing::{self},
    validate::{
        validate_bitpos, validate_bitwidth, validate_enum, validate_layout, validate_map_author, validate_name,
        validate_name_unique, validate_register, validate_register_properties, Namespace,
    },
    Access, AccessMode, Defaults, Docs, Enum, EnumEntry, FieldType, LayoutField, Register, RegisterBlock,
    RegisterBlockInstance, RegisterBlockMember, RegisterMap, RegisterOrigin, TypeBitwidth, TypeValue,
};

// ==== Main Conversion Routine ====================================================================

pub fn convert_map(m: &listing::RegisterMap, input_file: &Option<PathBuf>) -> Result<RegisterMap, Error> {
    let bt = &m.name;

    // Convert basic properties:
    let from_file = input_file.clone();
    validate_name(&m.name, bt, ".name")?;
    let name = m.name.clone();
    let notice = m.notice.clone().map(|x| x.trim_end().to_string());
    let author = m.author.clone();
    validate_map_author(&author, bt)?;
    let docs = convert_docs(&m.doc, bt)?;
    let defaults = convert_defaults(&m.defaults, bt)?;

    // Construct empty register map:
    let mut map = RegisterMap {
        from_file,
        name,
        docs,
        notice,
        author,
        defaults,
        enums: BTreeMap::new(),
        layouts: BTreeMap::new(),
        register_blocks: BTreeMap::new(),
        registers: BTreeMap::new(),
    };

    // Namespace map to detect naming collisions and provide nice error messages:
    let mut namespace: Namespace = BTreeMap::new();

    // Convert data.
    // Note: Order matters. Registers depend on layouts and enums. Layouts depend
    // on enums.
    convert_shared_enums(&mut map, &mut namespace, m, bt)?;
    convert_shared_layouts(&mut map, &mut namespace, m, bt)?;
    convert_registers(&mut map, &mut namespace, m, bt)?;

    Ok(map)
}

// ==== Properties/Types Conversions ===============================================================

fn convert_defaults(defaults: &listing::Defaults, bt: &str) -> Result<Defaults, Error> {
    let bt = bt.to_owned() + ".defaults";
    if let Some(bitwidth) = defaults.layout_bitwidth {
        validate_bitwidth(bitwidth, &bt)?;
    }

    Ok(Defaults {
        layout_bitwidth: defaults.layout_bitwidth,
        field_access_mode: defaults.field_access_mode.as_ref().map(convert_access_list),
    })
}

fn convert_bits(bits: &listing::Bits, bt: &str) -> Result<TypeValue, Error> {
    let bt = bt.to_owned() + ".bits";

    let mut result: TypeValue = 0;

    for piece in bits {
        let mask = match piece {
            listing::BitRange::Bit(bitpos) => convert_bitpos(*bitpos, &bt)?,
            listing::BitRange::Range(range) => convert_bitrange(range, &bt)?,
        };

        if mask & result != 0 {
            return Err(Error::ConversionError {
                bt,
                msg: format!("Bitranges in list overlap (mask: 0x{:x})", mask & result,),
            });
        };

        result |= mask;
    }
    if result == 0 {
        return Err(Error::ConversionError {
            bt,
            msg: "Bits cannot be zero.".into(),
        });
    };

    Ok(result)
}

fn convert_bitpos(bitpos: TypeBitwidth, bt: &str) -> Result<TypeValue, Error> {
    validate_bitpos(bitpos, bt)?;
    Ok(1 << bitpos)
}

lazy_static! {
    static ref BITRANGE_RE: Regex = Regex::new(r"[^_a-zA-Z0-9]").unwrap();
}

fn convert_bitrange(bitrange: &str, bt: &str) -> Result<TypeValue, Error> {
    if !BITRANGE_RE.is_match(bitrange) {
        return Err(Error::ConversionError {
            bt: bt.to_string(),
            msg: format!("Malformed bit range '{bitrange}'"),
        });
    };

    let limit_strs: Vec<&str> = bitrange.splitn(2, '-').collect();
    let mut limits = vec![];
    for limit in limit_strs {
        let limit: TypeBitwidth = limit.parse().map_err(|_| Error::ConversionError {
            bt: bt.to_owned(),
            msg: format!("Malformed bit range '{bitrange}'"),
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

fn convert_bitwidth(map: &RegisterMap, bitwidth: &Option<TypeBitwidth>, bt: &str) -> Result<TypeBitwidth, Error> {
    let width = match (bitwidth, map.defaults.layout_bitwidth) {
        (Some(width), _) => Ok(*width),
        (None, Some(width)) => Ok(width),
        (None, None) => Err(Error::ConversionError {
            bt: bt.to_string(),
            msg: "Unknown bitwidth: No specific bitwidth given and no default bitwidth is set.".to_string(),
        }),
    }?;

    validate_bitwidth(width, bt)?;
    Ok(width)
}

fn convert_access_list(access: &listing::Access) -> Access {
    access
        .iter()
        .map(|x| match x {
            listing::AccessMode::R => AccessMode::R,
            listing::AccessMode::W => AccessMode::W,
        })
        .collect()
}

fn convert_access(map: &RegisterMap, access: &Option<listing::Access>) -> Option<Access> {
    if let Some(access) = access {
        Some(convert_access_list(access))
    } else {
        map.defaults.field_access_mode.clone()
    }
}

fn convert_docs(doc: &Option<String>, bt: &str) -> Result<Docs, Error> {
    if let Some(doc) = doc {
        let mut lines: Vec<&str> = doc.trim().lines().collect();

        // If there is only one line, treat it as only a brief:
        if lines.len() == 1 {
            let doc = Docs {
                brief: Some(lines[0].to_string()),
                doc: None,
            };
            return validate_docs(doc, bt);
        }

        // Otherwise, find the first blank line (if any), and split the doc into two parts:
        let (brief_lines, mut doc_lines): (Vec<&str>, Vec<&str>) =
            if let Some(blank_line) = lines.clone().iter().position(|x| x.trim().is_empty()) {
                (lines[0..blank_line].to_vec(), lines[blank_line..].to_vec())
            } else {
                (vec![], lines.clone())
            };

        if brief_lines.len() == 1 {
            // Trim leading and trailing blank lines:
            while let Some(true) = doc_lines.first().map(|x| x.trim().is_empty()) {
                doc_lines.remove(0);
            }
            while let Some(true) = doc_lines.last().map(|x| x.trim().is_empty()) {
                doc_lines.pop();
            }

            let doc = doc_lines.join("\n");
            let doc = if doc.is_empty() { None } else { Some(doc) };

            let doc = Docs {
                brief: Some(brief_lines[0].to_string()),
                doc,
            };
            validate_docs(doc, bt)
        } else {
            // Trim leading and trailing blank lines:
            while let Some(true) = lines.first().map(|x| x.trim().is_empty()) {
                lines.remove(0);
            }
            while let Some(true) = lines.last().map(|x| x.trim().is_empty()) {
                lines.pop();
            }

            let doc = lines.join("\n");
            let doc = if doc.is_empty() { None } else { Some(doc) };

            let doc = Docs { brief: None, doc };
            validate_docs(doc, bt)
        }
    } else {
        Ok(Docs::default())
    }
}

// ==== Enum Conversion ============================================================================

fn convert_shared_enums(
    map: &mut RegisterMap,
    namespace: &mut Namespace,
    inp: &listing::RegisterMap,
    bt: &str,
) -> Result<(), Error> {
    let bt = bt.to_owned() + ".enums";

    for (shared_enum_name, shared_enum) in &inp.enums {
        let bt = bt.clone() + "." + shared_enum_name.as_str();

        validate_name(shared_enum_name, &bt, "")?;
        validate_name_unique(shared_enum_name, namespace, &bt)?;

        let e = Rc::new(Enum {
            name: shared_enum_name.to_owned(),
            is_local: false,
            docs: convert_docs(&shared_enum.doc, &bt)?,
            entries: convert_enum_entries(&shared_enum.entries, &bt)?,
        });

        validate_enum(&e, &bt)?;

        map.enums.insert(shared_enum_name.to_owned(), e);
    }
    Ok(())
}

fn convert_enum_entries(entries: &listing::EnumEntries, bt: &str) -> Result<BTreeMap<String, EnumEntry>, Error> {
    let mut result: BTreeMap<String, EnumEntry> = BTreeMap::new();

    for (entry_name, entry) in entries {
        let bt = bt.to_owned() + "." + entry_name.as_str();

        validate_name(entry_name, &bt, "")?;

        result.insert(
            entry_name.clone(),
            EnumEntry {
                name: entry_name.to_owned(),
                value: entry.val,
                docs: convert_docs(&entry.doc, &bt)?,
            },
        );
    }

    Ok(result)
}

// ==== Layout Conversion ==========================================================================

fn convert_shared_layouts(
    map: &mut RegisterMap,
    namespace: &mut Namespace,
    inp: &listing::RegisterMap,
    bt: &str,
) -> Result<(), Error> {
    let bt = bt.to_owned() + ".layout";

    for (shared_layout_name, shared_layout) in &inp.layouts {
        let bt = bt.clone() + "." + shared_layout_name.as_str();

        validate_name(shared_layout_name, &bt, "")?;
        validate_name_unique(shared_layout_name, namespace, &bt)?;

        let docs = convert_docs(&shared_layout.doc, &bt)?;
        let fields = convert_layout_fields(map, namespace, &shared_layout.layout, &bt)?;

        let layout = Layout {
            name: shared_layout_name.clone(),
            bitwidth: convert_bitwidth(map, &shared_layout.bitwidth, &bt)?,
            is_local: false,
            docs,
            fields,
        };

        validate_layout(&layout, &bt)?;
        map.layouts.insert(shared_layout_name.clone(), layout.into());
    }
    Ok(())
}

fn convert_layout_fields(
    map: &mut RegisterMap,
    namespace: &mut Namespace,
    fields: &listing::LayoutFields,
    bt: &str,
) -> Result<BTreeMap<String, LayoutField>, Error> {
    let mut result: BTreeMap<String, LayoutField> = BTreeMap::new();

    let bt = bt.to_owned() + ".layout";

    for (field_name, field) in fields {
        let bt = bt.clone() + "." + field_name.as_str();

        result.insert(field_name.clone(), convert_field(map, namespace, field_name, field, &bt)?);
    }

    Ok(result)
}

fn convert_field(
    map: &mut RegisterMap,
    namespace: &mut Namespace,
    field_name: &str,
    field: &listing::LayoutField,
    bt: &str,
) -> Result<LayoutField, Error> {
    let bt = bt.to_owned() + "." + field_name;
    validate_name(field_name, &bt, "")?;

    let mask = convert_bits(&field.bits, &bt)?;

    let accepts = match &field.accepts {
        listing::FieldType::UInt => FieldType::UInt,
        listing::FieldType::Bool => FieldType::Bool,
        listing::FieldType::Fixed(fixed) => FieldType::Fixed(*fixed),
        listing::FieldType::SharedEnum(name) => {
            let e = match map.enums.get(name) {
                Some(e) if !e.is_local => Ok(e),
                _ => Err(Error::ConversionError {
                    bt: bt.to_string(),
                    msg: format!("Shared enum '{name}' not found."),
                }),
            }?;
            FieldType::Enum(e.clone())
        }
        listing::FieldType::SharedLayout(name) => {
            let layout = match map.layouts.get(name) {
                Some(e) if !e.is_local => Ok(e),
                _ => Err(Error::ConversionError {
                    bt: bt.to_string(),
                    msg: format!("Shared layout '{name}' not found."),
                }),
            }?;
            FieldType::Layout(layout.clone())
        }
        listing::FieldType::Enum(entries) => convert_local_enum(map, namespace, field_name, field, entries, &bt)?,
        listing::FieldType::Layout(entries) => {
            convert_field_local_layout(map, namespace, field_name, field, entries, mask_width(mask), &bt)?
        }
    };

    Ok(LayoutField {
        name: field_name.to_owned(),
        mask,
        docs: convert_docs(&field.doc, &bt)?,
        access: convert_access(map, &field.access),
        accepts,
    })
}

fn convert_local_enum(
    map: &mut RegisterMap,
    namespace: &mut Namespace,
    field_name: &str,
    field: &listing::LayoutField,
    entries: &listing::EnumEntries,
    bt: &str,
) -> Result<FieldType, Error> {
    validate_name(field_name, bt, "")?;
    validate_name_unique(field_name, namespace, bt)?;

    let e = Enum {
        name: field_name.to_owned(),
        is_local: true,
        docs: convert_docs(&field.doc, bt)?,
        entries: convert_enum_entries(entries, bt)?,
    };
    validate_enum(&e, bt)?;

    let e = Rc::from(e);
    map.enums.insert(field_name.to_string(), e.clone());

    Ok(FieldType::Enum(e))
}

fn convert_field_local_layout(
    map: &mut RegisterMap,
    namespace: &mut Namespace,
    field_name: &str,
    field: &listing::LayoutField,
    entries: &listing::LayoutFields,
    bitwidth: TypeBitwidth,
    bt: &str,
) -> Result<FieldType, Error> {
    validate_name(field_name, bt, "")?;
    validate_name_unique(field_name, namespace, bt)?;

    let layout = Layout {
        name: field_name.to_owned(),
        docs: convert_docs(&field.doc, bt)?,
        bitwidth,
        is_local: true,
        fields: convert_layout_fields(map, namespace, entries, bt)?,
    };

    validate_layout(&layout, bt)?;

    let layout = Rc::from(layout);
    map.layouts.insert(field_name.to_string(), layout.clone());

    Ok(FieldType::Layout(layout))
}

// ==== Register Conversion ========================================================================

fn convert_registers(
    map: &mut RegisterMap,
    namespace: &mut Namespace,
    inp: &listing::RegisterMap,
    bt: &str,
) -> Result<(), Error> {
    let bt = bt.to_owned() + ".registers";

    for (name, item) in &inp.registers {
        match item {
            listing::RegisterListing::Register(reg) => convert_register(map, namespace, name, reg, &bt)?,
            listing::RegisterListing::RegisterBlock(regblock) => {
                convert_register_block(map, namespace, name, regblock, &bt)?
            }
        }
    }

    Ok(())
}

fn convert_register_layout(
    map: &mut RegisterMap,
    namespace: &mut Namespace,
    layout: &listing::RegisterLayout,
    name: &str,
    docs: Docs,
    bitwidth: Option<TypeBitwidth>,
    bt: &str,
) -> Result<Rc<Layout>, Error> {
    match &layout {
        listing::RegisterLayout::Layout(fields) => {
            validate_name(name, bt, "")?;
            validate_name_unique(name, namespace, bt)?;

            let fields = convert_layout_fields(map, namespace, fields, bt)?;

            let layout = Layout {
                name: name.to_string(),
                is_local: true,
                bitwidth: convert_bitwidth(map, &bitwidth, bt)?,
                docs,
                fields,
            };

            validate_layout(&layout, bt)?;

            let layout = Rc::new(layout);
            map.layouts.insert(name.to_string(), layout.clone());
            Ok(layout)
        }
        listing::RegisterLayout::SharedLayout(shared_name) => {
            if bitwidth.is_some() {
                return Err(Error::ConversionError {
                    bt: bt.to_string(),
                    msg:
                        "Specified bitwidth for shared layout. Bitwidth can only be specified if specifying new layout."
                            .to_string(),
                });
            }

            let layout = match map.layouts.get(shared_name) {
                Some(e) if !e.is_local => Ok(e),
                _ => Err(Error::ConversionError {
                    bt: bt.to_string(),
                    msg: format!("Shared layout '{shared_name}' not found."),
                }),
            }?;
            Ok(layout.clone())
        }
    }
}

fn convert_register(
    map: &mut RegisterMap,
    namespace: &mut Namespace,
    reg_name: &str,
    reg: &listing::Register,
    bt: &str,
) -> Result<(), Error> {
    let bt = bt.to_owned() + "." + reg_name;

    validate_name(reg_name, &bt, "")?;

    let docs = convert_docs(&reg.doc, &bt)?;
    let layout = convert_register_layout(map, namespace, &reg.layout, reg_name, docs.clone(), reg.bitwidth, &bt)?;
    let reg = Register {
        name: reg_name.to_owned(),
        docs,
        adr: reg.adr,
        reset_val: reg.reset_val,
        layout,
        from_block: None,
    };

    validate_register(&reg, &bt)?;

    let reg = Rc::new(reg);
    map.registers.insert(reg_name.to_string(), reg);

    Ok(())
}

fn convert_register_block(
    map: &mut RegisterMap,
    namespace: &mut Namespace,
    block_name: &str,
    block: &listing::RegisterBlock,
    bt: &str,
) -> Result<(), Error> {
    let bt = bt.to_owned() + "." + block_name;

    validate_name(block_name, &bt, "")?;

    let mut members = BTreeMap::new();
    let mut fixed_reset_vals: BTreeMap<String, TypeValue> = BTreeMap::new();

    for (member_name_raw, member) in &block.registers {
        let bt = bt.to_owned() + ".registers." + member_name_raw.as_str();

        let member_name = join_with_underscore(block_name, member_name_raw);

        validate_name(&member_name, &bt, "")?;
        let docs = convert_docs(&member.doc, &bt)?;
        let offset = member.offset;
        let bitwidth = convert_bitwidth(map, &member.bitwidth, &bt)?;
        let layout =
            convert_register_layout(map, namespace, &member.layout, &member_name, docs.clone(), member.bitwidth, &bt)?;

        if let Some(reset_val) = member.reset_val {
            fixed_reset_vals.insert(member_name.to_string(), reset_val);
        }
        let reset_val = member.reset_val;

        validate_register_properties(&layout, bitwidth, reset_val, &bt)?;

        let member = RegisterBlockMember {
            name: member_name.to_string(),
            name_raw: member_name_raw.to_string(),
            docs,
            offset,
            layout,
        };

        members.insert(member_name.to_string(), Rc::new(member));
    }

    let mut instances = BTreeMap::new();

    for (block_instance_name, block_instance) in &block.instances {
        let bt = bt.to_owned() + ".instances." + block_instance_name.as_str();

        let mut register_instances = BTreeMap::new();

        // Provide error message if a reset value is specified for a member that does not exist:
        for reset_val_name in block_instance.reset_vals.keys() {
            if !block.registers.contains_key(reset_val_name) {
                return Err(Error::ConversionError {
                    bt: bt.to_string() + ".reset_vals",
                    msg: format!("Register block instance register specifies reset value for member '{reset_val_name}' which does not exist."),
                });
            }
        }

        for member in members.values() {
            let adr = member.offset + block_instance.adr;
            let member_name_generic = &member.name;
            let member_name_raw = &member.name_raw;
            let register_instance_name = join_with_underscore(block_instance_name, &member.name_raw);
            let reset_val = match (block_instance.reset_vals.get(member_name_generic), fixed_reset_vals.get(member_name_generic)) {
                (None, None) => None,
                (None, Some(val)) => Some(*val),
                (Some(val), None) => Some(*val),
                (Some(_), Some(_)) => return Err(Error::ConversionError {
                    bt: bt.to_string(),
                    msg: format!("Both register block member '{member_name_raw}' and instance '{block_instance_name}' have a reset value specified."),
                }),
            };

            let instance = Rc::new(Register {
                name: register_instance_name,
                docs: member.docs.clone(),
                adr,
                reset_val,
                layout: member.layout.clone(),
                from_block: Some(RegisterOrigin {
                    block: block_name.to_string(),
                    instance: block_instance_name.clone(),
                    block_member: member.name.clone(),
                }),
            });

            validate_register(&instance, &bt)?;

            map.registers.insert(instance.name.clone(), instance.clone());
            register_instances.insert(member_name_generic.clone(), instance);
        }

        let instance_docs = convert_docs(&block_instance.doc, &bt)?;

        let block_instance = RegisterBlockInstance {
            name: block_instance_name.clone(),
            adr: block_instance.adr,
            docs: instance_docs,
            registers: register_instances,
        };

        instances.insert(block_instance_name.to_string(), block_instance);
    }

    let docs = convert_docs(&block.doc, &bt)?;
    let block = RegisterBlock {
        name: block_name.to_owned(),
        docs,
        instances,
        members,
    };

    map.register_blocks.insert(block_name.to_string(), block);
    Ok(())
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
    fn convert_examples_max77654() {
        let map_yaml = convert_yaml_example("max77654.yaml");
        let map_hjson = convert_hjson_example("max77654.hjson");
        assert_regmap_eq(map_yaml, map_hjson);
    }

    #[test]
    fn test_bitwidth_defaults() {
        let yaml = "
        name: DummyChip
        registers:
            REG: !Register
                bitwidth: 8
                adr: 0x1000
                layout: !Layout
        ";
        let map = RegisterMap::from_yaml_str(yaml).unwrap();
        assert!(map.registers.get("REG").unwrap().layout.bitwidth == 8);

        let yaml = "
        name: DummyChip
        defaults:
            layout_bitwidth: 10
        registers:
            REG: !Register
                adr: 0x1000
                layout: !Layout
        ";
        let map = RegisterMap::from_yaml_str(yaml).unwrap();
        assert!(map.registers.get("REG").unwrap().layout.bitwidth == 10);

        let yaml = "
        name: DummyChip
        defaults:
            layout_bitwidth: 10
        registers:
            REG: !Register
                adr: 0x1000
                bitwidth: 12
                layout: !Layout
        ";
        let map = RegisterMap::from_yaml_str(yaml).unwrap();
        assert!(map.registers.get("REG").unwrap().layout.bitwidth == 12);

        let yaml = "
        name: DummyChip
        registers:
            REG: !Register
                adr: 0x1000
                layout: !Layout
        ";
        let err = RegisterMap::from_yaml_str(yaml).unwrap_err();
        println!("{}", err);
        assert!(format!("{}", err).contains("Unknown bitwidth:"));
    }

    #[test]
    fn test_bad_shared_refernces() {
        let yaml = "
        name: DummyChip
        registers:
            REG: !Register
                adr: 0x1000
                layout: !SharedLayout DoesNotExist
        ";
        let err = RegisterMap::from_yaml_str(yaml).unwrap_err();
        println!("{}", err);
        assert!(format!("{}", err).contains("Shared layout 'DoesNotExist' not found."));

        let yaml = "
        name: DummyChip
        registers:
            REG: !Register
                bitwidth: 8
                adr: 0x1000
                layout: !Layout
                    A:
                        bits: [0]
                        accepts: !SharedEnum DoesNotExist
        ";
        let err = RegisterMap::from_yaml_str(yaml).unwrap_err();
        println!("{}", err);
        assert!(format!("{}", err).contains("Shared enum 'DoesNotExist' not found."));

        let yaml = "
        name: DummyChip
        registers:
            REG: !Register
                bitwidth: 8
                adr: 0x1000
                layout: !Layout
                    A:
                        bits: [0]
                        accepts: !SharedLayout DoesNotExist
        ";
        let err = RegisterMap::from_yaml_str(yaml).unwrap_err();
        println!("{}", err);
        assert!(format!("{}", err).contains("Shared layout 'DoesNotExist' not found."));
    }

    #[test]
    fn test_convert_nested_layouts() {
        let yaml = "
        name: DummyChip
        layouts:
            REG:
                bitwidth: 8
                layout:
                    A:
                        bits: [\"7-0\"]
                        accepts: !Layout
                            B:
                                bits: [\"6-0\"]
                                accepts: !Layout
                                    C:
                                        bits: [\"6-0\"]
        ";
        let map = RegisterMap::from_yaml_str(yaml).unwrap();
        let layout_a = map.layouts.get("A").unwrap();
        let field_b = layout_a.fields.get("B").unwrap();
        let layout_b = match &field_b.accepts {
            FieldType::Layout(l) => l,
            _ => panic!(),
        };
        let field_c = layout_b.fields.get("C").unwrap();
        assert_eq!(field_c.name, "C");
    }

    #[test]
    fn test_convert_shared_layout() {
        let yaml = "
        name: DummyChip
        layouts:
            My_CoOl_LayOut:
                bitwidth: 8
                layout:
                    FIELD_A:
                        bits: [0]
                    FIELD_B:
                        bits: [1]
        registers:
            A: !Register
                adr: 0x01
                layout: !SharedLayout My_CoOl_LayOut
        ";
        let map = RegisterMap::from_yaml_str(yaml).unwrap();
        let reg_a_fields = map
            .registers
            .get("A")
            .unwrap()
            .layout
            .fields
            .keys()
            .cloned()
            .collect::<Vec<String>>();
        assert_eq!(reg_a_fields, vec![String::from("FIELD_A"), String::from("FIELD_B")]);

        let yaml = "
        name: DummyChip
        layouts:
            My_CoOl_LayOut:
                bitwidth: 2
                layout:
                    FIELD_A:
                        bits: [0]
                    FIELD_B:
                        bits: [1]
        registers:
            A: !Register
                adr: 0x01
                bitwidth: 8
                layout: !Layout
                    PARENT_FIELD:
                        bits: [0,1]
                        accepts: !SharedLayout My_CoOl_LayOut
        ";
        let map = RegisterMap::from_yaml_str(yaml).unwrap();
        let parent_field = map
            .registers
            .get("A")
            .unwrap()
            .layout
            .fields
            .get("PARENT_FIELD")
            .unwrap();
        let layout_fields = match &parent_field.accepts {
            FieldType::Layout(l) => l,
            _ => panic!(),
        }
        .fields
        .keys()
        .cloned()
        .collect::<Vec<String>>();
        assert_eq!(layout_fields, vec![String::from("FIELD_A"), String::from("FIELD_B")]);
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

    #[test]
    fn test_convert_docs() {
        let doc = convert_docs(&Some("".to_string()), "bt").unwrap();
        assert_eq!(doc, Docs { brief: None, doc: None });

        let doc = convert_docs(&Some("brief".to_string()), "bt").unwrap();
        assert_eq!(
            doc,
            Docs {
                brief: Some(String::from("brief")),
                doc: None,
            }
        );

        let doc = convert_docs(&Some("brief\n\n\ndoc".to_string()), "bt").unwrap();
        assert_eq!(
            doc,
            Docs {
                brief: Some(String::from("brief")),
                doc: Some(String::from("doc")),
            }
        );

        let doc = convert_docs(&Some("doc1\ndoc\n\ndoc2\ndoc3\n\n\n\n".to_string()), "bt").unwrap();
        assert_eq!(
            doc,
            Docs {
                brief: None,
                doc: Some(String::from("doc1\ndoc\n\ndoc2\ndoc3")),
            }
        );
    }
}
