from typing import List, Tuple

from reginald.datamodel import *
from reginald.generator import OutputGenerator
from reginald.utils import c_sanitize


class Generator(OutputGenerator):
    @classmethod
    def description(cls):
        return "TODO"

    @classmethod
    def generate(cls, map: RegisterMap, args: List[str]):

        dev_name = map.device_name
        dev_macro = c_sanitize(dev_name).upper()

        out = []

        out.append(f"/*")
        out.append(f"* {dev_name} Register Map.")
        out.append(f"* Note: do not edit: Generated using Reginald.")
        out.append(f"*/")
        out.append(f"")
        out.append(f"#ifndef {dev_macro}_REG_H_")
        out.append(f"#define {dev_macro}_REG_H_")
        out.append(f"")

        for reg_name_orig, r in map.registers.items():
            reg_name = c_sanitize(reg_name_orig)

            title_line = f"// ==== {reg_name} "
            if len(title_line) < 80:
                title_line += ("=" * (80 - len(title_line)))
            out.append(title_line)

            if r.brief is not None:
                out.append(f"// {r.brief}")

            if r.doc is not None:
                for l in r.doc.splitlines():
                    out.append(f"// {l}")

            # Generate all defines, keeping comments seperate for now:

            defines = []  # type: List[Tuple[str, str, str]]

            register_prefix = f"{dev_macro}__REG_{reg_name}"
            if r.adr is not None:
                docstr = f"({r.brief})" if r.brief is not None else ""
                defines.append((f"#define {register_prefix}", f"(0x{r.adr:02X}U)", f"// Register Address {docstr}"))

            if r.reserved_val is not None:
                defines.append((f"#define {register_prefix}__RESERVED", f"(0x{r.reserved_val:02X}U)", f"// Reserved Bits"))

            for field_name_orig, field in r.fields.items():
                docstr = f"({field.brief})" if field.brief is not None else ""
                field_name = c_sanitize(field_name_orig)
                field_prefix = f"{register_prefix}__FIELD_{field_name}"
                defines.append((f"#define {field_prefix}",
                               f"(0x{field.get_bits().get_bitmask():02X}U)", f"// Field Mask {docstr}"))

                if field.enum is not None:
                    for const_name_orig, const in field.enum.items():
                        docstr = f"({const.brief})" if const.brief is not None else ""
                        const_name = c_sanitize(const_name_orig)
                        const_prefix = f"{field_prefix}__CONST_{const_name}"
                        defines.append((f"#define {const_prefix}", f"(0x{const.value:02X}U)", f"// Constant {docstr}"))

                if field.accepts_enum is not None:
                    for const_name_orig, const in map.enums[field.accepts_enum].items():
                        docstr = f"({const.brief})" if const.brief is not None else ""
                        const_name = c_sanitize(const_name_orig)
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
        out.append(f"#endif /* {dev_macro}_REG_H_ */")

        return "\n".join(out)
