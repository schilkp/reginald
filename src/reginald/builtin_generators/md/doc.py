from typing import List

from tabulate import tabulate

from reginald.datamodel import RegisterMap, Bits
from reginald.generator import OutputGenerator
from reginald.utils import str_list


class Generator(OutputGenerator):
    def description(self):
        return "Markdown register documentation."

    def generate(self, rmap: RegisterMap, input_file: str, output_file: str, args: List[str]):
        out = []

        _ = input_file
        _ = args

        registers = []
        for block in rmap.register_blocks.values():
            for template_name, template in block.register_templates.items():
                for instance_name, instance_adr in block.instances.items():
                    register_name = instance_name + template_name
                    register_adr = instance_adr + template.adr

                    registers.append((register_adr, register_name, template))

        registers.sort(key=lambda x: x[0])

        # Generate header:
        out.append(f"# {rmap.map_name} Register Map")
        out.extend(rmap.docs.as_multi_line(prefix=""))
        out.append("")

        # Generate overview table:
        out.append(f"## Overview:")
        out.append("")
        rows = []

        for reg_adr, reg_name, template in registers:

            fields = str_list(template.fields.keys())
            rows.append([hex(reg_adr), reg_name, fields])
        out.append("")
        out.append(tabulate(rows, headers=["Address", "Register", "Fields"], tablefmt="pipe"))
        out.append("")

        # Generate register section:

        out.append(f"## Registers:")
        for reg_adr, reg_name, template in registers:
            # Register name:
            out.append(f"### {reg_name}:")

            # Register info:
            out.extend(template.docs.as_two_line(prefix=" - "))
            out.append(f" - Address: 0x{reg_adr:X}")
            if template.reset_val is not None:
                out.append(f" - Reset Val: 0x{template.reset_val:X}")

            # Register bitfields table:

            # Collect all bitranges that make up this register - field or not:
            register_bitranges = []
            for field in template.fields.values():
                for range in field.get_bitranges():
                    register_bitranges.append(range)
            register_bitranges.extend(template.get_unused_bits(include_always_write=True).get_bitranges())

            # Sort bitranges:
            register_bitranges = sorted(register_bitranges, key=lambda x: x.lsb_position, reverse=True)

            bitrow = ["Bits:"]
            field_row = ["Field:"]
            access_row = ["Access:"]

            for bitrange in register_bitranges:
                # Retrieve field that coresponds to this range (if any):
                field_name = template.get_fieldname_at(bitrange.lsb_position)

                bitrow.append(str(bitrange))

                if field_name is not None:
                    field = template.fields[field_name]

                    field_row.append(field_name)
                    if field.access is not None:
                        access_row.append(field.access_str())
                    else:
                        access_row.append("?")

                elif template.is_bit_always_write(bitrange.lsb_position):
                    access_row.append("")
                    val = template.get_always_write_value(Bits.from_bitrange(bitrange))
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

            for field_name, field in template.fields.items():

                # Access (if any):
                if len(field.access) > 0:
                    access_str = f" [{field.access_str()}]"
                else:
                    access_str = ""

                out.append("")
                out.append(f"  - {field_name}{access_str}:")

                # Documentation (if any):
                out.extend(field.docs.as_two_line(prefix="    - "))

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

        with open(output_file, 'w') as outfile:
            outfile.write("\n".join(out))
