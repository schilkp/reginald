import argparse
from dataclasses import dataclass
from os import path
from typing import Any, List

from reginald.datamodel import (Docs, Field, RegEnum, Register, RegisterBlock,
                                RegisterMap)
from reginald.generator import OutputGenerator
from reginald.utils import (c_fitting_unsigned_type, c_sanitize,
                            str_pad_to_length)


@dataclass
class GenArg():
    flag: str
    action: str | type[argparse.Action]
    help: str
    default: Any


ARGS = {
    'field_enum_prefix':
    GenArg(flag='--field-enum-prefix',
           action=argparse.BooleanOptionalAction,
           help="prefix a field enum with the register name",
           default=True),
    'generic_funcs':
    GenArg(flag='--generic-funcs',
           action=argparse.BooleanOptionalAction,
           help="generate '_Generic' register functions",
           default=True),
    'registers_as_bitfields':
    GenArg(flag='--registers-as-bitfields',
           action=argparse.BooleanOptionalAction,
           help="generate register structs as bitfields to save space",
           default=True),
    'clang_format_guard':
    GenArg(flag='--clang-format-guard',
           action=argparse.BooleanOptionalAction,
           help="include a clang-format guard covering the complete file",
           default=False),
}


class Generator(OutputGenerator):

    def __init__(self):
        self.out = []
        super().__init__()

    def description(self) -> str:
        return "C header with register structs and conversion functions."

    def generate(self, rmap: RegisterMap, input_file: str, output_file: str, args: List[str]):
        opts = parse_args(args)
        input_file_base = path.basename(input_file)
        output_file_base = path.basename(output_file)

        self.out = []

        self.emit(f"/**")
        self.emit(f" * @file {output_file_base}")
        self.emit(f" * @brief {rmap.map_name} registers")
        self.emit(f" * @note do not edit directly: generated using reginald from {input_file_base}")
        self.emit(f" *")
        self.emit(f" * Parameters:")
        self.emit(f" *   - Generator: c.funcpack")
        for key, val in opts.__dict__.items():
            self.emit(f" *   - {key}={val}")
        self.emit(f" */")
        self.emit(f"#ifndef {c_macro(output_file_base)}_")
        self.emit(f"#define {c_macro(output_file_base)}_")
        self.emit(f"")

        if opts.clang_format_guard:
            self.emit(f"// clang-format off")
            self.emit(f"")

        self.emit(f"#include <stdint.h>")
        self.emit(f"")

        self.generate_shared_enums(rmap)

        for block in rmap.register_blocks.values():
            for template in block.register_templates.values():

                self.emit(str_pad_to_length(f"// ==== {block.name+template.name} register ", "=", 80))
                if not template.docs.empty():
                    self.emit(template.docs.as_multi_line(prefix="// "))
                self.emit(f"")

                self.generate_register_defines(rmap, block, template)
                self.generate_register_enums(rmap, block, template, opts)

                if len(template.fields) == 0:
                    # Don't generate structs + funcs if there are no fields.
                    continue

                self.generate_register_struct(rmap, block, template, opts)
                self.generate_register_funcs(rmap, block, template, opts)

        if opts.generic_funcs:
            self.generate_generic_funcs(rmap, opts)

        if opts.clang_format_guard:
            self.emit(f"// clang-format on")
        self.emit(f"")
        self.emit(f"#endif /* {c_macro(output_file_base)} */")

        with open(output_file, 'w') as outfile:
            outfile.write("\n".join(self.out) + "\n")
        print(f"Generated {output_file}...")

    def emit(self, s: str | List[str]):
        if isinstance(s, str):
            self.out.append(s)
        else:
            self.out.extend(s)

    def generate_shared_enums(self, rmap: RegisterMap):
        self.emit(str_pad_to_length(f"// ==== Shared enums ", "=", 80))
        self.emit(f"")
        for enum in rmap.enums.values():
            self.emit(doxy_comment(enum.docs))
            self.emit(f"enum {name_shared_enum(rmap, enum)} {{")
            for entry in enum.entries.values():
                self.emit(doxy_comment(entry.docs, prefix="  "))
                self.emit(f"  {name_shared_enum(rmap, enum).upper()}_{c_sanitize(entry.name).upper()} = 0x{entry.value:X}U,")
            self.emit(f"}};")
            self.emit(f"")

    def generate_register_defines(self, rmap: RegisterMap, block: RegisterBlock, template: Register):
        macro_reg_template = c_macro(block.name + template.name)
        macro_prefix = c_macro(rmap.map_name) + "_REG"

        for instance_name, instance_start in block.instances.items():
            self.emit(f"#define {macro_prefix}_{c_macro(instance_name+template.name)} "
                      f"(0x{template.adr+instance_start:X}U)"
                      f"//!< {instance_name+template.name} address")

        if len(block.instances) > 1 and len(block.register_templates) > 1:
            self.emit(f"#define {macro_prefix}_{c_macro(block.name+template.name)}__OFFSET "
                      f"(0x{template.adr:X}U) "
                      f"//!< Offset of {block.name+template.name} from {block.name} block start")

        if template.reset_val is not None:
            self.emit(f"#define {macro_prefix}_{macro_reg_template}__RESET "
                      f"(0x{template.reset_val:X}U) "
                      f"//!< {block.name+template.name} reset value")

        if template.always_write is not None:
            self.emit(f"#define {macro_prefix}_{macro_reg_template}__ALWAYSWRITE_MASK "
                      f"(0x{template.always_write.bits.get_bitmask():X}U) "
                      f"//!< {block.name+template.name} always write mask")
            self.emit(f"#define {macro_prefix}_{macro_reg_template}__ALWAYSWRITE_VALUE "
                      f"(0x{template.always_write.value:X}U) "
                      f"//!< {block.name+template.name} always write value")

    def generate_register_enums(self, rmap: RegisterMap, block: RegisterBlock, template: Register, opts):
        for enum in template.get_local_enums():
            self.emit(f"")
            self.emit(doxy_comment(enum.docs))
            self.emit(f"enum {name_register_enum(rmap, block, template, enum, opts)} {{")
            for entry in enum.entries.values():
                self.emit(doxy_comment(entry.docs, prefix="  "))
                self.emit(f"  {c_macro(name_register_enum(rmap, block,template, enum, opts))}_{c_macro(entry.name)} "
                          f"= 0x{entry.value:X}U,")
            self.emit(f"}};")

    def generate_register_struct(self, rmap: RegisterMap, block: RegisterBlock, template: Register, opts):
        struct_name = name_register_struct(rmap, block, template)

        self.emit("")
        self.emit(doxy_comment(template.docs, note="use pack/unpack/overwrite functions for conversion to/form packed register value"))
        self.emit(f"struct {struct_name} {{")
        for field in template.fields.values():
            field_type = register_struct_member_type(rmap, block, template, field, opts)
            self.emit(doxy_comment(field.docs, prefix="  "))
            if opts.registers_as_bitfields:
                self.emit(f"  {field_type} {c_code(field.name)} : {field.bits.total_width()};")
            else:
                self.emit(f"  {field_type} {c_code(field.name)};")
        self.emit(f"}};")

    def generate_register_funcs(self, rmap: RegisterMap, block: RegisterBlock, template: Register, opts):
        struct_name = name_register_struct(rmap, block, template)
        packed_type = c_fitting_unsigned_type(template.bitwidth)
        macro_reg_template = c_macro(block.name + template.name)
        macro_prefix = c_macro(rmap.map_name) + "_REG"

        self.emit(f"")
        self.emit(doxy_comment(Docs(
            brief="Convert register struct to packed register value.",
            doc="All bits that are not part of a field or specified as 'always write' are kept as in 'val'.")))
        self.emit(f"static inline {packed_type} {struct_name}_overwrite(const struct {struct_name} *r, {packed_type} val) {{")
        if template.always_write is not None:
            self.emit(f"  val &= ~{macro_prefix}_{macro_reg_template}__ALWAYSWRITE_MASK;")
            self.emit(f"  val |= {macro_prefix}_{macro_reg_template}__ALWAYSWRITE_VALUE;")
        for field in template.fields.values():
            mask = field.bits.get_bitmask()
            unpos_mask = field.bits.get_unpositioned_bits().get_bitmask()
            shift = field.bits.lsb_position()
            self.emit(
                f"  val = (val & ~0x{mask:X}U) | ({packed_type}) ((r->{c_code(field.name)} & 0x{unpos_mask:X}U) << {shift}U);")
        self.emit(f" return val;")
        self.emit(f"}}")

        self.emit(f"")
        self.emit(doxy_comment(Docs(brief="Convert register struct to packed register value.", doc=None)))
        self.emit(f"static inline {packed_type} {struct_name}_pack(const struct {struct_name} *r) {{")
        self.emit(f"  return {struct_name}_overwrite(r, 0);")
        self.emit(f"}}")

        self.emit(f"")
        self.emit(doxy_comment(Docs(brief="Convert packed register value to register struct.", doc=None)))
        self.emit(f"static inline struct {struct_name} {struct_name}_unpack({packed_type} val) {{")
        self.emit(f"  struct {struct_name} r = {{")
        for field in template.fields.values():
            mask = field.bits.get_bitmask()
            field_type = register_struct_member_type(rmap, block, template, field, opts)
            shift = field.bits.lsb_position()
            self.emit(f"    .{c_code(field.name)} = ({field_type}) ((val & 0x{mask:X}U) >> {shift}U),")
        self.emit(f"  }};")
        self.emit(f"  return r;")
        self.emit(f"}}")

        self.emit(f"")
        self.emit(doxy_comment(Docs(brief="Convert packed register value to into a register struct.", doc=None)))
        self.emit(f"static inline void {struct_name}_unpack_into({packed_type} val, struct {struct_name} *s) {{")
        self.emit(f"  *s = {struct_name}_unpack(val);")
        self.emit(f"}}")
        self.emit(f"")

    def generate_generic_funcs(self, rmap: RegisterMap, opts):
        macro_prefix = c_macro(rmap.map_name) + "_REG"

        self.emit(f"")
        if not opts.clang_format_guard:
            self.emit(f"// Disable clang-format for this section, since _Generic formatting is buggy up to v14.")
            self.emit(f"// clang-format off")

        self.emit(f"")
        self.emit(doxy_comment(Docs(
            brief="Convert register struct to packed register value.",
            doc="All bits that are not part of a field or specified as 'always write' are kept as in 'val'.")))
        self.emit(f"#define {macro_prefix+'_OVERWRITE'}(_struct_ptr_, _val_) _Generic((_struct_ptr_), \\")
        for block in rmap.register_blocks.values():
            for template in block.register_templates.values():
                struct_name = name_register_struct(rmap, block, template)
                if len(template.fields) == 0:
                    continue  # Register does not have packing funcs if there are no fields.
                self.emit(f"    struct {struct_name}* : {struct_name}_overwrite,  \\")
        self.out[-1] = self.out[-1].replace(",", "")
        self.emit(f"  )(_struct_ptr_, _val_)")

        self.emit(f"")
        self.emit(doxy_comment(Docs(brief="Convert register struct to packed register value.", doc=None)))
        self.emit(f"#define {macro_prefix+'_PACK'}(_struct_ptr_) _Generic((_struct_ptr_), \\")
        for block in rmap.register_blocks.values():
            for template in block.register_templates.values():
                struct_name = name_register_struct(rmap, block, template)
                if len(template.fields) == 0:
                    continue  # Register does not have packing funcs if there are no fields.
                self.emit(f"    struct {struct_name}* : {struct_name}_pack,  \\")
        self.out[-1] = self.out[-1].replace(",", "")
        self.emit(f"  )(_struct_ptr_)")

        self.emit(f"")
        self.emit(doxy_comment(Docs(brief="Convert packed register value to into a register struct.", doc=None)))
        self.emit(f"#define {macro_prefix+'_UNPACK_INTO'}(_val_, _struct_ptr_) _Generic((_struct_ptr_), \\")
        for block in rmap.register_blocks.values():
            for template in block.register_templates.values():
                struct_name = name_register_struct(rmap, block, template)
                if len(template.fields) == 0:
                    continue  # Register does not have packing funcs if there are no fields.
                self.emit(f"    struct {struct_name}* : {struct_name}_unpack_into,  \\")
        self.out[-1] = self.out[-1].replace(",", "")
        self.emit(f"  )(_val_,_struct_ptr_)")

        self.emit(f"")
        if not opts.clang_format_guard:
            self.emit(f"// clang-format on")


def parse_args(args: List[str]):

    parser = argparse.ArgumentParser(
        prog="c.funcpack",
        description="C Output generator, using functions for register management.")

    for arg in ARGS.values():
        parser.add_argument(arg.flag, action=arg.action, help=arg.help, default=arg.default)

    return parser.parse_args(args)


def doxy_comment(docs: Docs, prefix: str = "", note: str | None = None) -> List[str]:
    brief = docs.brief
    doc = docs.doc

    have_brief = brief is not None
    have_doc = doc is not None
    have_note = note is not None

    match (have_brief, have_note, have_doc):
        case (False, False, False):
            return []
        case (True, False, False):
            return [f"{prefix}/** @brief {brief} */"]
        case (False, True, False):
            return [f"{prefix}/** @note {note} */"]
        case _:
            out = []
            out.append(f"{prefix}/**")
            if brief is not None:
                out.append(f"{prefix} * @brief {brief}")
            if note is not None:
                out.append(f"{prefix} * @note {note}")
            if doc is not None:
                for line in doc.splitlines():
                    out.append(f"{prefix} * {line}")
            out.append(f"{prefix} */")
            return out


def c_macro(s: str) -> str:
    return c_sanitize(s).upper()


def c_code(s: str) -> str:
    return c_sanitize(s).lower()


def name_shared_enum(rmap: RegisterMap, enum: RegEnum) -> str:
    mapname_c = c_code(rmap.map_name)
    enumname_c = c_code(enum.name)
    return f"{mapname_c}_{enumname_c}"


def name_register_enum(rmap: RegisterMap, block: RegisterBlock, template: Register, enum: RegEnum, opts) -> str:
    mapname_c = c_code(rmap.map_name)
    regname_c = c_code(block.name + template.name)
    enumname_c = c_code(enum.name)
    if opts.field_enum_prefix:
        return f"{mapname_c}_{regname_c}_{enumname_c}"
    else:
        return f"{mapname_c}_{enumname_c}"


def name_register_struct(rmap: RegisterMap, block: RegisterBlock, template: Register) -> str:
    mapname_c = c_code(rmap.map_name)
    regname_c = c_code(block.name + template.name)
    return f"{mapname_c}_reg_{regname_c}"


def register_struct_member_type(rmap: RegisterMap, block: RegisterBlock, template: Register, field: Field, opts) -> str:
    if field.enum is None:
        return c_fitting_unsigned_type(field.bits.total_width())
    else:
        if field.enum.is_shared:
            return "enum " + name_shared_enum(rmap, field.enum)
        else:
            return "enum " + name_register_enum(rmap, block, template, field.enum, opts)
