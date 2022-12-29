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

        # Generate header:
        out.append(f"# {map.device_name} Register Map")
        out.append(f"Register bit width: {map.register_bitwidth}")
        out.append("")

        # Generate overview table:

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
        for reg_name in regs:
            reg = map.registers[reg_name]

            if reg.adr is None:
                adr = "?"
            else:
                adr = f"0x{reg.adr:X}"

            fields = ", ".join(reg.fields.keys())
            rows.append([adr, reg_name, fields])

        out.append("")
        out.append(tabulate(rows, headers=["Address", "Register", "Fields"], tablefmt="pipe"))
        out.append("")

        # Generate register section:

        out.append(f"## Registers:")
        for reg_name, reg in map.registers.items():

            # Register name:
            out.append(f"### {reg_name}:")

            # Register info:
            if reg.doc is not None:
                doc = str.join(" ", reg.doc.splitlines())
                out.append(f" - {doc}")
            if reg.adr is not None:
                out.append(f" - Address: 0x{reg.adr:X}")
            if reg.reset_val is not None:
                out.append(f" - Reset Val: 0x{reg.reset_val:X}")

            # Register bitfields table:

            # Collect all bitranges that make up this register - field or not:
            register_bitranges = []  # type: List[BitRange]
            for field in reg.fields.values():
                for range in field.get_bitranges():
                    register_bitranges.append(range)
            register_bitranges.extend(reg.get_unused_bitranges(map.register_bitwidth))

            # Sort bitranges:
            register_bitranges = sorted(register_bitranges, key=lambda x: x.lsb_position, reverse=True)

            bitrow = ["Bits:"]
            field_row = ["Field:"]
            access_row = ["Access:"]

            for bitrange in register_bitranges:
                # Retrieve field that coresponds to this range (if any):
                field_name = reg.get_fieldname_at(bitrange.lsb_position)

                bitrow.append(str(bitrange))

                if field_name is not None:
                    field = reg.fields[field_name]

                    field_row.append(field_name)
                    if field.access is not None:
                        access_row.append(field.access)
                    else:
                        access_row.append("?")
                else:
                    access_row.append("?")
                    field_row.append("?")

            out.append("")
            out.append(tabulate([bitrow, field_row, access_row], headers="firstrow",
                                tablefmt="pipe", numalign="center", stralign="center"))
            out.append("")

            # Field info:
            out.append("")
            out.append(f"*Bitfields*:")

            for field_name, field in reg.fields.items():

                # Access (if any):
                if field.access is not None:
                    access_str = f" [{field.access}]"
                else:
                    access_str = ""

                out.append("")
                out.append(f"  - {field_name}{access_str}:")

                # Documentation (if any):
                if field.doc is not None:
                    doc = str.join(" ", field.doc.splitlines())
                    out.append(f"    - {doc}")

                # Accepted values (through local or global enum):
                if field.enum is not None or field.accepts_enum is not None:
                    out.append(f"    - Values:")

                    # Global enum:
                    if field.accepts_enum is not None:
                        out.append(f"      - A value from enum {field.accepts_enum}:")
                        for key, entry in map.enums[field.accepts_enum].items():
                            if entry.doc is not None:
                                doc = str.join(" ", entry.doc.splitlines())
                                out.append(f"        - {key}: 0x{entry.value:X} ({doc})")
                            else:
                                out.append(f"        - {key}: 0x{entry.value:X}")

                    # Local enum:
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
            for reg_name, enum in map.enums.items():
                out.append(f"### {reg_name}:")
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
