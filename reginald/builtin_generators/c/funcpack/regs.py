import os
from typing import Union

from reginald.builtin_generators.c.funcpack.name_generator import NameGenerator
from reginald.builtin_generators.c.funcpack.utils import doxy_comment
from reginald.cli import CLI
from reginald.datamodel import *
from reginald.utils import str_pad_to_length


def generate(map: RegisterMap, name: NameGenerator, cli: CLI):

    out = []

    out.append(f"/*!")
    out.append(f" * \\file {name.filename_regs()}")
    out.append(f" * \\brief {map.map_name} Registers.")
    out.append(f" * \\note Do not edit: Generated using Reginald.")
    out.append(f" */")
    out.append(f"")
    out.append(f"")

    out.append(f"#ifndef {name.include_guard_macro(name.filename_regs())}")
    out.append(f"#define {name.include_guard_macro(name.filename_regs())}")
    out.append(f"")
    out.append(f"#include <stdint.h>")
    out.append(f"#include \"{name.filename_enums()}\"")
    out.append(f"")

    out.append(str_pad_to_length(f"// ==== Register Addresses ", "=", 80))
    for reg in map.registers.values():
        if reg.adr is not None:
            if reg.docs.brief is not None:
                comment = f"//!< {reg.name} Address ({reg.docs.brief})"
            else:
                comment = f"//!< {reg.name} Address"
            out.append(f"#define {name.reg_adr_macro(reg.name)} (0x{reg.adr:X}U) {comment}")
    out.append(f"")

    for block in map.register_block_templates.values():
        out.append(str_pad_to_length(f"// ==== {block.name} Register Block ", "=", 80))
        for template in block.registers.values():
            regname = block.name + template.name
            out.append(f"#define {name.block_offset_macro(regname)} ({template.offset}U)")
        out.append(f"")
        for startadr, instname in block.instances.items():
            out.append(f"#define {name.block_instance_start_macro(block.name, instname)} (0x{startadr:X}U)")
    out.append(f"")

    registers = {}  # type: Dict[str, Union[Register, RegisterTemplate]]
    for reg in map.registers.values():
        if not reg.originates_from_template:
            registers[reg.name] = reg
    for block in map.register_block_templates.values():
        for template in block.registers.values():
            registers[block.name + template.name] = template

    for reg_name, reg in registers.items():

        out.append(str_pad_to_length(f"// ==== {reg_name} ", "=", 80))
        out.append(f"")

        if reg.reset_val is not None:
            out.append(f"#define {name.reg_resetval_macro(reg_name)} (0x{reg.reset_val:X}U) //!< {reg_name} Reset Value")
            out.append(f"")

        if reg.always_write is not None:
            mask = reg.always_write.bits.get_bitmask()
            value = reg.always_write.value
            out.append(f"#define {name.reg_alwayswrite_mask_macro(reg_name)} (0x{mask:X}U) //!< {reg_name} Always Write Mask")
            out.append(f"#define {name.reg_alwayswrite_val_macro(reg_name)} (0x{value:X}U) //!< {reg_name} Always Write Value")
            out.append(f"")

        if len(reg.fields) == 0:
            # Don't generate structs + funcs if there are no fields.
            continue

        # Generate register struct:
        out.extend(doxy_comment(reg.docs, prefix=""))
        out.append(f"struct {name.reg_struct_name(reg_name)} {{")
        for field in reg.fields.values():
            type = name.reg_struct_member_type(reg_name, field)
            out.extend(doxy_comment(field.docs, prefix="  "))
            out.append(f"  {type} {name.reg_struct_member(field)} : {field.bits.total_width()};")
        out.append(f"}};")
        out.append(f"")

    out.append(f"")
    out.append(f"#endif /* {name.include_guard_macro(name.filename_regs())} */")

    output_file = os.path.join(cli.output_path, name.filename_regs())
    with open(output_file, 'w') as outfile:
        outfile.write("\n".join(out))
    print(f"Generated {output_file}...")