import os
from math import inf

from tabulate import tabulate

from reginald.cli import CLI
from reginald.datamodel import *
from reginald.generator import OutputGenerator
from reginald.utils import str_list


class Generator(OutputGenerator):
    @classmethod
    def description(cls):
        return "TODO"

    @classmethod
    def generate(cls, map: RegisterMap, cli: CLI):
        out = []

        # Generate header:
        out.append(f"# {map.map_name} Register Map")
        out.extend(map.docs.multi_line(prefix=""))
        out.append("")

        # Generate overview table:
        out.append(f"## Overview:")
        out.append("")
        rows = []

        def sort_by_adr(x: str) -> float:
            adr = map.registers[x].adr
            if adr is None:
                return inf
            else:
                return adr
        regs = sorted(map.registers.keys(), key=sort_by_adr)

        for reg_name in regs:
            reg = map.registers[reg_name]

            if reg.adr is None:
                adr = "?"
            else:
                adr = f"0x{reg.adr:X}"

            fields = str_list(reg.fields.keys())
            rows.append([adr, reg_name, fields])

        out.append("")
        out.append(tabulate(rows, headers=["Address", "Register", "Fields"], tablefmt="pipe"))
        out.append("")

        # Generate register section:

        out.append(f"## Registers:")
        for reg in map.registers.values():

            # Register name:
            out.append(f"### {reg.name}:")

            # Register info:
            out.extend(reg.docs.two_line(prefix=" -"))
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
            register_bitranges.extend(reg.get_unused_bits(include_always_write=True).get_bitranges())

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
                        access_row.append(field.access_str())
                    else:
                        access_row.append("?")
                elif reg.is_bit_always_write(bitrange.lsb_position):
                    access_row.append("?")
                    val = reg.get_always_write_value(Bits.from_bitrange(bitrange))
                    field = f"Always write 0x{val:x}"
                    field_row.append(field)
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
                    access_str = f" [{field.access_str()}]"
                else:
                    access_str = ""

                out.append("")
                out.append(f"  - {field_name}{access_str}:")

                # Documentation (if any):
                out.extend(field.docs.two_line(prefix=" -"))

                # Accepted values (through local or global enum):
                if field.enum is not None:
                    out.append(f"    - Accepts:")
                    for entry in field.enum.entries.values():
                        if entry.docs.brief is not None:
                            out.append(f"      - {entry.name}: 0x{entry.value:X} ({entry.docs.brief})")
                        else:
                            out.append(f"      - {entry.name}: 0x{entry.value:X}")

            # horizontal rule:
            out.append("")
            out.append("---")

        output_file = os.path.join(cli.output_path, f"{map.map_name.lower()}.md")
        with open(output_file, 'w') as outfile:
            outfile.write("\n".join(out))
        print(f"Generated {output_file}...")
