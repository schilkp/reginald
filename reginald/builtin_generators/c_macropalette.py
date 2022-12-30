from typing import List

from reginald.datamodel import *
from reginald.generator import OutputGenerator
from reginald.utils import c_fitting_unsigned_type, c_sanitize


class Generator(OutputGenerator):
    @classmethod
    def description(cls):
        return "TODO"

    @classmethod
    def generate(cls, map: RegisterMap, args: List[str]):

        out = []
        out.append(f"#include <stdint.h>")
        out.append(f"")
        out.append(f"void main() {{")
        out.append(f"")

        type = c_fitting_unsigned_type(map.register_bitwidth)

        for reg_name_orig, r in map.registers.items():
            reg_name_macro = c_sanitize(reg_name_orig)
            reg_name_var = reg_name_macro.lower()

            out.append(f"  //{reg_name_macro}:")
            out.append(f"  {type} {reg_name_var} = 0;")

            for field_name_orig in r.fields:
                field_name_macro = c_sanitize(field_name_orig)
                out.append(f"  {reg_name_var} = REG_FIELD_SET({reg_name_macro}, {field_name_macro}, {reg_name_var}, X);")

            out.append(f"")
            out.append(f"")

        out.append(f"}}")

        return "\n".join(out)
