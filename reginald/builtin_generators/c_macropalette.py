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

        out = []
        out.append(f"#include <stdint.h>")
        out.append(f"")
        out.append(f"void main() {{")
        out.append(f"")

        possible_variable_sizes = [8, 16, 32, 64]
        possible_variable_sizes = [size for size in possible_variable_sizes if size >= map.register_bitwidth]
        if len(possible_variable_sizes) == 0:
            raise ReginaldException(f"No valid c type found for to store {map.register_bitwidth} bits!")

        variable_size = min(possible_variable_sizes)

        type = f"uint{variable_size}_t"

        for reg_name_orig, r in map.registers.items():
            reg_name_macro = c_sanitize(reg_name_orig)
            reg_name_var = reg_name_macro.lower()

            out.append(f"  //{reg_name_macro}:")
            out.append(f"  {type} {reg_name_var} = 0;")

            for field_name_orig, f in r.fields.items():
                field_name_macro = c_sanitize(field_name_orig)
                out.append(f"  {reg_name_var} = REG_FIELD_SET({reg_name_macro}, {field_name_macro}, {reg_name_var}, X);")

            out.append(f"")
            out.append(f"")

        out.append(f"}}")

        return "\n".join(out)
