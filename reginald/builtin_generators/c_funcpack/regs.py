from typing import List

from reginald.datamodel import *
from reginald.generator import OutputGenerator
from reginald.utils import (c_fitting_unsigned_type, c_sanitize, doxy_comment,
                            str_pad_to_length)


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
        out.append(f" * \\file {devname_c}_regs.h")
        out.append(f" * \\brief {devname} Register Map.")
        out.append(f" * \\note Do not edit: Generated using Reginald.")
        out.append(f" */")
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

            out.append(str_pad_to_length(f"// ==== {regname_orig} ", "=", 80))

            # Combine brief + doc to include in doc of register address:
            docstr = ""
            if reg.brief is not None:
                docstr += reg.brief
            if reg.doc is not None:
                docstr += "\n"+reg.doc

            # Generate register address:
            if reg.adr is not None:
                out.extend(doxy_comment(f" {regname_orig} Register Address", docstr))
                out.append(f"#define {devname_macro}_REG_{regname_macro} (0x{reg.adr:X}U)")
                out.append(f"")

            if reg.reserved_val is not None:
                out.extend(doxy_comment(f" {regname_orig} Default Initialiser", "Correctly sets reserved bits"))
                out.append(f"#define {devname_macro}_REG_{regname_macro}__RESERVED (0x{reg.reserved_val:X}U)")
                out.append(f"")

            if len(reg.fields) == 0:
                # Don't generate structs + funcs if there are no fields.
                continue

            # Generate register struct:
            out.extend(doxy_comment(reg.brief, reg.doc))
            out.append(f"struct reg_{regname_c} {{")

            types = {}

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

                types[fieldname_orig] = type

                out.extend(doxy_comment(field.brief, field.doc))
                out.append(f"  {type} {fieldname_c} : {field.get_bits().width()};")

            out.append(f"}};")
            out.append(f"")

            # Generate packing and unpacking function:
            packed_type = c_fitting_unsigned_type(map.register_bitwidth)

            out.append(f"/**")
            out.append(f" * @brief Pack the '{regname_c}' register's fields into their binary representation")
            out.append(f" * @param r struct holding register fields")
            out.append(f" * @return packed register representation")
            out.append(f" */")
            out.append(f"static inline {packed_type} pack_reg_{regname_c}(const struct reg_{regname_c} *r){{")
            if reg.reserved_val is not None:
                default_initialiser = f"{devname_macro}_REG_{regname_macro}__RESERVED"
                out.append(f"  {packed_type} packed = {default_initialiser};")
            else:
                out.append(f"  {packed_type} packed = 0x0U;")
            for fieldname_orig, field in reg.fields.items():
                fieldname_c = c_sanitize(fieldname_orig).lower()
                mask = field.get_bits().get_unpositioned_bits().get_bitmask()
                shift = field.get_bits().lsb_position()
                out.append(f"  packed |= (r->{fieldname_c} & 0x{mask:X}U) << {shift}U;")
            out.append(f"  return packed;")
            out.append(f"}}")
            out.append(f"")

            out.append(f"/**")
            out.append(f" * @brief Unpack the '{regname_c}' register's binary representation into seperate fields")
            out.append(f" * @param r buffer to store the unpacked fields")
            out.append(f" * @param val packed register representation")
            out.append(f" */")
            out.append(f"static inline void unpack_reg_{regname_c}(struct reg_{regname_c} *r, {packed_type} val){{")
            for fieldname_orig, field in reg.fields.items():
                fieldname_c = c_sanitize(fieldname_orig).lower()
                mask = field.get_bits().get_unpositioned_bits().get_bitmask()
                shift = field.get_bits().lsb_position()
                out.append(f"  r->{fieldname_c} = ({types[fieldname_orig]}) ((val >> {shift}U) & 0x{mask:X}U);")
            out.append(f"}}")
            out.append(f"")

            out.append(f"/**")
            out.append(f" * @brief Unpack the '{regname_c}' register's binary representation into a struct initialiser.")
            out.append(f" * @note use static unpack_reg_{regname_c}() to unpack into an exsisting struct.")
            out.append(f" * Example:")
            out.append(f" *   `struct reg_{regname_c} {regname_c} = UNPACK_REG_{regname_macro}(0xAB);`")
            out.append(f" * ")
            out.append(f" * @param _VAL_ packed register representation")
            out.append(f" */")
            out.append(f"#define UNPACK_REG_{regname_macro}(_VAL_){{\\")
            for fieldname_orig, field in reg.fields.items():
                fieldname_c = c_sanitize(fieldname_orig).lower()
                mask = field.get_bits().get_unpositioned_bits().get_bitmask()
                shift = field.get_bits().lsb_position()
                out.append(f"  .{fieldname_c} = ({types[fieldname_orig]}) (((_VAL_) >> {shift}U) & 0x{mask:X}U), \\")
            out.append(f"}}")
            out.append(f"")

        out.append(f"")
        out.append(f"#endif /* {devname_macro}_REG_H_ */")
        return "\n".join(out)
