from copy import copy

from tabulate import tabulate

from reginald.datamodel import *
from reginald.generator import OutputGenerator


class Generator(OutputGenerator):
    @classmethod
    def description(cls):
        return "TODO"

    @classmethod
    def generate(cls, map: RegisterMap, args: List[str]):
        out = []

        out.append(f"# {map.device_name} Register Map")
        out.append(f"Register bit width: {map.register_bitwidth}")
        out.append("")

        out.append(f"## Overview:")
        out.append("")
        rows = []

        def sort_by_adr(x: str) -> int:
            adr = map.registers[x].adr
            if adr is None:
                return (2 ** map.register_bitwidth) + 1
            else:
                return adr

        regs = sorted(map.registers.keys(), key=sort_by_adr)
        for name in regs:
            r = map.registers[name]

            if r.adr is None:
                adr = "?"
            else:
                adr = f"0x{r.adr:X}"

            fields = ", ".join(r.fields.keys())
            rows.append([adr, name, fields])

        out.append("")
        out.append(tabulate(rows, headers=["Address", "Register", "Fields"], tablefmt="pipe"))
        out.append("")

        out.append(f"## Registers:")
        for name, r in map.registers.items():
            out.append(f"### {name}:")
            if r.doc is not None:
                doc = str.join(" ", r.doc.splitlines())
                out.append(f" - {doc}")
            if r.adr is not None:
                out.append(f" - Address: 0x{r.adr:X}")
            if r.reset_val is not None:
                out.append(f" - Reset Val: 0x{r.reset_val:X}")

            # Split field into seperate fields for every consecutive interval of bits it
            # occupies. (Required if a register field is non-continous).
            seperated_fields = []
            for name, field in r.fields.items():
                for bitrange in field.get_bits().get_bitranges():
                    f = copy(field)
                    f.bits = bitrange.get_bitlist()
                    seperated_fields.append((name, f))

            # Fill missing/unused bits:
            for bitrange in r.get_unused_bits(map.register_bitwidth).get_bitranges():
                seperated_fields.append(("[UNSPECIFIED]", Field(bits=bitrange.get_bitlist())))

            # Sort fields:
            seperated_fields = sorted(
                seperated_fields, key=lambda x: x[1].get_bits().lsb_position(), reverse=True)

            # Generate table:
            bitrow = ["Bits:"]
            field_row = ["Field:"]
            access_row = ["Access:"]

            for name, field in seperated_fields:
                bitrow.append(str(field.get_bits().get_bitrange()))
                field_row.append(name)
                if field.access is not None:
                    access_row.append(field.access)
                else:
                    access_row.append("")

            out.append("")
            out.append(tabulate([bitrow, field_row, access_row], headers="firstrow",
                                tablefmt="pipe", numalign="center", stralign="center"))
            out.append("")

            # Field info:
            out.append("")
            out.append(f"*Bitfields* :")
            for name, field in r.fields.items():
                out.append("")
                out.append(f"  - {name}:")
                if field.doc is not None:
                    doc = str.join(" ", field.doc.splitlines())
                    out.append(f"    - Description:  {doc}")
                if field.access is not None:
                    out.append(f"    - Access: {field.access}")
                    out.append("")
                if field.enum is not None or field.accepts_enum is not None:
                    out.append(f"    - Values:")
                    if field.accepts_enum is not None:
                        out.append(f"      - A value from enum {field.accepts_enum}:")
                        for key, entry in map.enums[field.accepts_enum].items():
                            if entry.doc is not None:
                                doc = str.join(" ", entry.doc.splitlines())
                                out.append(f"        - {key}: 0x{entry.value:X} ({doc})")
                            else:
                                out.append(f"        - {key}: 0x{entry.value:X}")
                    if field.enum is not None:
                        for key, entry in field.enum.items():
                            if entry.doc is not None:
                                doc = str.join(" ", entry.doc.splitlines())
                                out.append(f"      - {key}: 0x{entry.value:X} ({doc})")
                            else:
                                out.append(f"      - {key}: 0x{entry.value:X}")

            # horizontal rule:
            out.append("")
            out.append("---")

        if map.enums is not None:
            out.append(f"## Enums:")
            out.append("")
            for name, enum in map.enums.items():
                out.append(f"### {name}:")
                out.append("")
                table_rows = []
                for key, entry in enum.items():
                    if entry.doc is not None:
                        table_rows.append([key, hex(entry.value), entry.doc])
                    else:
                        table_rows.append([key, hex(entry.value), "?"])
                out.append("")
                out.append(tabulate(table_rows, headers=["Name", "Value", "Description"], tablefmt="pipe"))
                out.append("")

            out.append("")

        return "\n".join(out)
