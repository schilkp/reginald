import os
from typing import Union

from reginald.builtin_generators.c.funcpack.name_generator import NameGenerator
from reginald.cli import CLI
from reginald.datamodel import *


def generate(rmap: RegisterMap, name: NameGenerator, cli: CLI, opt):

    out = []

    out.append(f"/*!")
    out.append(f" * \\file {name.filename_reg_utils()}")
    out.append(f" * \\brief {rmap.map_name} Registers Utilities.")
    out.append(f" * \\note Do not edit: Generated using Reginald.")
    out.append(f" */")
    out.append(f"")
    out.append(f"")

    out.append(f"#ifndef {name.include_guard_macro(name.filename_reg_utils())}")
    out.append(f"#define {name.include_guard_macro(name.filename_reg_utils())}")
    out.append(f"")
    out.append(f"#include <stdint.h>")
    out.append(f"#include \"{name.filename_enums()}\"")
    out.append(f"#include \"{name.filename_regs()}\"")
    out.append(f"")

    registers = {}  # type: Dict[str, Union[Register, RegisterTemplate]]
    for reg in rmap.registers.values():
        if not reg.originates_from_template:
            registers[reg.name] = reg
    for block in rmap.register_block_templates.values():
        for template in block.registers.values():
            registers[block.name + template.name] = template
    out.append(f"/**")
    out.append(f" * \\defgroup {name.doxygroup_genericfuncs()} Generic register modify/pack/unpack utilities.")
    out.append(f" * @{{")
    out.append(f" */")

    # Generate generic modify, pack and unpack macro:
    out.append(f"/**")
    out.append(f" * @brief Modify a register's binary representation")
    out.append(f" * All fields are replaced with the struct's values.")
    out.append(f" * All 'always_write' bits (if any) are forced to the correct value.")
    out.append(f" * All other bits are kept the same.")
    out.append(f" * @note This is a generic macro that picks the correct function based on _struct_ptr_'s type.")
    out.append(f" * @param _struct_ptr_ struct holding register fields")
    out.append(f" * @param _val_ current binary register representation")
    out.append(f" * @return packed register representation")
    out.append(f" */")
    out.append(f"#define {name.generic_modify_macro()}(_struct_ptr_, _val_) _Generic((_struct_ptr_), \\")
    out.append(f"/* type : selected function */ \\")
    for reg_name, reg in registers.items():
        struct_name = name.reg_struct_name(reg_name)
        modify_func = name.reg_modify_func(reg_name)
        out.append(f" struct {struct_name}* : {modify_func},  \\")
    out[-1] = out[-1].replace(",", "")
    out.append(f")(_struct_ptr_, _val_)")
    out.append(f"")

    out.append(f"/**")
    out.append(f" * @brief Pack a register's fields into their binary representation")
    out.append(f" * All fields are set to the struct's values.")
    out.append(f" * All 'always_write' bits (if any) are set to the correct value.")
    out.append(f" * All other bits are are set to 0.")
    out.append(f" * @note This is a generic macro that picks the correct function based on _struct_ptr_'s type.")
    out.append(f" * @param _struct_ptr_r struct holding register fields")
    out.append(f" * @return packed register representation")
    out.append(f" */")
    out.append(f"#define {name.generic_pack_macro()}(_struct_ptr_) _Generic((_struct_ptr_), \\")
    out.append(f"/* type : selected function */ \\")
    for reg_name, reg in registers.items():
        struct_name = name.reg_struct_name(reg_name)
        pack_func = name.reg_pack_func(reg_name)
        out.append(f" struct {struct_name}* : {pack_func},  \\")
    out[-1] = out[-1].replace(",", "")
    out.append(f")(_struct_ptr_)")
    out.append(f"")

    out.append(f"/**")
    out.append(f" * @brief Unpack a register's binary representation into seperate fields")
    out.append(f" * @note This is a generic macro that picks the correct function based on _struct_ptr_'s type.")
    out.append(f" * @param _struct_ptr_ buffer to store the unpacked fields")
    out.append(f" * @param _val_ packed register representation")
    out.append(f" */")
    out.append(f"#define {name.generic_unpack_macro()}(_struct_ptr_, _val_) _Generic((_struct_ptr_), \\")
    out.append(f"/* type : selected function */ \\")
    for reg_name, reg in registers.items():
        struct_name = name.reg_struct_name(reg_name)
        unpack_func = name.reg_unpack_func(reg_name)
        out.append(f" struct {struct_name}* : {unpack_func},  \\")
    out[-1] = out[-1].replace(",", "")
    out.append(f")(_struct_ptr_)")
    out.append(f"")

    out.append(f"/** @}} */")
    out.append(f"")

    for reg_name, reg in registers.items():

        if len(reg.fields) == 0:
            # Don't generate structs + funcs if there are no fields.
            continue

        out.append(f"/**")
        out.append(f" * \\defgroup {name.doxygroup_regfuncs(reg_name)} {reg_name} register modify/pack/unpack utilities.")
        out.append(f" * @{{")
        out.append(f" */")

        packed_type = name.reg_packed_type(reg)
        struct_name = name.reg_struct_name(reg_name)
        modify_func = name.reg_modify_func(reg_name)
        pack_func = name.reg_pack_func(reg_name)
        unpack_func = name.reg_unpack_func(reg_name)
        unpack_macro = name.reg_unpack_macro(reg_name)

        if opt.short_packfunc_comment:
            out.append(f"/** @brief Modify the '{reg_name}' register's binary representation */")
        else:
            out.append(f"/**")
            out.append(f" * @brief Modify the '{reg_name}' register's binary representation")
            out.append(f" * All fields are replaced with the struct's values.")
            out.append(f" * All 'always_write' bits (if any) are forced to the correct value.")
            out.append(f" * All other bits are kept the same.")
            out.append(f" * @param r struct holding register fields")
            out.append(f" * @param val current binary register representation")
            out.append(f" * @return packed register representation")
            out.append(f" */")
        out.append(f"static inline  {packed_type} {modify_func}(const struct {struct_name} *r, {packed_type} val){{")
        if reg.always_write is not None:
            out.append(f"  val &= ~{name.reg_alwayswrite_mask_macro(reg_name)};")
            out.append(f"  val |= {name.reg_alwayswrite_val_macro(reg_name)};")
        for field in reg.fields.values():
            member_name = name.reg_struct_member(field)
            mask = field.bits.get_bitmask()
            shift = field.bits.lsb_position()
            out.append(f"  val = (val & ~0x{mask:X}U) | ({packed_type}) ((r->{member_name} & 0x{mask:X}U) << {shift}U);")
        out.append(f" return val;")
        out.append(f"}}")
        out.append(f"")

        if opt.short_packfunc_comment:
            out.append(f"/** @brief Pack the '{reg_name}' register's fields into their binary representation. */")
        else: 
            out.append(f"/**")
            out.append(f" * @brief Pack the '{reg_name}' register's fields into their binary representation")
            out.append(f" * All fields are set to the struct's values.")
            out.append(f" * All 'always_write' bits (if any) are set to the correct value.")
            out.append(f" * All other bits are are set to 0.")
            out.append(f" * @param r struct holding register fields")
            out.append(f" * @return packed register representation")
            out.append(f" */")
        out.append(f"static inline  {packed_type} {pack_func}(const struct {struct_name} *r){{")
        out.append(f"  return {modify_func}(r, 0);")
        out.append(f"}}")
        out.append(f"")

        if opt.short_packfunc_comment:
            out.append(f"/** @brief Unpack the '{reg_name}' register's binary representation into seperate fields. */")
        else: 
            out.append(f"/**")
            out.append(f" * @brief Unpack the '{reg_name}' register's binary representation into seperate fields")
            out.append(f" * @param r buffer to store the unpacked fields")
            out.append(f" * @param val packed register representation")
            out.append(f" */")
        out.append(f"static inline void {unpack_func}(struct {struct_name} *r, {packed_type} val){{")
        for field in reg.fields.values():
            member_name = name.reg_struct_member(field)
            member_type = name.reg_struct_member_type(reg_name, field)
            mask = field.bits.get_bitmask()
            shift = field.bits.lsb_position()
            out.append(f"  r->{member_name} = ({member_type}) ((val >> {shift}U) & 0x{mask:X}U);")
        out.append(f"}}")
        out.append(f"")

        out.append(f"/**")
        out.append(f" * @brief Unpack the '{reg_name}' register's binary representation into a struct initialiser.")
        out.append(f" * @note use static {unpack_func}() to unpack into an exsisting struct.")
        out.append(f" * Example:")
        out.append(f" *   `struct {struct_name} reg = {unpack_macro}(0xAB);`")
        out.append(f" * ")
        out.append(f" * @param _VAL_ packed register representation")
        out.append(f" */")
        out.append(f"#define {unpack_macro}(_VAL_) {{ \\")
        for field in reg.fields.values():
            member_name = name.reg_struct_member(field)
            member_type = name.reg_struct_member_type(reg_name, field)
            mask = field.bits.get_bitmask()
            shift = field.bits.lsb_position()
            out.append(f"  .{member_name} = ({member_type}) ((val >> {shift}U) & 0x{mask:X}U), \\")
        out.append(f"}}")
        out.append(f"")

        out.append(f"/** @}} */")
        out.append(f"")

    out.append(f"#endif /* {name.include_guard_macro(name.filename_reg_utils())} */")
    out.append(f"")

    output_file = os.path.join(cli.output_path, name.filename_reg_utils())
    with open(output_file, 'w') as outfile:
        outfile.write("\n".join(out))
    print(f"Generated {output_file}...")
