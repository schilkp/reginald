import argparse
from os import path
from typing import List

from reginald.datamodel import (Docs, Field, RegEnum, Register, RegisterBlock,
                                RegisterMap)
from reginald.generator import OutputGenerator
from reginald.utils import (c_fitting_unsigned_type, c_sanitize,
                            str_pad_to_length)


class Generator(OutputGenerator):
    def description(self) -> str:
        return "C header with register structs and conversion functions."

    def generate(self, rmap: RegisterMap, input_file: str, output_file: str, args: List[str]):
        opts = parse_args(args)
        input_file_base = path.basename(input_file)
        output_file_base = path.basename(output_file)

        out = []

        def emit(s: str | List[str]):
            if isinstance(s, str):
                out.append(s)
            else:
                out.extend(s)

        emit(f"/**")
        emit(f" * @file {output_file_base}")
        emit(f" * @brief {rmap.map_name} registers")
        emit(f" * @note do not edit directly: generated using reginald from {input_file_base}")
        emit(f" *")
        emit(f" * Reginald settings:")
        for key, val in opts.__dict__.items():
            emit(f" *   - {key}={val}")
        emit(f" */")
        emit(f"#ifndef {c_macro(output_file_base)}_")
        emit(f"#define {c_macro(output_file_base)}_")
        emit(f"")

        if opts.clang_format_guard:
            emit(f"// clang-format off")
            emit(f"")

        emit(f"#include <stdint.h>")
        emit(f"")

        emit(str_pad_to_length(f"// ==== Shared enums ", "=", 80))
        emit(f"")
        for enum in rmap.enums.values():
            emit(doxy_comment(enum.docs))
            emit(f"enum {name_shared_enum(rmap, enum)} {{")
            for entry in enum.entries.values():
                emit(doxy_comment(entry.docs, prefix="  "))
                emit(f"  {name_shared_enum(rmap, enum).upper()}_{c_sanitize(entry.name).upper()} = 0x{entry.value:X}U,")
            emit(f"}};")
            emit(f"")

        macro_prefix = c_macro(rmap.map_name) + "_REG"

        for block in rmap.register_blocks.values():
            for template in block.register_templates.values():
                macro_reg_template = c_macro(block.name + template.name)

                emit(str_pad_to_length(f"// ==== {block.name+template.name} register ", "=", 80))
                if not template.docs.empty():
                    emit(template.docs.as_multi_line(prefix="// "))
                emit(f"")

                for instance_name, instance_start in block.instances.items():
                    emit(f"#define {macro_prefix}_{c_macro(instance_name+template.name)} "
                         f"(0x{template.adr+instance_start:X}U)"
                         f"//!< {instance_name+template.name} address")

                if len(block.instances) > 1 and len(block.register_templates) > 1:
                    emit(f"#define {macro_prefix}_{c_macro(block.name+template.name)}__OFFSET "
                         f"(0x{template.adr:X}U) "
                         f"//!< Offset of {block.name+template.name} from {block.name} block start")

                if template.reset_val is not None:
                    emit(f"#define {macro_prefix}_{macro_reg_template}__RESET "
                         f"(0x{template.reset_val:X}U) "
                         f"//!< {block.name+template.name} reset value")

                if template.always_write is not None:
                    emit(f"#define {macro_prefix}_{macro_reg_template}__ALWAYSWRITE_MASK "
                         f"(0x{template.always_write.bits.get_bitmask():X}U) "
                         f"//!< {block.name+template.name} always write mask")
                    emit(f"#define {macro_prefix}_{macro_reg_template}__ALWAYSWRITE_VALUE "
                         f"(0x{template.always_write.value:X}U) "
                         f"//!< {block.name+template.name} always write value")

                for enum in template.get_local_enums():
                    emit(f"")
                    emit(doxy_comment(enum.docs))
                    emit(f"enum {name_register_enum(rmap, block, template, enum, opts)} {{")
                    for entry in enum.entries.values():
                        emit(doxy_comment(entry.docs, prefix="  "))
                        emit(f"  {c_macro(name_register_enum(rmap, block,template, enum, opts))}_{c_macro(entry.name)} "
                             f"= 0x{entry.value:X}U,")
                    emit(f"}};")

                if len(template.fields) == 0:
                    # Don't generate structs + funcs if there are no fields.
                    continue

                struct_name = name_register_struct(rmap, block, template)
                packed_type = c_fitting_unsigned_type(template.bitwidth)

                emit("")
                emit(doxy_comment(template.docs, note="use pack/unpack/overwrite functions for conversion to/form packed register value"))
                emit(f"struct {struct_name} {{")
                for field in template.fields.values():
                    field_type = register_struct_member_type(rmap, block, template, field, opts)
                    emit(doxy_comment(field.docs, prefix="  "))
                    if opts.registers_as_bitfields:
                        emit(f"  {field_type} {c_code(field.name)} : {field.bits.total_width()};")
                    else:
                        emit(f"  {field_type} {c_code(field.name)};")
                emit(f"}};")

                emit(f"")
                emit(doxy_comment(Docs(
                    brief="Convert register struct to packed register value.",
                    doc="All bits that are not part of a field or specified as 'always write' are kept as in 'val'.")))
                emit(f"static inline {packed_type} {struct_name}_overwrite(const struct {struct_name} *r, {packed_type} val) {{")
                if template.always_write is not None:
                    emit(f"  val &= ~{macro_prefix}_{macro_reg_template}__ALWAYSWRITE_MASK;")
                    emit(f"  val |= {macro_prefix}_{macro_reg_template}__ALWAYSWRITE_VALUE;")
                for field in template.fields.values():
                    mask = field.bits.get_bitmask()
                    unpos_mask = field.bits.get_unpositioned_bits().get_bitmask()
                    shift = field.bits.lsb_position()
                    emit(f"  val = (val & ~0x{mask:X}U) | ({packed_type}) ((r->{c_code(field.name)} & 0x{unpos_mask:X}U) << {shift}U);")
                emit(f" return val;")
                emit(f"}}")

                emit(f"")
                emit(doxy_comment(Docs(brief="Convert register struct to packed register value.", doc=None)))
                emit(f"static inline {packed_type} {struct_name}_pack(const struct {struct_name} *r) {{")
                emit(f"  return {struct_name}_overwrite(r, 0);")
                emit(f"}}")

                emit(f"")
                emit(doxy_comment(Docs(brief="Convert packed register value to register struct.", doc=None)))
                emit(f"static inline struct {struct_name} {struct_name}_unpack({packed_type} val) {{")
                emit(f"  return {{")
                for field in template.fields.values():
                    mask = field.bits.get_bitmask()
                    field_type = register_struct_member_type(rmap, block, template, field, opts)
                    shift = field.bits.lsb_position()
                    emit(f"    .{c_code(field.name)} = ({field_type}) ((val & 0x{mask:X}U) >> {shift}U),")
                emit(f"  }};")
                emit(f"}}")

                emit(f"")
                emit(doxy_comment(Docs(brief="Convert packed register value to into a register struct.", doc=None)))
                emit(f"static inline void {struct_name}_unpack_into({packed_type} val, struct {struct_name} *s) {{")
                emit(f"  *s = {struct_name}_unpack(val);")
                emit(f"}}")
                emit(f"")

        if opts.generic_funcs:
            emit(f"")
            if not opts.clang_format_guard:
                emit(f"// Disable clang-format for this section, since _Generic formatting is buggy up to v14.")
                emit(f"// clang-format off")

            emit(f"")
            emit(doxy_comment(Docs(
                brief="Convert register struct to packed register value.",
                doc="All bits that are not part of a field or specified as 'always write' are kept as in 'val'.")))
            emit(f"#define {macro_prefix+'_OVERWRITE'}(_struct_ptr_, _val_) _Generic((_struct_ptr_), \\")
            for block in rmap.register_blocks.values():
                for template in block.register_templates.values():
                    struct_name = name_register_struct(rmap, block, template)
                    if len(template.fields) == 0:
                        continue  # Register does not have packing funcs if there are no fields.
                    emit(f"    struct {struct_name}* : {struct_name}_overwrite,  \\")
            out[-1] = out[-1].replace(",", "")
            emit(f"  )(_struct_ptr_, _val_)")

            emit(f"")
            emit(doxy_comment(Docs(brief="Convert register struct to packed register value.", doc=None)))
            emit(f"#define {macro_prefix+'_PACK'}(_struct_ptr_) _Generic((_struct_ptr_), \\")
            for block in rmap.register_blocks.values():
                for template in block.register_templates.values():
                    struct_name = name_register_struct(rmap, block, template)
                    if len(template.fields) == 0:
                        continue  # Register does not have packing funcs if there are no fields.
                    emit(f"    struct {struct_name}* : {struct_name}_pack,  \\")
            out[-1] = out[-1].replace(",", "")
            emit(f"  )(_struct_ptr_)")

            emit(f"")
            emit(doxy_comment(Docs(brief="Convert packed register value to into a register struct.", doc=None)))
            emit(f"#define {macro_prefix+'_UNPACK_INTO'}(_val_, _struct_ptr_) _Generic((_struct_ptr_), \\")
            for block in rmap.register_blocks.values():
                for template in block.register_templates.values():
                    struct_name = name_register_struct(rmap, block, template)
                    if len(template.fields) == 0:
                        continue  # Register does not have packing funcs if there are no fields.
                    emit(f"    struct {struct_name}* : {struct_name}_unpack_into,  \\")
            out[-1] = out[-1].replace(",", "")
            emit(f"  )(_val_,_struct_ptr_)")

            emit(f"")
            if not opts.clang_format_guard:
                emit(f"// clang-format on")

        if opts.clang_format_guard:
            emit(f"// clang-format on")
        emit(f"")
        emit(f"#endif /* {c_macro(output_file_base)} */")

        with open(output_file, 'w') as outfile:
            outfile.write("\n".join(out) + "\n")
        print(f"Generated {output_file}...")


def parse_args(args: List[str]):

    # Options:
    # Field Enum: Prefix with register name (Default: Yes)

    parser = argparse.ArgumentParser(
        prog="c.funcpack",
        description="C Output generator, using functions for register management.")

    parser.add_argument('--field-enum-prefix', action=argparse.BooleanOptionalAction,
                        help="prefix a field enum with the register name", default=True)
    parser.add_argument('--generic-funcs', action=argparse.BooleanOptionalAction,
                        help="generate '_Generic' register functions", default=True)
    parser.add_argument('--registers-as-bitfields', action=argparse.BooleanOptionalAction,
                        help="generate register structs as bitfields to save space", default=True)
    parser.add_argument('--clang-format-guard', action=argparse.BooleanOptionalAction,
                        help="include a clang-format guard covering the complete file", default=False)

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
