use std::fmt::Write;

use clap::Parser;

use crate::{
    error::Error,
    indent_write::IndentWrite,
    regmap::{
        bits::{bitmask_from_width, lsb_pos, mask_width, msb_pos, unpositioned_mask},
        Enum, Field, FieldType, Register, RegisterBlock, RegisterMap,
    },
    utils::filename,
};

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
}

// ====== Generator ============================================================

pub fn generate(out: &mut dyn Write, map: &RegisterMap, opts: &GeneratorOpts) -> Result<(), Error> {
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

    let generator = Generator {
        opts: opts.clone(),
        address_type,
        map,
    };

    let mut out = IndentWrite::new(out, "    ");
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
    fn generate(&self, out: &mut IndentWrite) -> Result<(), Error> {
        self.generate_header(out)?;

        if !self.map.shared_enums.is_empty() {
            writeln!(out)?;
            writeln!(out, "// ==== Shared Enums: ====")?;
            self.generate_shared_enums(out)?;
        }

        writeln!(out)?;
        writeln!(out, "// ==== Registers: ====")?;

        for block in self.map.register_blocks.values() {
            self.generate_register_block_module(out, block)?;
        }

        Ok(())
    }

    fn generate_header(&self, out: &mut IndentWrite) -> Result<(), Error> {
        writeln!(out, "/// {} Registers", self.map.map_name)?;
        writeln!(out, "///")?;
        if let Some(input_file) = &self.map.from_file {
            writeln!(out, "/// Generated using reginald from {}.", filename(input_file)?)?;
        } else {
            writeln!(out, "/// Generated using reginald.",)?;
        }
        writeln!(out, "/// Generator: rs-nodeps")?;
        if !self.map.docs.is_empty() {
            writeln!(out, "///")?;
            write!(out, "{}", self.map.docs.as_multiline("/// "))?;
        }

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

    fn generate_shared_enums(&self, out: &mut IndentWrite) -> Result<(), Error> {
        for shared_enum in self.map.shared_enums.values() {
            self.generate_enum(out, shared_enum)?;
        }

        Ok(())
    }

    fn generate_enum(&self, out: &mut IndentWrite, e: &Enum) -> Result<(), Error> {
        let uint_type = rs_fitting_unsigned_type(e.min_bitdwith())?;

        writeln!(out)?;
        generate_doc_comment(out, &e.docs, "")?;
        writeln!(out, "#[derive(PartialEq, Debug)]")?;
        writeln!(out, "#[repr({uint_type})]")?;
        writeln!(out, "pub enum {} {{", rs_pascalcase(&e.name))?;
        for entry in e.entries.values() {
            generate_doc_comment(out, &entry.docs, "    ")?;
            writeln!(out, "    {} = 0x{:x},", rs_pascalcase(&entry.name), entry.value)?;
        }
        writeln!(out, "}}")?;

        self.generate_enum_impl(out, e)?;

        Ok(())
    }

    fn generate_enum_impl(&self, out: &mut IndentWrite, e: &Enum) -> Result<(), Error> {
        let uint_type = rs_fitting_unsigned_type(e.min_bitdwith())?;

        if e.can_unpack_min_bitwidth() {
            // If the enum can represent every value from its minimal bitwidth,
            // implement a wrapping conversion.
            writeln!(out)?;
            writeln!(out, "impl From<{uint_type}> for {} {{", rs_pascalcase(&e.name))?;
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
            writeln!(out)?;
            writeln!(out, "impl TryFrom<{uint_type}> for {} {{", rs_pascalcase(&e.name))?;
            if self.opts.unpacking_error_msg {
                writeln!(out, "    type Error = &'static str;")?;
            } else {
                writeln!(out, "    type Error = ();")?;
            }
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

    fn generate_register_block_module(&self, out: &mut IndentWrite, block: &RegisterBlock) -> Result<(), Error> {
        writeln!(out)?;
        if block.from_explicit_listing_block {
            writeln!(out, "/// {} register block", block.name)?;
        } else {
            writeln!(out, "/// {} register", block.name)?;
        }
        if !block.docs.is_empty() {
            writeln!(out, "///")?;
            write!(out, "{}", block.docs.as_multiline("/// "))?;
        }
        writeln!(out, "pub mod {} {{", rs_snakecase(&block.name))?;

        out.push_indent();

        if block.instances.len() > 1 && block.register_templates.len() > 1 {
            self.generate_register_block_consts(out, block)?;
        }

        for template in block.register_templates.values() {
            self.generate_register(out, block, template)?;
        }

        out.pop_indent();

        writeln!(out, "}}")?;
        Ok(())
    }

    fn generate_register_block_consts(&self, out: &mut IndentWrite, block: &RegisterBlock) -> Result<(), Error> {
        let mut lines: Vec<(String, String)> = vec![]; // Doc comment, const definition.

        if block.instances.len() > 1 && block.register_templates.len() > 1 {
            for instance in block.instances.values() {
                if let Some(adr) = &instance.adr {
                    let comment = format!("/// Start of {} instance {}", block.name, instance.name);
                    let constdef = format!(
                        "pub const {}_INSTANCE: {} = 0x{:x};",
                        rs_const(&instance.name),
                        self.address_type,
                        adr
                    );
                    lines.push((comment, constdef));
                }
            }
        }

        for (comment, constdef) in lines {
            writeln!(out)?;
            writeln!(out, "{}", comment)?;
            writeln!(out, "{}", constdef)?;
        }

        Ok(())
    }

    fn generate_register(
        &self,
        out: &mut IndentWrite,
        block: &RegisterBlock,
        template: &Register,
    ) -> Result<(), Error> {
        let template_name = template.name_in_block(block);

        if block.from_explicit_listing_block {
            writeln!(out)?;
            writeln!(out, "// ==== {template_name}: ====")?;
        }

        self.generate_register_consts(out, block, template)?;

        for field in template.fields.values() {
            if let FieldType::LocalEnum(local_enum) = &field.accepts {
                self.generate_enum(out, local_enum)?;
            }
        }

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
        let template_name = template.name_in_block(block);
        let mut lines: Vec<(String, String)> = vec![]; // Doc comment, const definition.
        if block.instances.len() > 1 && block.register_templates.len() > 1 {
            if let Some(template_offset) = template.adr {
                let comment = format!("/// Offset of {} register from {} block start", template.name, block.name);
                let constdef = format!(
                    "pub const {}_OFFSET: {} = 0x{:x};",
                    rs_const(&template_name),
                    self.address_type,
                    template_offset
                );
                lines.push((comment, constdef));
            }
        }

        if let Some(template_offset) = template.adr {
            for instance in block.instances.values() {
                let instance_name = template.name_in_instance(instance);
                if let Some(instance_adr) = &instance.adr {
                    let comment = format!("/// {instance_name} register address.");
                    let constdef = format!(
                        "pub const {}_ADDRESS: {} = 0x{:x};",
                        rs_const(&instance_name),
                        self.address_type,
                        template_offset + instance_adr
                    );
                    lines.push((comment, constdef));
                }
            }
        }

        let register_type = rs_fitting_unsigned_type(template.bitwidth)?;

        if let Some(reset_val) = &template.reset_val {
            let comment = format!("/// {template_name} reset value");
            let constdef =
                format!("pub const {}_RESET: {} = 0x{:x};", rs_const(&template_name), register_type, reset_val);
            lines.push((comment, constdef));
        }

        if let Some(always_write) = &template.always_write {
            let comment = format!("/// {template_name} always write mask");
            let constdef = format!(
                "pub const {}_ALWAYSWRITE_MASK: {} = 0x{:x};",
                rs_const(&template_name),
                register_type,
                always_write.mask
            );
            lines.push((comment, constdef));
            let comment = format!("/// {template_name} always write value");
            let constdef = format!(
                "pub const {}_ALWAYSWRITE_VALUE: {} = 0x{:x};",
                rs_const(&template_name),
                register_type,
                always_write.value
            );
            lines.push((comment, constdef));
        }
        for (comment, constdef) in lines {
            writeln!(out)?;
            writeln!(out, "{}", comment)?;
            writeln!(out, "{}", constdef)?;
        }

        Ok(())
    }

    fn generate_register_struct(
        &self,
        out: &mut IndentWrite,
        block: &RegisterBlock,
        template: &Register,
    ) -> Result<(), Error> {
        let template_name = template.name_in_block(block);

        writeln!(out)?;

        writeln!(out, "/// {} register", template_name)?;
        if !block.docs.is_empty() {
            writeln!(out, "///")?;
            write!(out, "{}", block.docs.as_multiline("/// "))?;
        }
        writeln!(out, "#[derive(PartialEq, Debug)]")?;
        writeln!(out, "pub struct {} {{", rs_pascalcase(&template_name))?;

        for (idx, field) in template.fields.values().enumerate() {
            if idx != 0 {
                writeln!(out)?;
            }
            let field_type = self.register_struct_member_type(field)?;
            let field_name = rs_snakecase(&field.name);
            generate_doc_comment(out, &field.docs, "    ")?;
            writeln!(out, "    pub {field_name}: {field_type},")?;
        }

        writeln!(out, "}}")?;

        Ok(())
    }

    fn register_struct_member_type(&self, field: &Field) -> Result<String, Error> {
        match &field.accepts {
            FieldType::LocalEnum(local_enum) => Ok(rs_pascalcase(&local_enum.name)),
            FieldType::SharedEnum(shared_enum) => Ok(format!("super::{}", rs_pascalcase(&shared_enum.name))),
            FieldType::UInt => rs_fitting_unsigned_type(mask_width(field.mask)),
            FieldType::Bool => Ok("bool".to_string()),
        }
    }

    fn generate_register_impl(
        &self,
        out: &mut IndentWrite,
        block: &RegisterBlock,
        template: &Register,
    ) -> Result<(), Error> {
        let uint_type = rs_fitting_unsigned_type(template.bitwidth)?;
        let template_name = template.name_in_block(block);
        let struct_name = rs_pascalcase(&template_name);

        if template.can_always_unpack() {
            writeln!(out)?;
            writeln!(out, "impl From<{uint_type}> for {struct_name} {{",)?;
            writeln!(out, "    fn from(value: {uint_type}) -> Self {{")?;
            writeln!(out, "        Self {{")?;
        } else {
            writeln!(out)?;
            writeln!(out, "impl TryFrom<{uint_type}> for {struct_name} {{",)?;
            if self.opts.unpacking_error_msg {
                writeln!(out, "    type Error = &'static str;")?;
            } else {
                writeln!(out, "    type Error = ();")?;
            }
            writeln!(out, "    fn try_from(value: {uint_type}) -> Result<Self, Self::Error> {{")?;
            writeln!(out, "        Ok(Self {{")?;
        }

        for field in template.fields.values() {
            let field_name = rs_snakecase(&field.name);
            let field_value = format!("(value & 0x{:X}) >> {}", field.mask, lsb_pos(field.mask));
            write!(out, "            ")?;
            write!(out, "{field_name}: ")?;
            if let Some(e) = field.get_enum() {
                let enum_type = rs_fitting_unsigned_type(e.min_bitdwith())?;
                let conversion = if field.can_always_unpack() {
                    "into()"
                } else {
                    "try_into()?"
                };
                if enum_type == uint_type {
                    writeln!(out, "({field_value}).{conversion},")?;
                } else {
                    writeln!(out, "(({field_value}) as {enum_type}).{conversion},")?;
                }
            } else {
                if matches!(field.accepts, FieldType::Bool) {
                    writeln!(out, "{field_value} != 0,")?;
                } else {
                    let field_type = self.register_struct_member_type(field)?;
                    if field_type == uint_type {
                        writeln!(out, "{field_value},")?;
                    } else {
                        writeln!(out, "({field_value}) as {field_type},")?;
                    }
                }
            }
        }

        if template.can_always_unpack() {
            writeln!(out, "        }}")?;
        } else {
            writeln!(out, "        }})")?;
        }
        writeln!(out, "    }}")?;
        writeln!(out, "}}")?;

        writeln!(out)?;
        writeln!(out, "impl From<{}> for {uint_type} {{", rs_pascalcase(&template_name))?;
        writeln!(out, "    fn from(value: {}) -> Self {{", rs_pascalcase(&template_name))?;
        for field in template.fields.values() {
            let field_name = rs_snakecase(&field.name);
            let field_type = self.register_struct_member_type(field)?;

            write!(out, "        ")?;
            write!(out, "let {field_name}: Self = ")?;
            let value = if let Some(e) = field.get_enum() {
                let enum_type = rs_fitting_unsigned_type(e.min_bitdwith())?;
                if enum_type == uint_type {
                    format!("(value.{field_name} as {enum_type})")
                } else {
                    format!("((value.{field_name} as {enum_type}) as {uint_type})")
                }
            } else {
                if field_type == uint_type {
                    format!("value.{field_name}")
                } else {
                    format!("(value.{field_name} as {uint_type})")
                }
            };
            writeln!(out, "({value} & 0x{:X}) << {};", unpositioned_mask(field.mask), lsb_pos(field.mask),)?;
        }
        write!(out, "        ")?;
        for (idx, field) in template.fields.values().enumerate() {
            if idx != 0 {
                write!(out, " | ")?;
            }
            write!(out, "{}", rs_snakecase(&field.name))?;
        }
        if template.always_write.is_some() {
            write!(out, " | {}_ALWAYSWRITE_VALUE", rs_const(&template_name))?;
        }
        writeln!(out)?;

        writeln!(out, "    }}")?;
        writeln!(out, "}}")?;

        Ok(())
    }
}
