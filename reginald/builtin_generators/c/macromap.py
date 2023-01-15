import os
from typing import List, Tuple, Union

from reginald.cli import CLI
from reginald.datamodel import *
from reginald.generator import OutputGenerator
from reginald.utils import c_sanitize, str_pad_to_length


class Generator(OutputGenerator):
    @classmethod
    def description(cls):
        return "TODO"

    @classmethod
    def generate(cls, rmap: RegisterMap, cli: CLI):

        mapname = rmap.map_name
        mapname_macro = c_sanitize(mapname).upper()

        out = []

        out.append(f"/*")
        out.append(f"* {mapname} Register Map.")
        out.append(f"* Note: Do not edit: Generated using Reginald.")
        out.append(f"*/")
        out.append(f"")
        out.append(f"#ifndef {mapname_macro}_REG_H_")
        out.append(f"#define {mapname_macro}_REG_H_")
        out.append(f"")

        # combine physical registers and register templates:
        registers = {}  # type: Dict[str, Union[Register, RegisterTemplate]]

        for register in rmap.registers.values():
            registers[register.name] = register

        for block_template in rmap.register_block_templates.values():
            for template in block_template.registers.values():
                registers[block_template.name + template.name] = template

        for reg_name, reg in registers.items():
            reg_name = c_sanitize(reg_name)

            out.append(str_pad_to_length(f"// ==== {reg_name} ", "=", 80))

            out.extend(reg.docs.multi_line(prefix="// "))

            # Generate all defines, keeping comments seperate for now:

            defines = []  # type: List[Tuple[str, str, str]]

            register_prefix = f"{mapname_macro}__REG_{reg_name}"
            if isinstance(reg, Register):
                if reg.adr is not None:
                    docstr = f"({reg.docs.brief})" if reg.docs.brief is not None else ""
                    defines.append((f"#define {register_prefix}", f"(0x{reg.adr:02X}U)", f"// Register Address {docstr}"))
            else:
                docstr = f"({reg.docs.brief})" if reg.docs.brief is not None else ""
                defines.append((f"#define {register_prefix}", f"(0x{reg.offset:02X}U)", f"// Register Offset {docstr}"))

            if reg.reset_val is not None:
                defines.append((f"#define {register_prefix}__RESET", f"(0x{reg.reset_val:02X}U)", f"// Reset Value"))

            if reg.always_write is not None:
                mask = reg.always_write.bits.get_bitmask()
                value = reg.always_write.value
                defines.append((f"#define {register_prefix}__ALWAYSWRITE_MASK", f"(0x{mask:02X}U)", f"// 'Always write' bit mask"))
                defines.append((f"#define {register_prefix}__ALWAYSWRITE_VALUE", f"(0x{value:02X}U)", f"// 'Always write' value"))

            for field in reg.fields.values():
                docstr = f"({field.docs.brief})" if field.docs.brief is not None else ""
                field_name = c_sanitize(field.name)
                field_prefix = f"{register_prefix}__FIELD_{field_name}"
                mask = field.bits.get_bitmask()
                defines.append((f"#define {field_prefix}", f"(0x{mask:02X}U)", f"// Field Mask {docstr}"))

                if field.enum is not None:
                    for const in field.enum.entries.values():
                        docstr = f"({const.docs.brief})" if const.docs.brief is not None else ""
                        const_name = c_sanitize(const.name)
                        const_prefix = f"{field_prefix}__CONST_{const_name}"
                        defines.append((f"#define {const_prefix}", f"(0x{const.value:02X}U)", f"// Constant {docstr}"))

            # Align values and comments:
            define_max_len = max([len(d[0]) for d in defines]) + 1
            val_max_len = max([len(d[1]) for d in defines]) + 1

            for define, val, comment in defines:
                define = define + (" " * (define_max_len - len(define)))
                val = (" " * (val_max_len - len(val))) + val
                out.append(define+val+" "+comment)

            out.append(f"")

        out.append(f"")
        out.append(f"#endif /* {mapname_macro}_REG_H_ */")

        output_file = os.path.join(cli.output_path, f"{rmap.map_name.lower()}_regs.h")
        with open(output_file, 'w') as outfile:
            outfile.write("\n".join(out)+"\n")
        print(f"Generated {output_file}...")
