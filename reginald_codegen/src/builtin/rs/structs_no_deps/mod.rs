use std::fmt::Write;

use crate::{
    bits::{bitmask_from_width, lsb_pos, mask_width, msb_pos, unpositioned_mask},
    error::Error,
    indent_write::IndentWrite,
    regmap::{Enum, Field, FieldType, Register, RegisterBlock, RegisterMap},
    utils::filename,
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
    /// May be given multiple times.
    #[cfg_attr(feature = "cli", arg(long))]
    #[cfg_attr(feature = "cli", arg(action = clap::ArgAction::Append))]
    #[cfg_attr(feature = "cli", arg(verbatim_doc_comment))]
    pub enum_derive: Vec<String>,
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

    // Generate
    let generator = Generator {
        opts: opts.clone(),
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
}

impl Generator<'_> {
    /// Generate complete file
    fn generate(&self, out: &mut IndentWrite) -> Result<(), Error> {
        // File header/preamble:
        self.generate_header(out)?;

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
        // Top doc comment:
        writeln!(out, "/// {} Registers", self.map.map_name)?;
        writeln!(out, "///")?;

        // Generated-with-reginald note, including original file name if known:
        if let Some(input_file) = &self.map.from_file {
            writeln!(out, "/// Generated using reginald from {}.", filename(input_file)?)?;
        } else {
            writeln!(out, "/// Generated using reginald.")?;
        }

        // Indicate which generator was used:
        writeln!(out, "/// Generator: rs-struct-no-deps")?;

        // Map top-level documentation:
        if !self.map.docs.is_empty() {
            writeln!(out, "///")?;
            write!(out, "{}", self.map.docs.as_multiline("/// "))?;
        }

        // Map author and note:
        if let Some(author) = &self.map.author {
            writeln!(out, "/// ")?;
            writeln!(out, "/// Listing file author: {author}")?;
        }
        if let Some(note) = &self.map.note {
            writeln!(out, "///")?;
            writeln!(out, "/// Listing file note:")?;
            for line in note.lines() {
                writeln!(out, "///   {line}")?;
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
        if !self.opts.enum_derive.is_empty() {
            let derives = self.opts.enum_derive.join(", ");
            writeln!(out, "#[derive({derives})]")?;
        }

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
            writeln!(out, "{header_comment} {} register block", block.name)?;
        } else {
            writeln!(out, "{header_comment} {} register", block.name)?;
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
                    writeln!(out, "/// Start of {block_name} instance {instance_name}")?;
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
            self.generate_register_impl(out, block, template)?;
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
        let register_uint = rs_fitting_unsigned_type(template.bitwidth)?;

        // Offset of register from register block start (if 'interesting' because there are multiple templates).
        if block.instances.len() > 1 && block.register_templates.len() > 1 {
            if let Some(template_offset) = template.adr {
                writeln!(out)?;
                writeln!(out, "/// Offset of {template_name} register from {block_name} block start")?;
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
                    writeln!(out, "/// {instance_name} register address")?;
                    writeln!(out, "pub const {reg_name_generic_const}_ADDRESS: {address_type} = 0x{adr:x};")?;
                }
            }
        }

        // Reset val.
        if let Some(reset_val) = &template.reset_val {
            writeln!(out)?;
            writeln!(out, "/// {reg_name_generic} reset value")?;
            writeln!(out, "pub const {reg_name_generic_const}_RESET: {register_uint} = 0x{reset_val:x};")?;
        }

        // Always write information.
        if let Some(always_write) = &template.always_write {
            let mask = &always_write.mask;
            let value = &always_write.value;
            writeln!(out)?;
            writeln!(out, "/// {reg_name_generic} always write mask")?;
            writeln!(out, "pub const {reg_name_generic_const}_ALWAYSWRITE_MASK: {register_uint} = 0x{mask:x};")?;
            writeln!(out)?;
            writeln!(out, "/// {reg_name_generic} always write value")?;
            writeln!(out, "pub const {reg_name_generic_const}_ALWAYSWRITE_VALUE: {register_uint} = 0x{value:x};")?;
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
        writeln!(out, "/// {reg_name_generic} register")?;
        if !block.docs.is_empty() {
            writeln!(out, "///")?;
            write!(out, "{}", block.docs.as_multiline("/// "))?;
        }

        // Register derives:
        if !self.opts.struct_derive.is_empty() {
            let derives = self.opts.enum_derive.join(", ");
            writeln!(out, "#[derive({derives})]")?;
        }

        // Struct proper:
        writeln!(out, "pub struct {} {{", rs_pascalcase(&reg_name_generic))?;

        for (idx, field) in template.fields.values().enumerate() {
            let field_type = self.register_struct_member_type(field)?;
            let field_name = rs_snakecase(&field.name);

            if idx != 0 {
                writeln!(out)?;
            }
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

    /// Generate register conversion functions
    fn generate_register_impl(
        &self,
        out: &mut IndentWrite,
        block: &RegisterBlock,
        template: &Register,
    ) -> Result<(), Error> {
        let uint_type = rs_fitting_unsigned_type(template.bitwidth)?;
        let reg_name_generic = template.name_in_block(block);
        let reg_name_generic_const = rs_const(&reg_name_generic);
        let struct_name = rs_pascalcase(&reg_name_generic);

        // IMPL 1: Uint -> Register

        writeln!(out)?;

        if template.can_always_unpack() {
            // If the register can always be unpacked, generate a 'From' impl.
            writeln!(out, "impl From<{uint_type}> for {struct_name} {{")?;

            writeln!(out, "    fn from(value: {uint_type}) -> Self {{")?;
            writeln!(out, "        Self {{")?;
        } else {
            // Otherwise, generate a try-from impl with the correct error type.
            writeln!(out, "impl TryFrom<{uint_type}> for {struct_name} {{")?;
            if self.opts.unpacking_error_msg {
                writeln!(out, "    type Error = &'static str;")?;
            } else {
                writeln!(out, "    type Error = ();")?;
            }

            writeln!(out, "    fn try_from(value: {uint_type}) -> Result<Self, Self::Error> {{")?;
            writeln!(out, "        Ok(Self {{")?;
        }

        for field in template.fields.values() {
            // Name of field in register:
            let field_name = rs_snakecase(&field.name);

            // Operation to extract field value from uint:
            let field_value = if lsb_pos(field.mask) == 0 {
                format!("value & 0x{:X}", field.mask)
            } else {
                format!("(value & 0x{:X}) >> {}", field.mask, lsb_pos(field.mask))
            };

            // Appropriate struct member init for each field, depending on type:
            write!(out, "            ")?;
            write!(out, "{field_name}: ")?;

            if let Some(e) = field.get_enum() {
                // Enum.
                // Uint type the enum from/tryfrom impls use:
                let enum_type = rs_fitting_unsigned_type(e.min_bitdwith())?;

                // Conversion function to use, depending on if enum can always unpack:
                let conversion = if field.can_always_unpack() {
                    "into()"
                } else {
                    // Fallible conversion. If any field can only be converted using a 'try_from',
                    // we must be inside a 'try_from' and can use the question mark operator.
                    "try_into()?"
                };

                // Convert enum to uint.
                if enum_type == uint_type {
                    // Enum from/try from use the same type as the register, so we can convert
                    // directly.
                    writeln!(out, "({field_value}).{conversion},")?;
                } else {
                    // Enum from/try from use a different type from the register, so we first
                    // covnert to that type.
                    writeln!(out, "(({field_value}) as {enum_type}).{conversion},")?;
                }
            } else if matches!(field.accepts, FieldType::Bool) {
                // direct bool conversion.
                writeln!(out, "{field_value} != 0,")?;
            } else {
                // Uint field. Convert if field uses a different uint type than the
                // complete register:
                let field_type = self.register_struct_member_type(field)?;
                if field_type == uint_type {
                    writeln!(out, "{field_value},")?;
                } else {
                    writeln!(out, "({field_value}) as {field_type},")?;
                }
            }
        }

        // Complete Struct initialiser. Try-from needs an extra bracket, because struct initialiser is
        // wrapped in an Ok()
        if template.can_always_unpack() {
            writeln!(out, "        }}")?;
        } else {
            writeln!(out, "        }})")?;
        }

        writeln!(out, "    }}")?;
        writeln!(out, "}}")?;

        // IMPL 2: Register -> Uint

        writeln!(out)?;
        writeln!(out, "impl From<{struct_name}> for {uint_type} {{")?;
        writeln!(out, "    fn from(value: {struct_name}) -> Self {{")?;

        // Convert each field to its (positioned) binary value:
        for field in template.fields.values() {
            let field_name = rs_snakecase(&field.name);
            let field_type = self.register_struct_member_type(field)?;

            // Binary value of field, unshifted:
            let value = if let Some(e) = field.get_enum() {
                // Enum.

                // Type used for enum repr:
                let enum_type = rs_fitting_unsigned_type(e.min_bitdwith())?;

                // Convert enum to uint:
                if enum_type == uint_type {
                    // If enum repr matches register uint, convert directly:
                    format!("(value.{field_name} as {enum_type})")
                } else {
                    // If enum repr uses a different type, convert to that first, then to the register
                    // uint. (Enum 'as' convertion only works into enum's repr type)
                    format!("((value.{field_name} as {enum_type}) as {uint_type})")
                }
            } else if field_type == uint_type {
                // Field has same type as output, take as-is:
                format!("value.{field_name}")
            } else {
                // Field is bool or different uart, convert:
                format!("(value.{field_name} as {uint_type})")
            };

            // Shift, store in temporary var:
            let mask = unpositioned_mask(field.mask);
            let shift = lsb_pos(field.mask);
            if shift == 0 {
                writeln!(out, "        let {field_name}: Self = {value} & 0x{mask:X};")?;
            } else {
                writeln!(out, "        let {field_name}: Self = ({value} & 0x{mask:X}) << {shift};")?;
            }
        }

        // Bitwise-Or all fields and always write fields together:
        write!(out, "        ")?;
        for (idx, field) in template.fields.values().enumerate() {
            if idx != 0 {
                write!(out, " | ")?;
            }
            write!(out, "{}", rs_snakecase(&field.name))?;
        }
        if template.always_write.is_some() {
            write!(out, " | {reg_name_generic_const}_ALWAYSWRITE_VALUE")?;
        }
        writeln!(out)?;

        writeln!(out, "    }}")?;
        writeln!(out, "}}")?;

        Ok(())
    }
}
