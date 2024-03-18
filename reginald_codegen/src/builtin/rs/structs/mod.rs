use std::fmt::Write;

use crate::{
    bits::{bitmask_from_width, mask_width, msb_pos},
    error::Error,
    indent_write::IndentWrite,
    regmap::{Enum, Field, FieldType, Register, RegisterBlock, RegisterMap, TypeBitwidth, TypeValue},
    utils::{
        byte_to_field_transform, field_to_byte_transform, filename, remove_wrapping_parens, Endianess, ShiftDirection,
    },
};
use clap::Parser;

use super::{generate_doc_comment, rs_const, rs_fitting_unsigned_type, rs_pascalcase, rs_snakecase};

// ====== Generator Opts =======================================================

#[derive(Debug, Clone)]
#[cfg_attr(feature = "cli", derive(Parser))]
pub struct GeneratorOpts {
    /// Rust type to use for register addresses.
    ///
    /// If none is specified, the smallest unsigned type capable of storing
    /// the largest address will be used.
    #[cfg_attr(feature = "cli", arg(long))]
    #[cfg_attr(feature = "cli", arg(action = clap::ArgAction::Set))]
    #[cfg_attr(feature = "cli", arg(verbatim_doc_comment))]
    pub address_type: Option<String>,

    /// Include static string error messages for unpacking errors.
    #[cfg_attr(feature = "cli", arg(long))]
    #[cfg_attr(feature = "cli", arg(action = clap::ArgAction::Set))]
    #[cfg_attr(feature = "cli", arg(default_value = "false"))]
    #[cfg_attr(feature = "cli", arg(verbatim_doc_comment))]
    pub unpacking_error_msg: bool,

    /// Wrap each register block into its own module.
    ///
    /// Prevents possible name collisions between registers, but makes
    /// accessing registers a little annoying. If disabling this does
    /// not cause naming conflicts for a given map, doing so is recommend.
    #[cfg_attr(feature = "cli", arg(long))]
    #[cfg_attr(feature = "cli", arg(value_name = "DERIVE"))]
    #[cfg_attr(feature = "cli", arg(action = clap::ArgAction::Set))]
    #[cfg_attr(feature = "cli", arg(default_value = "true"))]
    #[cfg_attr(feature = "cli", arg(verbatim_doc_comment))]
    pub register_block_mods: bool,

    /// Trait to derive on all register structs.
    ///
    /// May be given multiple times.
    #[cfg_attr(feature = "cli", arg(long))]
    #[cfg_attr(feature = "cli", arg(action = clap::ArgAction::Append))]
    #[cfg_attr(feature = "cli", arg(verbatim_doc_comment))]
    pub struct_derive: Vec<String>,

    /// Trait to derive on all enums.
    ///
    /// May be given multiple times. Note: All enums always derive
    /// the "Clone" and "Copy" traits.
    #[cfg_attr(feature = "cli", arg(long = "enum-derive"))]
    #[cfg_attr(feature = "cli", arg(value_name = "DERIVE"))]
    #[cfg_attr(feature = "cli", arg(action = clap::ArgAction::Append))]
    #[cfg_attr(feature = "cli", arg(verbatim_doc_comment))]
    pub raw_enum_derive: Vec<String>,

    /// Module should be 'use'ed at the top of the generated module.
    ///
    /// May be given multiple times.
    #[cfg_attr(feature = "cli", arg(long))]
    #[cfg_attr(feature = "cli", arg(action = clap::ArgAction::Append))]
    #[cfg_attr(feature = "cli", arg(verbatim_doc_comment))]
    pub add_use: Vec<String>,

    /// Use an external definition of the `ToBytes`/`FromBytes`/`TryFromBytes` traits,
    ///
    /// No trait definition are generated, and implementations of the traits refeer
    /// to `[prefix]ToBytes`, `[prefix]FromBytes`, and `[prefix]TryFromBytes`,
    /// where `[preifx]` is the value given to this flag.
    #[cfg_attr(feature = "cli", arg(long))]
    #[cfg_attr(feature = "cli", arg(action = clap::ArgAction::Set))]
    #[cfg_attr(feature = "cli", arg(verbatim_doc_comment))]
    pub external_traits: Option<String>,

    /// Generate `Default` implementations for all register structs, using the reset
    /// value - if given.
    #[cfg_attr(feature = "cli", arg(long))]
    #[cfg_attr(feature = "cli", arg(action = clap::ArgAction::Set))]
    #[cfg_attr(feature = "cli", arg(default_value = "true"))]
    #[cfg_attr(feature = "cli", arg(verbatim_doc_comment))]
    pub generate_defaults: bool,

    /// Generate `From/TryFrom/From` implementations that convert a register
    /// to/from the smallest rust unsigned integer value wide enough to hold the
    /// register, if one exists.
    #[cfg_attr(feature = "cli", arg(long))]
    #[cfg_attr(feature = "cli", arg(action = clap::ArgAction::Set))]
    #[cfg_attr(feature = "cli", arg(default_value = "true"))]
    #[cfg_attr(feature = "cli", arg(verbatim_doc_comment))]
    pub generate_uint_conversion: bool,
}

// ====== Generator ============================================================

pub fn generate(out: &mut dyn Write, map: &RegisterMap, opts: &GeneratorOpts) -> Result<(), Error> {
    // Determine address type: Use option override, or smallest
    // unsigned type that fits the largest address in the map.
    let physical_registers = map.physical_registers();
    let address_type = if let Some(address_type) = &opts.address_type {
        address_type.clone()
    } else {
        let max_addr = physical_registers
            .iter()
            .filter_map(|x| x.absolute_adr)
            .max()
            .unwrap_or(0);
        rs_fitting_unsigned_type(msb_pos(max_addr) + 1)?
    };

    // Wrap output in IndentWrite which handles (nested) indentation of lines:
    let mut out = IndentWrite::new(out, "    ");

    let mut enum_derives: Vec<String> = vec!["Clone".into(), "Copy".into()];
    enum_derives.extend(opts.raw_enum_derive.clone());

    // Generate
    let generator = Generator {
        opts: opts.clone(),
        enum_derives,
        address_type,
        map,
    };
    generator.generate(&mut out)?;
    out.flush()?;

    Ok(())
}

struct Generator<'a> {
    opts: GeneratorOpts,
    map: &'a RegisterMap,
    address_type: String,
    enum_derives: Vec<String>,
}

impl Generator<'_> {
    /// Generate complete file
    fn generate(&self, out: &mut IndentWrite) -> Result<(), Error> {
        // File header/preamble:
        self.generate_header(out)?;

        if self.opts.external_traits.is_none() {
            self.generate_traits(out)?;
        }

        // Shared enums:
        if !self.map.shared_enums.is_empty() {
            writeln!(out)?;
            writeln!(out, "// ==== Shared Enums: ====")?;
            for shared_enum in self.map.shared_enums.values() {
                self.generate_enum(out, shared_enum)?;
            }
        }

        // Registers:
        writeln!(out)?;
        writeln!(out, "// ==== Registers: ====")?;
        for block in self.map.register_blocks.values() {
            self.generate_register_block(out, block)?;
        }

        Ok(())
    }

    /// Generate file header
    fn generate_header(&self, out: &mut IndentWrite) -> Result<(), Error> {
        writeln!(out, "#![allow(clippy::unnecessary_cast)]")?;

        // Top doc comment:
        writeln!(out, "//! `{}` Registers", self.map.map_name)?;
        writeln!(out, "//!")?;

        // Generated-with-reginald note, including original file name if known:
        if let Some(input_file) = &self.map.from_file {
            writeln!(out, "//! Generated using reginald from `{}`.", filename(input_file)?)?;
        } else {
            writeln!(out, "//! Generated using reginald.")?;
        }

        // Indicate which generator was used:
        writeln!(out, "//! Generator: rs-structs")?;

        // Map top-level documentation:
        if !self.map.docs.is_empty() {
            writeln!(out, "//!")?;
            write!(out, "{}", self.map.docs.as_multiline("//! "))?;
        }

        // Map author and note:
        if let Some(author) = &self.map.author {
            writeln!(out, "//! ")?;
            writeln!(out, "//! Listing file author: {author}")?;
        }
        if let Some(note) = &self.map.note {
            writeln!(out, "//!")?;
            writeln!(out, "//! Listing file note:")?;
            for line in note.lines() {
                writeln!(out, "//!   {line}")?;
            }
        }

        if !self.opts.add_use.is_empty() {
            writeln!(out)?;
            for add_use in &self.opts.add_use {
                writeln!(out, "use {add_use};")?;
            }
        }

        Ok(())
    }

    /// Generate enum (both shared and inside field)
    fn generate_enum(&self, out: &mut IndentWrite, e: &Enum) -> Result<(), Error> {
        // Smallest uint type that can be used to represent the enum's content:
        let uint_type = rs_fitting_unsigned_type(e.min_bitdwith())?;

        writeln!(out)?;
        generate_doc_comment(out, &e.docs, "")?;

        // Enum derives:
        let derives = self.enum_derives.join(", ");
        writeln!(out, "#[derive({derives})]")?;

        // Enum proper:
        writeln!(out, "#[repr({uint_type})]")?;
        writeln!(out, "pub enum {} {{", rs_pascalcase(&e.name))?;
        for entry in e.entries.values() {
            generate_doc_comment(out, &entry.docs, "    ")?;
            writeln!(out, "    {} = 0x{:x},", rs_pascalcase(&entry.name), entry.value)?;
        }
        writeln!(out, "}}")?;

        // Enum impl for uint -> enum conversion:
        self.generate_enum_impl(out, e)?;

        Ok(())
    }

    fn generate_enum_impl(&self, out: &mut IndentWrite, e: &Enum) -> Result<(), Error> {
        // Smallest uint type that can be used to represent the enum's content:
        let uint_type = rs_fitting_unsigned_type(e.min_bitdwith())?;

        let enum_name = rs_pascalcase(&e.name);

        if e.can_unpack_min_bitwidth() {
            // If the enum can represent every value that fits into a N-bit value, where
            // N is its minimal bitwidth, implement a 'Try' wrapping conversion:
            writeln!(out)?;
            writeln!(out, "impl From<{uint_type}> for {enum_name} {{")?;
            writeln!(out, "    fn from(value: {uint_type}) -> Self {{")?;
            writeln!(out, "        match value & 0x{:X} {{", bitmask_from_width(e.min_bitdwith()))?;
            for entry in e.entries.values() {
                writeln!(out, "            0x{:X} => Self::{},", entry.value, rs_pascalcase(&entry.name))?;
            }
            writeln!(out, "            _ => unreachable!(),")?;
            writeln!(out, "        }}")?;
            writeln!(out, "    }}")?;
            writeln!(out, "}}")?;
        } else {
            // Otherwise, implement a try-from conversion:
            writeln!(out)?;
            writeln!(out, "impl TryFrom<{uint_type}> for {enum_name} {{")?;

            // Error type:
            if self.opts.unpacking_error_msg {
                writeln!(out, "    type Error = &'static str;")?;
            } else {
                writeln!(out, "    type Error = ();")?;
            }

            // Conversion:
            writeln!(out)?;
            writeln!(out, "    fn try_from(value: {uint_type}) -> Result<Self, Self::Error> {{")?;
            writeln!(out, "        match value {{")?;
            for entry in e.entries.values() {
                writeln!(out, "            0x{:X} => Ok(Self::{}),", entry.value, rs_pascalcase(&entry.name))?;
            }
            if self.opts.unpacking_error_msg {
                writeln!(out, "            _ => Err(\"{} unpack error\"),", rs_pascalcase(&e.name))?;
            } else {
                writeln!(out, "            _ => Err(()),")?;
            }
            writeln!(out, "        }}")?;
            writeln!(out, "    }}")?;

            writeln!(out, "}}")?;
        }

        Ok(())
    }

    /// Generate structs/enums/impls/consts for a register block.
    fn generate_register_block(&self, out: &mut IndentWrite, block: &RegisterBlock) -> Result<(), Error> {
        // Header comment.
        // If this block will be wrapped in a module, have the header be a doc
        // comment that is associated with that module. Otherwise have it be a normal
        // comment.
        let header_comment = if self.opts.register_block_mods { "///" } else { "//" };

        // Have appropriate doc comment dependiong on if this block origininated as an
        // individual listing or register block in the listing.
        writeln!(out)?;
        if block.from_explicit_listing_block {
            writeln!(out, "{header_comment} `{}` register block", block.name)?;
        } else {
            writeln!(out, "{header_comment} `{}` register", block.name)?;
        }
        if !block.docs.is_empty() {
            writeln!(out, "{header_comment}")?;
            write!(out, "{}", block.docs.as_multiline(&(header_comment.to_string() + " ")))?;
        }

        // If specified, wrap register block in a module, and indent all its content:
        if self.opts.register_block_mods {
            writeln!(out, "pub mod {} {{", rs_snakecase(&block.name))?;
            out.push_indent();
        }

        // Generate register block constants (starts of instances), if that information is not
        // redundant from the actual register addresses.
        if block.instances.len() > 1 && block.register_templates.len() > 1 {
            self.generate_register_block_consts(out, block)?;
        }

        // Consts, struct, and impls for every register template:
        for template in block.register_templates.values() {
            self.generate_register(out, block, template)?;
        }

        // Un-indent and end module if necessary.
        if self.opts.register_block_mods {
            out.pop_indent();
            writeln!(out, "}}")?;
        }

        Ok(())
    }

    // Register block consts (instance addresses)
    fn generate_register_block_consts(&self, out: &mut IndentWrite, block: &RegisterBlock) -> Result<(), Error> {
        if block.instances.len() > 1 && block.register_templates.len() > 1 {
            for instance in block.instances.values() {
                if let Some(adr) = &instance.adr {
                    let block_name = &block.name;
                    let instance_name = &instance.name;
                    let instance_name_const = rs_const(&instance.name);
                    let address_type = &self.address_type;
                    writeln!(out)?;
                    writeln!(out, "/// Start of `{block_name}` instance `{instance_name}`")?;
                    writeln!(out, "pub const {instance_name_const}_INSTANCE: {address_type} = 0x{adr:x};")?;
                }
            }
        }
        Ok(())
    }

    fn generate_register(
        &self,
        out: &mut IndentWrite,
        block: &RegisterBlock,
        template: &Register,
    ) -> Result<(), Error> {
        let reg_name_generic = template.name_in_block(block);

        // Header:
        if block.from_explicit_listing_block {
            writeln!(out)?;
            writeln!(out, "// ==== {reg_name_generic}: ====")?;
        }

        // Register consts (address, reset val, offset from block start, always_write)
        self.generate_register_consts(out, block, template)?;

        // Register enums:
        for field in template.fields.values() {
            if let FieldType::LocalEnum(local_enum) = &field.accepts {
                self.generate_enum(out, local_enum)?;
            }
        }

        // Register struct + conversion impls:
        if !template.fields.is_empty() {
            self.generate_register_struct(out, block, template)?;
            if self.opts.generate_defaults {
                self.generate_register_default(out, block, template)?;
            }
            self.generate_register_to_bytes(out, block, template)?;
            self.generate_register_from_bytes(out, block, template)?;
            if self.opts.generate_uint_conversion {
                self.generate_uint_conversion_funcs(out, block, template)?;
            }
        }

        Ok(())
    }

    fn generate_register_consts(
        &self,
        out: &mut IndentWrite,
        block: &RegisterBlock,
        template: &Register,
    ) -> Result<(), Error> {
        let template_name = &template.name;
        let block_name = &block.name;
        let reg_name_generic = template.name_in_block(block);
        let reg_name_generic_const = rs_const(&reg_name_generic);
        let address_type = &self.address_type;

        let byte_array = format!("[u8; {}]", template.width_bytes());

        // Offset of register from register block start (if 'interesting' because there are multiple templates).
        if block.instances.len() > 1 && block.register_templates.len() > 1 {
            if let Some(template_offset) = template.adr {
                writeln!(out)?;
                writeln!(out, "/// Offset of `{template_name}` register from `{block_name}` block start")?;
                writeln!(out, "pub const {reg_name_generic_const}_OFFSET: {address_type} = 0x{template_offset:x};")?;
            }
        }

        // Register address for each instance.
        if let Some(template_offset) = template.adr {
            for instance in block.instances.values() {
                let instance_name = template.name_in_instance(instance);
                if let Some(instance_adr) = &instance.adr {
                    let adr = instance_adr + template_offset;
                    writeln!(out)?;
                    writeln!(out, "/// `{instance_name}` register address")?;
                    writeln!(out, "pub const {reg_name_generic_const}_ADDRESS: {address_type} = 0x{adr:x};")?;
                }
            }
        }

        // Reset val.
        if let Some(reset_val) = &template.reset_val {
            let val = to_array_literal(Endianess::Little, *reset_val, template.width_bytes());
            writeln!(out)?;
            writeln!(out, "/// `{reg_name_generic}` little-endian reset value")?;
            writeln!(out, "pub const {reg_name_generic_const}_RESET_LE: {byte_array} = {val};")?;

            let val = to_array_literal(Endianess::Big, *reset_val, template.width_bytes());
            writeln!(out)?;
            writeln!(out, "/// `{reg_name_generic}` big-endian reset value")?;
            writeln!(out, "pub const {reg_name_generic_const}_RESET_BE: {byte_array} = {val};")?;
        }

        // Always write information.
        if let Some(always_write) = &template.always_write {
            let value = &always_write.value;

            let val = to_array_literal(Endianess::Little, *value, template.width_bytes());
            writeln!(out)?;
            writeln!(out, "/// `{reg_name_generic}` little-endian always write value")?;
            writeln!(out, "pub const {reg_name_generic_const}_ALWAYSWRITE_VALUE_LE: {byte_array} = {val};")?;

            let val = to_array_literal(Endianess::Big, *value, template.width_bytes());
            writeln!(out)?;
            writeln!(out, "/// `{reg_name_generic}` big-endian always write value")?;
            writeln!(out, "pub const {reg_name_generic_const}_ALWAYSWRITE_VALUE_BE: {byte_array} = {val};")?;
        }

        Ok(())
    }

    fn generate_register_struct(
        &self,
        out: &mut IndentWrite,
        block: &RegisterBlock,
        template: &Register,
    ) -> Result<(), Error> {
        let reg_name_generic = template.name_in_block(block);

        // Struct doc comment:
        writeln!(out)?;
        writeln!(out, "/// `{reg_name_generic}` register")?;
        if !block.docs.is_empty() {
            writeln!(out, "///")?;
            write!(out, "{}", block.docs.as_multiline("/// "))?;
        }

        // Register derives:
        if !self.opts.struct_derive.is_empty() {
            let derives = self.opts.struct_derive.join(", ");
            writeln!(out, "#[derive({derives})]")?;
        }

        // Struct proper:
        writeln!(out, "pub struct {} {{", rs_pascalcase(&reg_name_generic))?;

        for field in template.fields.values() {
            let field_type = self.register_struct_member_type(field)?;
            let field_name = rs_snakecase(&field.name);
            generate_doc_comment(out, &field.docs, "    ")?;
            writeln!(out, "    pub {field_name}: {field_type},")?;
        }

        writeln!(out, "}}")?;

        Ok(())
    }

    /// Type of a field inside a register struct.
    fn register_struct_member_type(&self, field: &Field) -> Result<String, Error> {
        match &field.accepts {
            FieldType::LocalEnum(local_enum) => Ok(rs_pascalcase(&local_enum.name)),
            FieldType::SharedEnum(shared_enum) => {
                // If registers blocks are wrapped in modules, all shared enums
                // are declared in the parent module:
                if self.opts.register_block_mods {
                    Ok(format!("super::{}", rs_pascalcase(&shared_enum.name)))
                } else {
                    Ok(rs_pascalcase(&shared_enum.name))
                }
            }
            FieldType::UInt => rs_fitting_unsigned_type(mask_width(field.mask)),
            FieldType::Bool => Ok("bool".to_string()),
        }
    }

    fn generate_register_default(
        &self,
        out: &mut IndentWrite,
        block: &RegisterBlock,
        template: &Register,
    ) -> Result<(), Error> {
        let Some(reset_val) = template.reset_val else {
            return Ok(());
        };

        let reg_name_generic = template.name_in_block(block);
        let struct_name = rs_pascalcase(&reg_name_generic);

        writeln!(out)?;
        writeln!(out, "/// Register state at reset.")?;
        writeln!(out, "#[allow(clippy::derivable_impls)]")?;
        writeln!(out, "impl Default for {struct_name} {{")?;
        writeln!(out, "    fn default() -> Self {{")?;
        writeln!(out, "        Self {{")?;
        out.increase_indent(3);

        for field in template.fields.values() {
            let field_name = rs_snakecase(&field.name);

            // Attempt to decode field:
            let decoded_value = field.decode_unpositioned_value(reset_val).map_err(|x|
                match x {
                    Error::GeneratorError(x) => Error::GeneratorError(format!("Register {reg_name_generic}: Generating struct default impls from reset values requires that the register can represent that value: {x}")),
                    x => x,
                }

            )?;

            let decoded_value = match decoded_value {
                crate::regmap::DecodedField::UInt(v) => format!("0x{v:X}"),
                crate::regmap::DecodedField::Bool(b) => if b { "true" } else { "false" }.to_string(),
                crate::regmap::DecodedField::EnumEntry(e) => {
                    let enum_name = self.register_struct_member_type(field)?;
                    let entry_name = rs_pascalcase(&e);
                    format!("{enum_name}::{entry_name}")
                }
            };

            writeln!(out, "{field_name}: {decoded_value},")?;
        }

        out.decrease_indent(3);
        writeln!(out, "        }}")?;
        writeln!(out, "    }}")?;
        writeln!(out, "}}")?;

        Ok(())
    }

    const CONVERSION_TRAITS: &'static str = include_str!("traits.txt");

    fn generate_traits(&self, out: &mut IndentWrite) -> Result<(), Error> {
        writeln!(out)?;
        writeln!(out, "// ==== Traits: ====")?;
        writeln!(out)?;
        write!(out, "{}", Self::CONVERSION_TRAITS)?;
        Ok(())
    }

    fn generate_register_to_bytes(
        &self,
        out: &mut IndentWrite,
        block: &RegisterBlock,
        template: &Register,
    ) -> Result<(), Error> {
        let reg_name_generic = template.name_in_block(block);
        let reg_name_generic_const = rs_const(&reg_name_generic);
        let struct_name = rs_pascalcase(&reg_name_generic);
        let width_bytes = template.width_bytes();

        let trait_prefix = self.trait_prefix();

        // Impl block and function signature:
        writeln!(out)?;
        writeln!(out, "impl {trait_prefix}ToBytes<{width_bytes}> for {struct_name} {{")?;
        writeln!(out, "    #[allow(clippy::cast_possible_truncation)]")?;
        writeln!(out, "    fn to_le_bytes(&self) -> [u8; {width_bytes}] {{")?;
        out.increase_indent(2);

        // Variable to hold result:
        writeln!(out, "let mut val: [u8; {width_bytes}] = [0; {width_bytes}];")?;

        // Apply always-write value (if any):
        if template.always_write.is_some() {
            let val_const = format!("{reg_name_generic_const}_ALWAYSWRITE_VALUE_LE");
            writeln!(out, "for i in 0..{width_bytes} {{")?;
            writeln!(out, "    val[i] |= {val_const}[i];")?;
            writeln!(out, "}}")?;
        }

        // Insert each field:
        for field in template.fields.values() {
            let field_name = rs_snakecase(&field.name);
            for byte in 0..width_bytes {
                let Some(transform) = field_to_byte_transform(Endianess::Little, field.mask, byte, width_bytes) else {
                    // This field is not present in this byte.
                    continue;
                };

                // Convert the field to some unsigned integer that can be shifted:
                let field_value = match &field.accepts {
                    FieldType::UInt => format!("self.{field_name}"),
                    FieldType::Bool => format!("u8::from(self.{field_name})"),
                    FieldType::LocalEnum(e) => {
                        let enum_uint = rs_fitting_unsigned_type(e.min_bitdwith())?;
                        format!("(self.{field_name} as {enum_uint})")
                    }
                    FieldType::SharedEnum(e) => {
                        let enum_uint = rs_fitting_unsigned_type(e.min_bitdwith())?;
                        format!("(self.{field_name} as {enum_uint})")
                    }
                };

                // The byte of interest:
                let field_byte = match &transform.shift {
                    Some((ShiftDirection::Left, amnt)) => format!("({field_value} << {amnt})"),
                    Some((ShiftDirection::Right, amnt)) => format!("({field_value} >> {amnt})"),
                    None => field_value,
                };

                let masked_field_byte = if transform.mask == 0xFF {
                    field_byte
                } else {
                    format!("({field_byte} & 0x{:X})", transform.mask)
                };

                writeln!(out, "val[{byte}] |= {masked_field_byte} as u8;")?;
            }
        }

        // Return result:
        writeln!(out, "val")?;

        // End of impl block/signature:
        out.decrease_indent(2);
        writeln!(out, "    }}")?;
        writeln!(out, "}}")?;

        Ok(())
    }

    fn generate_register_from_bytes(
        &self,
        out: &mut IndentWrite,
        block: &RegisterBlock,
        template: &Register,
    ) -> Result<(), Error> {
        let reg_name_generic = template.name_in_block(block);
        let struct_name = rs_pascalcase(&reg_name_generic);
        let width_bytes = template.width_bytes();

        // If registers are in modules, trait must be explicitly used:
        let trait_prefix = self.trait_prefix();

        let error_type = if self.opts.unpacking_error_msg {
            "&'static str"
        } else {
            "()"
        };

        // Impl block and function signature:
        // Depending on if the bytes-to-register conversion can fail, we either
        // generate an 'FromBytes' or 'TryFromBytes' impl.
        if template.can_always_unpack() {
            writeln!(out)?;
            writeln!(out, "impl {trait_prefix}FromBytes<{width_bytes}> for {struct_name} {{")?;
            writeln!(out, "    fn from_le_bytes(val: [u8; {width_bytes}]) -> Self {{")?;
            writeln!(out, "        Self {{")?;
        } else {
            writeln!(out)?;
            writeln!(out, "impl {trait_prefix}TryFromBytes<{width_bytes}> for {struct_name} {{")?;
            writeln!(out, "    type Error = {error_type};")?;
            writeln!(out, "    fn try_from_le_bytes(val: [u8; {width_bytes}]) -> Result<Self, Self::Error> {{")?;
            writeln!(out, "        Ok(Self {{")?;
        }
        out.increase_indent(3);

        for field in template.fields.values() {
            let field_name = rs_snakecase(&field.name);

            let field_raw_type = match &field.accepts {
                FieldType::UInt => self.register_struct_member_type(field)?,
                FieldType::Bool => "u8".to_string(),
                FieldType::LocalEnum(e) => rs_fitting_unsigned_type(e.min_bitdwith())?,
                FieldType::SharedEnum(e) => rs_fitting_unsigned_type(e.min_bitdwith())?,
            };

            let mut unpacked_value_parts: Vec<String> = vec![];

            for byte in 0..width_bytes {
                let Some(transform) = byte_to_field_transform(Endianess::Little, field.mask, byte, width_bytes) else {
                    continue;
                };

                let casted_value = if field_raw_type == "u8" {
                    format!("val[{byte}]")
                } else {
                    format!("{field_raw_type}::from(val[{byte}])")
                };

                let masked = if transform.mask == 0xFF {
                    casted_value
                } else {
                    format!("({casted_value} & 0x{:X})", transform.mask)
                };

                match &transform.shift {
                    Some((ShiftDirection::Left, amnt)) => unpacked_value_parts.push(format!("{masked} << {amnt}")),
                    Some((ShiftDirection::Right, amnt)) => unpacked_value_parts.push(format!("{masked} >> {amnt}")),
                    None => unpacked_value_parts.push(masked),
                };
            }
            assert!(!unpacked_value_parts.is_empty());

            let unpacked_value = remove_wrapping_parens(&unpacked_value_parts.join(" | "));

            let converted_value = if field.get_enum().is_some() {
                // Conversion function to use, depending on if enum can always unpack:
                let conversion = if field.can_always_unpack() {
                    "into()"
                } else {
                    "try_into()?"
                };

                // Convert enum to uint.
                format!("({unpacked_value}).{conversion}")
            } else {
                match field.accepts {
                    FieldType::UInt => unpacked_value,
                    FieldType::Bool => format!("({unpacked_value}) != 0"),
                    FieldType::LocalEnum(_) | FieldType::SharedEnum(_) => unreachable!(),
                }
            };

            writeln!(out, "{field_name}: {converted_value},")?;
        }

        out.decrease_indent(3);
        // Close struct, function and impl:
        if template.can_always_unpack() {
            writeln!(out, "        }}")?;
        } else {
            writeln!(out, "        }})")?;
        }
        writeln!(out, "    }}")?;
        writeln!(out, "}}")?;
        Ok(())
    }

    fn generate_uint_conversion_funcs(
        &self,
        out: &mut IndentWrite,
        block: &RegisterBlock,
        template: &Register,
    ) -> Result<(), Error> {
        let reg_name_generic = template.name_in_block(block);
        let struct_name = rs_pascalcase(&reg_name_generic);
        let trait_prefix = self.trait_prefix();
        let (uint_type, uint_width_bytes) = match template.width_bytes() {
            1 => ("u8", 1),
            2 => ("u16", 2),
            3..=4 => ("u32", 4),
            5..=8 => ("u64", 8),
            9..=16 => ("u128", 16),
            _ => return Ok(()),
        };

        // Struct -> Bytes:

        writeln!(out)?;
        writeln!(out, "impl From<&{struct_name}> for {uint_type} {{")?;
        writeln!(out, "    fn from(value: &{struct_name}) -> Self {{")?;
        out.increase_indent(2);

        if !trait_prefix.is_empty() {
            writeln!(out, "use {trait_prefix}ToBytes;")?;
        }
        if uint_width_bytes == template.width_bytes() {
            writeln!(out, "Self::from_le_bytes(value.to_le_bytes())")?;
        } else {
            writeln!(out, "let mut bytes = [0; {uint_width_bytes}];")?;
            writeln!(out, "bytes[0..{}].copy_from_slice(&value.to_le_bytes());", template.width_bytes())?;
            writeln!(out, "Self::from_le_bytes(bytes)")?;
        }

        out.decrease_indent(2);
        writeln!(out, "    }}")?;
        writeln!(out, "}}")?;

        // Bytes -> Struct:

        if template.can_always_unpack() {
            writeln!(out)?;
            writeln!(out, "impl From<{uint_type}> for {struct_name} {{")?;
            writeln!(out, "    fn from(value: {uint_type}) -> Self {{")?;
            if !trait_prefix.is_empty() {
                writeln!(out, "        use {trait_prefix}FromBytes;")?;
            }
            if uint_width_bytes == template.width_bytes() {
                writeln!(out, "        Self::from_le_bytes(value.to_le_bytes())")?;
            } else {
                writeln!(out, "        let mut bytes = [0; {}];", template.width_bytes())?;
                writeln!(out, "        bytes.copy_from_slice(&(value.to_le_bytes()[0..{}]));", template.width_bytes())?;
                writeln!(out, "        Self::from_le_bytes(bytes)")?;
            }
            writeln!(out, "    }}")?;
            writeln!(out, "}}")?;
        } else {
            writeln!(out)?;
            writeln!(out, "impl TryFrom<{uint_type}> for {struct_name} {{")?;
            if self.opts.unpacking_error_msg {
                writeln!(out, "    type Error = &'static str;")?;
            } else {
                writeln!(out, "    type Error = ();")?;
            }
            writeln!(out, "    fn try_from(value: {uint_type}) -> Result<Self, Self::Error> {{")?;
            if !trait_prefix.is_empty() {
                writeln!(out, "        use {trait_prefix}TryFromBytes;")?;
            }
            if uint_width_bytes == template.width_bytes() {
                writeln!(out, "        Self::try_from_le_bytes(value.to_le_bytes())")?;
            } else {
                writeln!(out, "        let mut bytes = [0; {}];", template.width_bytes())?;
                writeln!(out, "        bytes.copy_from_slice(&(value.to_le_bytes()[0..{}]));", template.width_bytes())?;
                writeln!(out, "        Self::try_from_le_bytes(bytes)")?;
            }
            writeln!(out, "    }}")?;
            writeln!(out, "}}")?;
        }

        Ok(())
    }

    fn trait_prefix(&self) -> String {
        // Decide trait prefix. If an external override is given, use that.
        // Otherwise, use the local definition. In this case, if the register-related
        // content is wrapped in a struct, the traits are defined in the parent
        // scope.
        self.opts.external_traits.as_ref().cloned().unwrap_or(
            // Local trait definition
            if self.opts.register_block_mods {
                String::from("super::")
            } else {
                String::new()
            },
        )
    }
}

/// Convert a value to an array literal of given endianess
fn to_array_literal(endian: Endianess, val: TypeValue, width_bytes: TypeBitwidth) -> String {
    let mut bytes: Vec<String> = vec![];

    for i in 0..width_bytes {
        let byte = format!("0x{:X}", ((val >> (8 * i)) & 0xFF) as u8);
        bytes.push(byte);
    }

    if matches!(endian, Endianess::Big) {
        bytes.reverse();
    }

    format!("[{}]", bytes.join(", "))
}
