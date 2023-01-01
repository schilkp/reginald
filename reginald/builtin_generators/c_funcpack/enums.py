from typing import List

from reginald.datamodel import *
from reginald.generator import OutputGenerator
from reginald.utils import c_sanitize, doxy_comment, str_pad_to_length


class Generator(OutputGenerator):
    @classmethod
    def description(cls):
        return "TODO"

    @classmethod
    def generate(cls, map: RegisterMap, args: List[str]):
        devname = map.device_name

        devname_macro = c_sanitize(devname).upper()
        devname_c = c_sanitize(devname).lower()

        out = []

        out.append(f"/*!")
        out.append(f" * \\file {devname_c}_reg_enums.h")
        out.append(f" * \\brief {devname} Register Enums.")
        out.append(f" * \\note Do not edit: Generated using Reginald.")
        out.append(f" */")
        out.append(f"")
        out.append(f"")
        out.append(f"#ifndef {devname_macro}_REG_ENUMS_H_")
        out.append(f"#define {devname_macro}_REG_ENUMS_H_")
        out.append(f"")

        out.append(str_pad_to_length(f"// ==== Global Enums ", "=", 80))

        out.append(f"")

        for enumname_orig, enum in map.enums.items():
            enumname_c = c_sanitize(enumname_orig).lower()
            enumname_macro = c_sanitize(enumname_orig).upper()
            out.append(f"typedef enum {{")

            for entryname_orig, entry in enum.items():
                entryname_macro = c_sanitize(entryname_orig).upper()
                out.extend(doxy_comment(entry.brief, entry.doc))
                out.append(f"  {devname_macro}_{enumname_macro}_{entryname_macro} = 0x{entry.value:X}U,")

            out.append(f"}} {devname_c}_{enumname_c}_t;")
            out.append(f"")

        for registername_orig, reg in map.registers.items():

            enum_count = len([field.enum for field in reg.fields.values() if field.enum is not None])

            if enum_count == 0:
                continue

            out.append(f"")
            out.append(str_pad_to_length(f"// ==== {registername_orig} Enums ", "=", 80))
            out.append(f"")

            for fieldname_orig, field in reg.fields.items():
                fieldname_c = c_sanitize(fieldname_orig).lower()
                fieldname_macro = c_sanitize(fieldname_orig).upper()

                if field.enum is not None:
                    out.extend(doxy_comment(field.brief, field.doc))
                    out.append(f"typedef enum {{")

                    for entryname_orig, entry in field.enum.items():
                        entryname_macro = c_sanitize(entryname_orig).upper()
                        out.extend(doxy_comment(entry.brief, entry.doc))
                        out.append(f"  {devname_macro}_{fieldname_macro}_{entryname_macro} = 0x{entry.value:X}U,")

                    out.append(f"}} {devname_c}_{fieldname_c}_t;")
                    out.append(f"")

        out.append(f"#endif /* {devname_macro}_REG_ENUMS_H_ */")
        return "\n".join(out)
