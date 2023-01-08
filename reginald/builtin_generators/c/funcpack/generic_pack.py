from typing import List

from reginald.datamodel import *
from reginald.generator import OutputGenerator
from reginald.utils import c_sanitize


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

        out.append(f"/*")
        out.append(f"* {devname} Register Packing/Unpacking Generic.")
        out.append(f"* Note: do not edit: Generated using Reginald.")
        out.append(f"*/")
        out.append(f"")
        out.append(f"#ifndef {devname_macro}_PACKING_H_")
        out.append(f"#define {devname_macro}_PACKING_H_")
        out.append(f"")

        out.append(f"#define {devname_macro}_PACK(_struct_ptr_) _Generic((_struct_ptr_), \\")
        for regname_orig in map.registers:
            regname_c = c_sanitize(regname_orig).lower()
            out.append(f"    struct reg_{regname_c}*: pack_{regname_c},\\")
        out.append(f")(_struct_ptr_)")

        out.append(f"")
        out.append(f"#endif /* {devname_macro}_PACKING_H_ */")
        return "\n".join(out)
