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
        devname = map.device_name

        devname_macro = c_sanitize(devname).upper()
        devname_c = c_sanitize(devname).lower()

        out = []

        out.append(f"/*")
        out.append(f"* {devname} Register Map.")
        out.append(f"* Note: do not edit: Generated using Reginald.")
        out.append(f"*/")
        out.append(f"")
        out.append(f"#ifndef {devname_macro}_REGS_H_")
        out.append(f"#define {devname_macro}_REGS_H_")
        out.append(f"")
        out.append(f"#include <stdint.h>")
        out.append(f"#include \"{devname_c}_reg_enums.h\"")
        out.append(f"")

        for regname_orig, reg in map.registers.items():
            regname_c = c_sanitize(regname_orig).lower()
            regname_macro = c_sanitize(regname_orig).upper()

            title_line = f"// ==== {regname_orig} "
            if len(title_line) < 80:
                title_line += ("=" * (80 - len(title_line)))
            out.append(title_line)

            if reg.doc is not None:
                for l in reg.doc.splitlines():
                    out.append(f"// {l}")

            out.append(f"")

            # Generate register address:
            out.append(f"#define {devname_macro}_REG_{regname_macro} (0x{reg.adr:X}U)")
            out.append(f"")

            # Generate register struct:
            out.append(f"struct reg_{regname_c} {{")

            for fieldname_orig, field in reg.fields.items():
                fieldname_c = c_sanitize(fieldname_orig).lower()

                if field.enum is not None and field.accepts_enum is not None:
                    raise ReginaldException(
                        f"c_funcpack does not support fields that accept more than one enum ({regname_orig}: {fieldname_orig})")

                if field.accepts_enum is not None:
                    enumname_c = c_sanitize(field.accepts_enum).lower()
                    type = f"{devname_c}_{enumname_c}_t"
                elif field.enum is not None:
                    type = f"{devname_c}_{fieldname_c}_t"
                else:
                    type = c_fitting_unsigned_type(field.get_bits().width())

                out.append(f"  {type} {fieldname_c} : {field.get_bits().width()};")

            out.append(f"}};")
            out.append(f"")

            # Generate packing and unpacking function:
            packed_type = c_fitting_unsigned_type(map.register_bitwidth)

            out.append(f"static inline {packed_type} pack_reg_{regname_c}(const struct reg_{regname_c} *r){{")
            out.append(f"  {packed_type} packed = 0;")
            for fieldname_orig, field in reg.fields.items():
                fieldname_c = c_sanitize(fieldname_orig).lower()
                mask = field.get_bits().get_unpositioned_bits().get_bitmask()
                shift = field.get_bits().lsb_position()
                out.append(f"  packed |= (r->{fieldname_c} & 0x{mask}U) << {shift}U;")
            out.append(f"  return packed;")
            out.append(f"}}")
            out.append(f"")

            out.append(f"static inline void unpack_reg_{regname_c}(struct reg_{regname_c} *r, {packed_type} val){{")
            for fieldname_orig, field in reg.fields.items():
                fieldname_c = c_sanitize(fieldname_orig).lower()
                mask = field.get_bits().get_unpositioned_bits().get_bitmask()
                shift = field.get_bits().lsb_position()
                out.append(f"  r->{fieldname_c} = (val >> {shift}U) & 0x{mask:X}U;")
            out.append(f"}}")
            out.append(f"")

            out.append(f"#define UNPACK_REG_{regname_macro}(_VAL_){{\\")
            for fieldname_orig, field in reg.fields.items():
                fieldname_c = c_sanitize(fieldname_orig).lower()
                mask = field.get_bits().get_unpositioned_bits().get_bitmask()
                shift = field.get_bits().lsb_position()
                out.append(f"  .{fieldname_c} = (((_VAL_) >> {shift}U) & 0x{mask:X}U), \\")
            out.append(f"}}")
            out.append(f"")

        out.append(f"")
        out.append(f"#endif /* {devname_macro}_REG_H_ */")
        return "\n".join(out)
