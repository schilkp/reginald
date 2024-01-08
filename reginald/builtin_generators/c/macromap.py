from typing import List, Tuple

from reginald.datamodel import *
from reginald.generator import OutputGenerator
from reginald.utils import c_sanitize, str_pad_to_length


class Generator(OutputGenerator):
    @classmethod
    def description(cls):
        return "C header with traditional register and field macros."

    @classmethod
    def generate(cls, rmap: RegisterMap, input_file: str, output_file: str, args: List[str]):

        _ = input_file
        _ = args

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

        # TODO: Block start adrs + offset adrs
        for block_name, block in rmap.registers.items():

            for register_template_name, register_template in block.registers.items():


                out.append(str_pad_to_length(f"// ==== {block_name + register_template_name} ", "=", 80))

                out.extend(register_template.docs.as_multi_line(prefix="// "))

                # Generate all defines, keeping comments seperate for now:
                defines = []  # type: List[Tuple[str, str, str]]

                # Address define, for each instance:
                for instance_name, instance_start in block.instances.items():
                    reg_name = (c_sanitize(instance_name)+c_sanitize(register_template_name)).upper()

                    register_prefix = f"{mapname_macro}__REG_{reg_name}"

                    adr = instance_start + register_template.offset

                    docstr = f"({register_template.docs.brief})" if register_template.docs.brief is not None else ""
                    defines.append((f"#define {register_prefix}", f"(0x{adr:02X}U)", f"// Register Address {docstr}"))

                # Reset value

                register_template_prefix = f"{mapname_macro}__REG_{(c_sanitize(block_name) +c_sanitize(register_template_name)).upper()}"

                if register_template.reset_val is not None:
                    defines.append((f"#define {register_template_prefix}__RESET",
                                   f"(0x{register_template.reset_val:02X}U)", f"// Reset Value"))

                if register_template.always_write is not None:
                    mask = register_template.always_write.bits.get_bitmask()
                    value = register_template.always_write.value
                    defines.append((f"#define {register_template_prefix}__ALWAYSWRITE_MASK",
                                   f"(0x{mask:02X}U)", f"// 'Always write' bit mask"))
                    defines.append((f"#define {register_template_prefix}__ALWAYSWRITE_VALUE",
                                   f"(0x{value:02X}U)", f"// 'Always write' value"))

                for field in register_template.fields.values():
                    docstr = f"({field.docs.brief})" if field.docs.brief is not None else ""
                    field_name = c_sanitize(field.name)
                    field_prefix = f"{register_template_prefix}__FIELD_{field_name}"
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

        with open(output_file, 'w') as outfile:
            outfile.write("\n".join(out)+"\n")
        print(f"Generated {output_file}...")
