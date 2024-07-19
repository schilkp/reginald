mod convert;
mod listing;
mod validate;

use std::{
    collections::{BTreeMap, HashSet},
    io,
    ops::{Deref, RangeInclusive},
    path::PathBuf,
    rc::Rc,
};

use reginald_utils::numbers_as_ranges;
use serde::{Deserialize, Serialize};

use crate::bits::{
    bitmask_from_range, bitmask_from_width, bitwidth_to_width_bytes, mask_to_bit_ranges, mask_to_bit_ranges_str,
    mask_to_bits, msb_pos, unpositioned_mask,
};
use crate::{bits::lsb_pos, error::Error};

use self::convert::convert_map;

// ==== Basic Types ============================================================

pub type TypeValue = u64;
pub type TypeBitwidth = u32;
pub const MAX_BITWIDTH: TypeBitwidth = 64;
pub type TypeAdr = u64;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccessMode {
    R,
    W,
}

pub type Access = Vec<AccessMode>;

#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Docs {
    pub brief: Option<String>,
    pub doc: Option<String>,
}

// ==== Enums ==================================================================

#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct EnumEntry {
    pub name: String,
    pub value: TypeValue,
    pub docs: Docs,
}

#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Enum {
    pub name: String,
    pub docs: Docs,
    pub is_local: bool,
    pub entries: BTreeMap<String, EnumEntry>,
}

// ==== Layouts ================================================================

#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum FieldType {
    #[default]
    UInt,
    Bool,
    Fixed(TypeValue),
    Enum(Rc<Enum>),
    Layout(Rc<Layout>),
}

#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct LayoutField {
    pub name: String,
    pub mask: TypeValue,
    pub docs: Docs,
    pub accepts: FieldType,
    pub access: Option<Access>,
}

#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Layout {
    pub name: String,
    pub docs: Docs,
    pub is_local: bool,
    pub bitwidth: TypeBitwidth,

    pub fields: BTreeMap<String, LayoutField>,
}

// ==== Registers ==============================================================

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegisterOrigin {
    pub block: String,
    pub instance: String,
    pub block_member: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Register {
    pub name: String,
    pub docs: Docs,

    pub adr: TypeAdr,
    pub reset_val: Option<TypeValue>,

    pub layout: Rc<Layout>,

    pub from_block: Option<RegisterOrigin>,
}

// ==== Register Blocks ========================================================

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegisterBlockInstance {
    pub name: String,
    pub adr: TypeAdr,
    pub docs: Docs,

    pub registers: BTreeMap<String, Rc<Register>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegisterBlockMember {
    pub name: String,
    pub name_raw: String,
    pub docs: Docs,

    pub offset: TypeAdr,

    pub layout: Rc<Layout>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegisterBlock {
    pub name: String,
    pub docs: Docs,
    pub instances: BTreeMap<String, RegisterBlockInstance>,

    pub members: BTreeMap<String, Rc<RegisterBlockMember>>,
}

// ==== Register Map ===========================================================

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Defaults {
    pub layout_bitwidth: Option<TypeBitwidth>,
    pub field_access_mode: Option<Access>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegisterMap {
    pub from_file: Option<PathBuf>,
    pub name: String,
    pub docs: Docs,
    pub notice: Option<String>,
    pub author: Option<String>,
    pub defaults: Defaults,

    // All enums:
    pub enums: BTreeMap<String, Rc<Enum>>,

    // All layouts:
    pub layouts: BTreeMap<String, Rc<Layout>>,

    // All registers:
    pub registers: BTreeMap<String, Rc<Register>>,

    // Register blocks
    pub register_blocks: BTreeMap<String, RegisterBlock>,
}

// ==== Impls ==================================================================

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
        let mask_vals_count = 2_u128.pow(mask_bit_count.try_into().unwrap());

        let enum_vals_count: u128 = enum_vals
            .len()
            .try_into()
            .expect("HashSet holding u64 cannot have more than u64::MAX entries");

        mask_vals_count == enum_vals_count
    }

    /// Minimum bitwidth required to represent all values in this enum.
    pub fn min_bitdwith(&self) -> TypeBitwidth {
        msb_pos(self.max_value()) + 1
    }

    /// Minimum number of bytes required to represent all values in this enum.
    pub fn min_width_bytes(&self) -> TypeBitwidth {
        bitwidth_to_width_bytes(self.min_bitdwith())
    }

    /// Check if enum can repreent every possible value of a N-bit number, where N is the
    /// minimum bitwidth of this enum.
    pub fn can_unpack_min_bitwidth(&self) -> bool {
        self.can_unpack_mask(bitmask_from_width(self.min_bitdwith()))
    }

    pub fn max_value(&self) -> TypeValue {
        self.entries.values().map(|x| x.value).max().unwrap_or(0)
    }

    pub fn can_unpack_masked(&self) -> bool {
        self.can_unpack_mask(self.occupied_bits())
    }

    pub fn decode(&self, val: TypeValue) -> Result<String, Error> {
        self.entries
            .values()
            .find(|x| x.value == val)
            .map(|x| x.name.clone())
            .ok_or(Error::GeneratorError(format!("Enum '{}' cannot represent value 0x{:X}.", self.name, val)))
    }

    pub fn occupied_bits(&self) -> TypeValue {
        self.entries.values().map(|x| x.value).reduce(|a, b| a | b).unwrap_or(0)
    }
}

pub enum DecodedField {
    UInt(TypeValue),
    Fixed { val: TypeValue, is_correct: bool },
    Bool(bool),
    EnumEntry(String),
}

impl LayoutField {
    pub fn can_always_unpack(&self) -> bool {
        match &self.accepts {
            FieldType::UInt => true,
            FieldType::Bool => true,
            FieldType::Enum(e) => e.can_unpack_mask(unpositioned_mask(self.mask)),
            FieldType::Fixed(_) => true,
            FieldType::Layout(l) => l.can_always_unpack(),
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
            FieldType::Enum(e) => Ok(DecodedField::EnumEntry(e.decode(val)?)),
            FieldType::Fixed(expected) => Ok(DecodedField::Fixed {
                val,
                is_correct: *expected == val,
            }),
            FieldType::Layout(_) => panic!("Decoding nested layouts is not implemented"),
        }
    }

    pub fn contains_content(&self) -> bool {
        !matches!(self.accepts, FieldType::Fixed(_))
    }
}

impl Layout {
    pub fn field_at_bitpos(&self, bitpos: TypeBitwidth) -> Option<&LayoutField> {
        for field in self.fields.values() {
            if (1 << bitpos) & field.mask != 0 {
                return Some(field);
            }
        }

        self.fields.values().find(|&field| (1 << bitpos) & field.mask != 0)
    }

    pub fn empty_at_bitpos(&self, bitpos: TypeBitwidth) -> bool {
        self.field_at_bitpos(bitpos).is_none()
    }

    pub fn can_always_unpack(&self) -> bool {
        for field in self.fields.values() {
            if !field.can_always_unpack() {
                return false;
            };
        }
        true
    }

    pub fn occupied_mask(&self) -> TypeValue {
        let mut mask: TypeValue = 0;
        for field in self.fields.values() {
            mask |= field.mask;
        }
        mask
    }

    pub fn empty_mask(&self) -> TypeValue {
        !self.occupied_mask() & (bitmask_from_width(self.bitwidth))
    }

    pub fn fixed_bits_mask(&self) -> TypeValue {
        let mut mask: TypeValue = 0;
        for field in self.fields.values() {
            if !matches!(field.accepts, FieldType::Fixed(_)) {
                continue;
            }
            mask |= field.mask;
        }
        mask
    }

    pub fn fixed_bits_val(&self) -> TypeValue {
        let mut val: TypeValue = 0;
        for field in self.fields.values() {
            if let FieldType::Fixed(fixed_val) = field.accepts {
                val |= fixed_val << lsb_pos(field.mask);
            }
        }
        val
    }

    pub fn contains_fixed_bits(&self) -> bool {
        self.fixed_bits_mask() != 0
    }

    /// Iterator over all enums local to this layout (excluding local
    /// enums in nested layouts)
    pub fn local_enums(&self) -> impl Iterator<Item = &Enum> {
        self.fields
            .values()
            .filter_map(|x| match &x.accepts {
                FieldType::Enum(e) => Some(e.deref()),
                _ => None,
            })
            .filter(|x| x.is_local)
    }

    /// Iterator over all enums local to this layout (including local
    /// enums in nested layouts)
    pub fn nested_local_enums(&self) -> impl Iterator<Item = &Enum> {
        let mut enums: Vec<&Enum> = vec![];

        for layout in self.nested_local_layouts() {
            enums.extend(layout.local_enums());
        }

        enums.extend(
            self.fields
                .values()
                .filter_map(|x| match &x.accepts {
                    FieldType::Enum(e) => Some(e.deref()),
                    _ => None,
                })
                .filter(|x| x.is_local),
        );

        enums.into_iter()
    }

    /// Iterator over all layouts local to this layout (excluding local
    /// layouts in nested layouts)
    pub fn local_layouts(&self) -> impl Iterator<Item = &Layout> {
        self.fields
            .values()
            .filter_map(|x| match &x.accepts {
                FieldType::Layout(l) => Some(l.deref()),
                _ => None,
            })
            .filter(|x| x.is_local)
    }

    /// Iterator over all layouts local to this layout (including local
    /// layouts in nested layouts), in dependency order.
    pub fn nested_local_layouts(&self) -> impl Iterator<Item = &Layout> {
        let mut layouts: Vec<&Layout> = vec![];

        for field in self
            .fields
            .values()
            .filter_map(|x| match &x.accepts {
                FieldType::Layout(l) => Some(l.deref()),
                _ => None,
            })
            .filter(|x| x.is_local)
        {
            layouts.extend(field.nested_local_layouts());
            layouts.push(field)
        }

        // Sanity assert that no layout has been included more than tonce:
        #[cfg(debug_assertions)]
        {
            let names: HashSet<String> = HashSet::from_iter(layouts.iter().map(|x| x.name.to_owned()));
            assert!(names.len() == layouts.len());
        }

        layouts.into_iter()
    }

    pub fn fields_with_content(&self) -> impl Iterator<Item = &LayoutField> {
        self.fields.values().filter(|x| x.contains_content())
    }

    pub fn width_bytes(&self) -> TypeBitwidth {
        bitwidth_to_width_bytes(self.bitwidth)
    }

    pub fn overview_text(&self, as_markdown: bool) -> String {
        let markdown_escape = |x: &str| {
            if as_markdown {
                "`".to_string() + x + "`"
            } else {
                x.to_string()
            }
        };

        let nested_fields = self.nested_fields();
        if nested_fields.is_empty() {
            return String::new();
        }

        let mut lines = vec![];

        for field in nested_fields {
            let name = markdown_escape(&field.name.join("."));

            let indent = field.name.len() - 1;
            let indent = String::from_iter(std::iter::repeat("  ").take(indent));

            let bits = markdown_escape(&format!("[{}]", mask_to_bit_ranges_str(field.mask)));

            let type_string = match &field.field.accepts {
                FieldType::UInt => String::from("(uint)"),
                FieldType::Bool => String::from("(bool)"),
                FieldType::Fixed(fix) => format!("(fixed: 0x{fix:x})"),
                FieldType::Enum(e) => format!("(enum {})", markdown_escape(&e.name)),
                FieldType::Layout(l) => format!("(layout {})", markdown_escape(&l.name)),
            };

            if let Some(brief) = &field.field.docs.brief {
                lines.push(format!("{indent}- {bits} {name} {type_string}: {brief}"))
            } else {
                lines.push(format!("{indent}- {bits} {name} {type_string}"))
            }

            if let Some(doc) = &field.field.docs.doc {
                for line in doc.lines() {
                    lines.push(format!("  {indent}{line}"))
                }
            }
        }

        lines.join("\n")
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegisterBitrangeContent<'a> {
    pub field: &'a LayoutField,
    pub is_split: bool,
    pub subfield_mask: TypeValue,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegisterBitrange<'a> {
    pub bits: RangeInclusive<TypeBitwidth>,
    pub content: Option<RegisterBitrangeContent<'a>>,
}

impl Layout {
    pub fn split_to_bitranges(&self) -> Vec<RegisterBitrange> {
        let mut result = vec![];

        for field in self.fields.values() {
            let ranges = mask_to_bit_ranges(field.mask);
            for range in &ranges {
                let subfield_mask = bitmask_from_range(range) >> lsb_pos(field.mask);

                result.push(RegisterBitrange {
                    bits: range.clone(),
                    content: Some(RegisterBitrangeContent {
                        field,
                        is_split: ranges.len() > 1,
                        subfield_mask,
                    }),
                });
            }
        }

        let empty_bits: Vec<TypeBitwidth> = (0..self.bitwidth).filter(|x| self.empty_at_bitpos(*x)).collect();

        for range in numbers_as_ranges(empty_bits) {
            result.push(RegisterBitrange {
                bits: range.clone(),
                content: None,
            });
        }

        result.sort_by_key(|x| *(x.bits.start()));

        result
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FlattenedLayoutField  {
    pub name: Vec<String>,
    pub mask: TypeValue,
    pub field: LayoutField,
}

impl Layout {
    pub fn nested_fields(&self) -> Vec<FlattenedLayoutField> {
        let mut result = vec![];

        for field in self.fields.values() {
            let field_define = FlattenedLayoutField {
                name: vec![field.name.clone()],
                mask: field.mask,
                field: field.clone(),
            };

            if let FieldType::Layout(sublayout) = &field.accepts {
                result.push(field_define);

                // Field contains another nested layout. flattened_fieldss:
                let mut sublayout_defines = sublayout.nested_fields();
                sublayout_defines.sort_by_key(|x| lsb_pos(x.mask));

                // Adjust them by prefixing them with the name of the enclosing field, and shifting all
                // masks into the position of the enclosing field:
                for sublayout_define in sublayout_defines {
                    let mut sublayout_field = sublayout_define.clone();
                    sublayout_field.name.insert(0, field.name.clone());
                    sublayout_field.mask <<= lsb_pos(field.mask);
                    result.push(sublayout_field);
                }
            } else {
                result.push(field_define);
            }
        }

        result.sort_by_key(|x| lsb_pos(x.mask));
        result
    }

    pub fn nested_fields_with_content(&self) -> Vec<FlattenedLayoutField> {
        self.nested_fields()
            .into_iter()
            .filter(|x| x.field.contains_content())
            .collect()
    }

    pub fn flattened_fields_with_content(&self) -> Vec<FlattenedLayoutField> {
        self.nested_fields()
            .into_iter()
            .filter(|x| !matches!(x.field.accepts, FieldType::Layout(_)))
            .filter(|x| x.field.contains_content())
            .collect()
    }

    pub fn flattened_fields(&self) -> Vec<FlattenedLayoutField> {
        self.nested_fields()
            .into_iter()
            .filter(|x| !matches!(x.field.accepts, FieldType::Layout(_)))
            .collect()
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

        for register in self.registers.values() {
            max_width = TypeBitwidth::max(max_width, register.layout.bitwidth);
        }

        max_width
    }

    pub fn shared_enums(&self) -> impl Iterator<Item = &Enum> {
        self.enums.values().filter(|x| !x.is_local).map(|x| x.deref())
    }

    pub fn shared_layouts(&self) -> impl Iterator<Item = &Layout> {
        self.layouts.values().filter(|x| !x.is_local).map(|x| x.deref())
    }

    pub fn individual_registers(&self) -> impl Iterator<Item = &Register> {
        self.registers
            .values()
            .filter(|x| x.from_block.is_none())
            .map(|x| x.deref())
    }

    pub fn layouts_in_dependency_order(&self) -> impl Iterator<Item = &Layout> {
        let mut layouts: Vec<&Layout> = vec![];

        for layout in self.shared_layouts() {
            layouts.extend(layout.nested_local_layouts());
            layouts.push(layout);
        }

        for register in self.individual_registers().filter(|x| x.layout.is_local) {
            layouts.extend(register.layout.nested_local_layouts());
            layouts.push(&register.layout);
        }

        for block in self.register_blocks.values() {
            for member in block.members.values().filter(|x| x.layout.is_local) {
                layouts.extend(member.layout.nested_local_layouts());
                layouts.push(&member.layout);
            }
        }

        // Sanity assert that every layout has been included, but only once:
        #[cfg(debug_assertions)]
        {
            assert!(layouts.len() == self.layouts.len());
            let names: HashSet<String> = HashSet::from_iter(layouts.iter().map(|x| x.name.to_owned()));
            assert!(names.len() == self.layouts.len());
        }

        layouts.into_iter()
    }
}

pub fn access_str(access: &Access) -> String {
    access
        .iter()
        .map(|x| match x {
            AccessMode::R => "R",
            AccessMode::W => "W",
        })
        .collect::<Vec<&str>>()
        .join("/")
}

#[cfg(test)]
pub fn assert_regmap_eq(left: RegisterMap, right: RegisterMap) {
    use std::iter::zip;

    use pretty_assertions::assert_eq;
    assert_eq!(left.name, right.name);
    assert_eq!(left.docs, right.docs);
    assert_eq!(left.author, right.author);
    assert_eq!(left.notice, right.notice);
    for (left, right) in zip(left.enums.values(), right.enums.values()) {
        assert_eq!(left, right);
    }
    for (left, right) in zip(left.layouts.values(), right.layouts.values()) {
        assert_eq!(left, right);
    }
    for (left, right) in zip(left.registers.values(), right.registers.values()) {
        assert_eq!(left, right);
    }
    for (left, right) in zip(left.register_blocks.values(), right.register_blocks.values()) {
        assert_eq!(left, right);
    }

    // Catch-all if a new field gets added and the above is not updated :)
    assert_eq!(left, right);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_to_bitranges() {
        use pretty_assertions::assert_eq;
        let layout = Layout {
            bitwidth: 16,
            name: String::new(),
            docs: Docs::default(),
            is_local: false,
            fields: BTreeMap::from([
                (
                    "A".into(),
                    LayoutField {
                        name: "A".into(),
                        mask: 0b1001,
                        ..Default::default()
                    },
                ),
                (
                    "B".into(),
                    LayoutField {
                        name: "B".into(),
                        mask: 0xF000,
                        ..Default::default()
                    },
                ),
            ]),
        };
        let ranges = layout.split_to_bitranges();

        assert_eq!(
            ranges,
            vec![
                RegisterBitrange {
                    bits: 0..=0,
                    content: Some(RegisterBitrangeContent {
                        field: &layout.fields["A"],
                        is_split: true,
                        subfield_mask: 0b1
                    }),
                },
                RegisterBitrange {
                    bits: 1..=2,
                    content: None,
                },
                RegisterBitrange {
                    bits: 3..=3,
                    content: Some(RegisterBitrangeContent {
                        field: &layout.fields["A"],
                        is_split: true,
                        subfield_mask: 0b1000
                    })
                },
                RegisterBitrange {
                    bits: 4..=11,
                    content: None
                },
                RegisterBitrange {
                    bits: 12..=15,
                    content: Some(RegisterBitrangeContent {
                        field: &layout.fields["B"],
                        is_split: false,
                        subfield_mask: 0b1111
                    })
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
            assert!(e.can_unpack_mask(val));
        }

        for val in 0..8 {
            println!("base val: 0b{val:b}");
            assert!(!e.can_unpack_mask(0b1000 | val));
            assert!(!e.can_unpack_mask(0b110101000 | val));
        }

        assert!(!e.can_unpack_mask(TypeValue::MAX));
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

        assert!(e.can_unpack_min_bitwidth());

        e.entries.remove("E4").unwrap();
        assert!(!e.can_unpack_min_bitwidth());

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
        assert!(!e.can_unpack_min_bitwidth());
    }
}
