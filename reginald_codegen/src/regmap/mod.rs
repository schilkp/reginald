use std::{
    collections::{BTreeMap, HashSet},
    io,
    ops::RangeInclusive,
    path::PathBuf,
    rc::Rc,
};

use crate::bits::{
    bitmask_from_range, bitmask_from_width, mask_to_bit_ranges, mask_to_bits, msb_pos, unpositioned_mask,
};
use crate::{bits::lsb_pos, error::Error, utils::numbers_as_ranges};

use self::convert::convert_map;

mod convert;
mod listing;
mod validate;

pub type TypeValue = u64;
pub type TypeBitwidth = u32;
pub const MAX_BITWIDTH: TypeBitwidth = 64;
pub type TypeAdr = u64;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AccessMode {
    R,
    W,
}

pub type Access = Vec<AccessMode>;

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Docs {
    pub brief: Option<String>,
    pub doc: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct EnumEntry {
    pub name: String,
    pub value: TypeValue,
    pub docs: Docs,
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Enum {
    pub name: String,
    pub is_shared: bool,
    pub docs: Docs,
    pub entries: BTreeMap<String, EnumEntry>,
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub enum FieldType {
    #[default]
    UInt,
    Bool,
    LocalEnum(Enum),
    SharedEnum(Rc<Enum>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AlwaysWrite {
    pub mask: TypeValue,
    pub value: TypeValue,
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Field {
    pub name: String,
    pub mask: TypeValue,
    pub access: Option<Access>,
    pub docs: Docs,
    pub accepts: FieldType,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Register {
    pub name: String,
    pub fields: BTreeMap<String, Field>,
    pub bitwidth: TypeBitwidth,
    pub from_explicit_listing_block: bool,
    pub adr: Option<TypeAdr>,
    pub always_write: Option<AlwaysWrite>,
    pub reset_val: Option<TypeValue>,
    pub docs: Docs,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Instance {
    pub name: String,
    pub adr: Option<TypeAdr>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegisterBlock {
    pub name: String,
    pub instances: BTreeMap<String, Instance>,
    pub docs: Docs,
    pub from_explicit_listing_block: bool,
    pub register_templates: BTreeMap<String, Register>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegisterMap {
    pub from_file: Option<PathBuf>,
    pub map_name: String,
    pub docs: Docs,
    pub register_blocks: BTreeMap<String, RegisterBlock>,
    pub shared_enums: BTreeMap<String, Rc<Enum>>,
    pub note: Option<String>,
    pub author: Option<String>,
}

impl Docs {
    pub fn is_empty(&self) -> bool {
        self.brief.is_none() && self.doc.is_none()
    }

    pub fn as_multiline(&self, prefix: &str) -> String {
        let mut out = String::new();
        if let Some(brief) = &self.brief {
            out.push_str(prefix);
            out.push_str(brief);
            out.push('\n');
        }
        if let Some(doc) = &self.doc {
            for line in doc.lines() {
                out.push_str(prefix);
                out.push_str(line);
                out.push('\n');
            }
        }

        out
    }

    pub fn as_twoline(&self, prefix: &str) -> String {
        let mut out = String::new();
        if let Some(brief) = &self.brief {
            out.push_str(prefix);
            out.push_str(brief);
            out.push('\n');
        }
        if let Some(doc) = &self.doc {
            out.push_str(prefix);
            for line in doc.lines() {
                out.push_str(line);
                out.push(' ');
            }
            out.push('\n');
        }

        out
    }
}

impl Enum {
    /// Check if enum can represent every possible value that fits into 'mask'
    pub fn can_unpack_mask(&self, unpos_mask: TypeValue) -> bool {
        // All enum values that fit into the mask:
        let enum_vals: HashSet<u64> = self
            .entries
            .values()
            .map(|x| x.value)
            .filter(|x| x & !unpos_mask == 0)
            .collect();

        // Number of values the mask can represent:
        let mask_bit_count = mask_to_bits(unpos_mask).len();
        let mask_vals_count = 2_usize.pow(mask_bit_count.try_into().unwrap());

        mask_vals_count == enum_vals.len()
    }

    /// Minimum bitwidth required to represent all values in this enum.
    pub fn min_bitdwith(&self) -> TypeBitwidth {
        msb_pos(self.max_value()) + 1
    }

    /// Check if enum can repreent every possible value of a N-bit number, where N is the
    /// minimum bitwidth of this enum.
    pub fn can_unpack_min_bitwidth(&self) -> bool {
        self.can_unpack_mask(bitmask_from_width(self.min_bitdwith()))
    }

    pub fn max_value(&self) -> TypeValue {
        self.entries.values().map(|x| x.value).max().unwrap_or(0)
    }

    pub fn decode(&self, val: TypeValue) -> Result<String, Error> {
        self.entries
            .values()
            .find(|x| x.value == val)
            .map(|x| x.name.clone())
            .ok_or(Error::GeneratorError(format!("Enum '{}' cannot represent value 0x{:X}.", self.name, val)))
    }
}

pub enum DecodedField {
    UInt(TypeValue),
    Bool(bool),
    EnumEntry(String),
}

impl Field {
    pub fn accepts_enum(&self) -> bool {
        match &self.accepts {
            FieldType::UInt | FieldType::Bool => false,
            FieldType::LocalEnum(_) | FieldType::SharedEnum(_) => true,
        }
    }

    pub fn get_enum(&self) -> Option<&Enum> {
        match &self.accepts {
            FieldType::UInt | FieldType::Bool => None,
            FieldType::LocalEnum(e) => Some(e),
            FieldType::SharedEnum(e) => Some(e),
        }
    }

    pub fn enum_entries(&self) -> Option<impl Iterator<Item = &EnumEntry>> {
        match &self.accepts {
            FieldType::UInt | FieldType::Bool => None,
            FieldType::LocalEnum(local_enum) => Some(local_enum.entries.values()),
            FieldType::SharedEnum(shared_enum) => Some(shared_enum.entries.values()),
        }
    }

    pub fn can_always_unpack(&self) -> bool {
        match &self.accepts {
            FieldType::UInt | FieldType::Bool => true,
            FieldType::LocalEnum(local_enum) => local_enum.can_unpack_mask(unpositioned_mask(self.mask)),
            FieldType::SharedEnum(shared_enum) => shared_enum.can_unpack_mask(unpositioned_mask(self.mask)),
        }
    }

    pub fn decode_unpositioned_value(&self, val: TypeValue) -> Result<DecodedField, Error> {
        self.decode_value(val >> (lsb_pos(self.mask)))
    }

    pub fn decode_value(&self, val: TypeValue) -> Result<DecodedField, Error> {
        let val = val & unpositioned_mask(self.mask);
        match &self.accepts {
            FieldType::UInt => Ok(DecodedField::UInt(val)),
            FieldType::Bool => Ok(DecodedField::Bool(val != 0)),
            FieldType::LocalEnum(e) => Ok(DecodedField::EnumEntry(e.decode(val)?)),
            FieldType::SharedEnum(e) => Ok(DecodedField::EnumEntry(e.decode(val)?)),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RegisterBitrangeContent<'a> {
    Empty,
    Field {
        field: &'a Field,
        is_split: bool,
        subfield_mask: TypeValue,
    },
    AlwaysWrite {
        val: TypeValue,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegisterBitrange<'a> {
    pub bits: RangeInclusive<TypeBitwidth>,
    pub content: RegisterBitrangeContent<'a>,
}

impl Register {
    pub fn split_to_bitranges(&self) -> Vec<RegisterBitrange> {
        let mut result = vec![];

        if let Some(AlwaysWrite { mask, value }) = &self.always_write {
            let ranges = mask_to_bit_ranges(*mask);
            for range in &ranges {
                let range_value = (value & bitmask_from_range(range)) >> range.start();
                result.push(RegisterBitrange {
                    bits: range.clone(),
                    content: RegisterBitrangeContent::AlwaysWrite { val: range_value },
                });
            }
        }

        for field in self.fields.values() {
            let ranges = mask_to_bit_ranges(field.mask);
            for range in &ranges {
                let subfield_mask = bitmask_from_range(range) >> lsb_pos(field.mask);

                result.push(RegisterBitrange {
                    bits: range.clone(),
                    content: RegisterBitrangeContent::Field {
                        field,
                        is_split: ranges.len() > 1,
                        subfield_mask,
                    },
                });
            }
        }

        let empty_bits: Vec<TypeBitwidth> = (0..self.bitwidth).filter(|x| self.empty_at_bitpos(*x)).collect();

        for range in numbers_as_ranges(empty_bits) {
            result.push(RegisterBitrange {
                bits: range.clone(),
                content: RegisterBitrangeContent::Empty,
            });
        }

        result.sort_by_key(|x| *(x.bits.start()));

        result
    }

    pub fn field_at_bitpos(&self, bitpos: TypeBitwidth) -> Option<&Field> {
        for field in self.fields.values() {
            if (1 << bitpos) & field.mask != 0 {
                return Some(field);
            }
        }

        self.fields.values().find(|&field| (1 << bitpos) & field.mask != 0)
    }

    pub fn always_write_at_bitpos(&self, bitpos: TypeBitwidth) -> Option<TypeValue> {
        if let Some(AlwaysWrite { mask, value }) = self.always_write {
            if (1 << bitpos) & mask != 0 {
                return Some((value >> bitpos) & 1);
            }
        }

        None
    }

    pub fn empty_at_bitpos(&self, bitpos: TypeBitwidth) -> bool {
        self.always_write_at_bitpos(bitpos).is_none() && self.field_at_bitpos(bitpos).is_none()
    }

    pub fn can_always_unpack(&self) -> bool {
        for field in self.fields.values() {
            if !field.can_always_unpack() {
                return false;
            };
        }
        true
    }

    pub fn name_in_block(&self, block: &RegisterBlock) -> String {
        if self.name.is_empty() {
            String::from(&block.name)
        } else {
            format!("{}_{}", block.name, self.name)
        }
    }

    pub fn name_in_instance(&self, instance: &Instance) -> String {
        if self.name.is_empty() {
            String::from(&instance.name)
        } else {
            format!("{}_{}", instance.name, self.name)
        }
    }

    pub fn width_bytes(&self) -> TypeBitwidth {
        (self.bitwidth + 7) / 8
    }
}

impl RegisterMap {
    pub fn from_file(path: &PathBuf) -> Result<Self, Error> {
        let inp = std::fs::File::open(path)?;
        let ext = path.extension().and_then(|x| x.to_str()).map(str::to_lowercase);
        let listing = match ext {
            Some(ext) if ext == "yaml" || ext == "yml" => listing::RegisterMap::from_yaml(inp)?,
            Some(ext) if ext == "json" || ext == "hjson" => listing::RegisterMap::from_hjson(inp)?,
            _ => {
                eprintln!("Unknown input file extension. Assuming YAML.");
                listing::RegisterMap::from_yaml(inp)?
            }
        };
        convert_map(&listing, &Some(path.clone()))
    }

    pub fn from_yaml<R>(inp: R) -> Result<Self, Error>
    where
        R: io::Read,
    {
        let listing = listing::RegisterMap::from_yaml(inp)?;
        convert_map(&listing, &None)
    }

    pub fn from_yaml_str(inp: &str) -> Result<Self, Error> {
        Self::from_yaml(inp.as_bytes())
    }

    pub fn from_hjson<R>(inp: R) -> Result<Self, Error>
    where
        R: io::Read,
    {
        let listing = listing::RegisterMap::from_hjson(inp)?;
        convert_map(&listing, &None)
    }

    pub fn from_hjson_str(inp: &str) -> Result<Self, Error> {
        Self::from_hjson(inp.as_bytes())
    }

    pub fn max_register_width(&self) -> TypeBitwidth {
        let mut max_width = 0;
        for block in self.register_blocks.values() {
            for template in block.register_templates.values() {
                max_width = TypeBitwidth::max(max_width, template.bitwidth);
            }
        }

        max_width
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RegisterOrigin<'a> {
    Register,
    RegisterBlockInstance {
        block: &'a RegisterBlock,
        instance: Instance,
        offset_from_block_start: Option<TypeAdr>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PhysicalRegister<'a> {
    pub name: String,
    pub absolute_adr: Option<TypeAdr>,
    pub origin: RegisterOrigin<'a>,
    pub template: &'a Register,
}

impl RegisterMap {
    pub fn physical_registers(&self) -> Vec<PhysicalRegister> {
        let mut result = vec![];
        for block in self.register_blocks.values() {
            for template in block.register_templates.values() {
                for instance in block.instances.values() {
                    let absolute_adr = match (instance.adr, template.adr) {
                        (Some(start), Some(ofs)) => Some(start + ofs),
                        (_, _) => None,
                    };

                    let origin = if template.from_explicit_listing_block {
                        RegisterOrigin::RegisterBlockInstance {
                            block,
                            instance: instance.clone(),
                            offset_from_block_start: template.adr,
                        }
                    } else {
                        RegisterOrigin::Register
                    };

                    let name = template.name_in_instance(instance);

                    result.push(PhysicalRegister {
                        name,
                        absolute_adr,
                        origin,
                        template,
                    });
                }
            }
        }
        result.sort_by_key(|x| x.absolute_adr.unwrap_or(TypeAdr::MAX));
        result
    }
}

pub fn access_string(v: &Access) -> String {
    let mut result = String::new();
    for i in v {
        match i {
            AccessMode::R => result.push('R'),
            AccessMode::W => result.push('W'),
        }
    }
    result
}

#[cfg(test)]
pub fn assert_regmap_eq(left: RegisterMap, right: RegisterMap) {
    use std::iter::zip;

    use pretty_assertions::assert_eq;
    assert_eq!(left.map_name, right.map_name);
    assert_eq!(left.docs, right.docs);
    assert_eq!(left.author, right.author);
    assert_eq!(left.note, right.note);
    for (left, right) in zip(&left.register_blocks, &right.register_blocks) {
        assert_eq!(left, right);
    }
    assert_eq!(left.register_blocks, right.register_blocks);
    for (name, left) in left.shared_enums {
        assert_eq!(left, *right.shared_enums.get(&name).unwrap());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_to_bitranges() {
        use pretty_assertions::assert_eq;
        let reg = Register {
            name: "TestReg".into(),
            bitwidth: 16,
            from_explicit_listing_block: false,
            adr: None,
            always_write: Some(AlwaysWrite {
                mask: 0b01110110,
                value: 0b001010100,
            }),
            reset_val: None,
            docs: Docs::default(),
            fields: BTreeMap::from([
                (
                    "A".into(),
                    Field {
                        name: "A".into(),
                        mask: 0b1001,
                        ..Default::default()
                    },
                ),
                (
                    "B".into(),
                    Field {
                        name: "B".into(),
                        mask: 0xF000,
                        ..Default::default()
                    },
                ),
            ]),
        };

        let ranges = reg.split_to_bitranges();

        assert_eq!(
            ranges,
            vec![
                RegisterBitrange {
                    bits: 0..=0,
                    content: RegisterBitrangeContent::Field {
                        field: &reg.fields["A".into()],
                        is_split: true,
                        subfield_mask: 0b1
                    }
                },
                RegisterBitrange {
                    bits: 1..=2,
                    content: RegisterBitrangeContent::AlwaysWrite { val: 0b10 }
                },
                RegisterBitrange {
                    bits: 3..=3,
                    content: RegisterBitrangeContent::Field {
                        field: &reg.fields["A".into()],
                        is_split: true,
                        subfield_mask: 0b1000
                    }
                },
                RegisterBitrange {
                    bits: 4..=6,
                    content: RegisterBitrangeContent::AlwaysWrite { val: 0b101 }
                },
                RegisterBitrange {
                    bits: 7..=11,
                    content: RegisterBitrangeContent::Empty
                },
                RegisterBitrange {
                    bits: 12..=15,
                    content: RegisterBitrangeContent::Field {
                        field: &reg.fields["B".into()],
                        is_split: false,
                        subfield_mask: 0b1111
                    }
                },
            ]
        );
    }

    #[test]
    fn test_enum_can_unpack_mask() {
        let e = Enum {
            entries: BTreeMap::from_iter(vec![0, 1, 2, 3, 4, 5, 6, 7].into_iter().map(|x| {
                (
                    format!("E{x}"),
                    EnumEntry {
                        value: x,
                        ..Default::default()
                    },
                )
            })),
            ..Default::default()
        };

        for val in 0..8 {
            println!("0b{val:b}");
            assert_eq!(e.can_unpack_mask(val), true);
        }

        for val in 0..8 {
            println!("base val: 0b{val:b}");
            assert_eq!(e.can_unpack_mask(0b1000 | val), false);
            assert_eq!(e.can_unpack_mask(0b110101000 | val), false);
        }
    }

    #[test]
    fn test_enum_min_bitwidth() {
        let mut e = Enum {
            entries: BTreeMap::from_iter(vec![0, 1, 2, 3, 4, 5, 6, 7].into_iter().map(|x| {
                (
                    format!("E{x}"),
                    EnumEntry {
                        value: x,
                        ..Default::default()
                    },
                )
            })),
            ..Default::default()
        };

        assert_eq!(e.min_bitdwith(), 3);

        e.entries.remove("E4").unwrap();

        assert_eq!(e.min_bitdwith(), 3);

        let e = Enum {
            entries: BTreeMap::from([(
                String::from("E0"),
                EnumEntry {
                    value: 0,
                    ..Default::default()
                },
            )]),
            ..Default::default()
        };

        assert_eq!(e.min_bitdwith(), 1);
    }

    #[test]
    fn test_enum_can_unpack_min_bitwidth() {
        let mut e = Enum {
            entries: BTreeMap::from_iter(vec![0, 1, 2, 3, 4, 5, 6, 7].into_iter().map(|x| {
                (
                    format!("E{x}"),
                    EnumEntry {
                        value: x,
                        ..Default::default()
                    },
                )
            })),
            ..Default::default()
        };

        assert_eq!(e.can_unpack_min_bitwidth(), true);

        e.entries.remove("E4").unwrap();
        assert_eq!(e.can_unpack_min_bitwidth(), false);

        let e = Enum {
            entries: BTreeMap::from([(
                String::from("E0"),
                EnumEntry {
                    value: 0,
                    ..Default::default()
                },
            )]),
            ..Default::default()
        };
        assert_eq!(e.can_unpack_min_bitwidth(), false);
    }
}
