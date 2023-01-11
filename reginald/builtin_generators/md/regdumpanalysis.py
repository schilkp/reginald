import os
from typing import Dict, List, Union

import yaml
from pydantic import ValidationError
from pydantic.dataclasses import dataclass
from tabulate import tabulate
from yaml import SafeLoader

from reginald.cli import CLI
from reginald.datamodel import *
from reginald.error import ReginaldException
from reginald.generator import OutputGenerator


@dataclass
class YamlBinaryDump:
    binary: Dict[NonNegativeInt, Union[List[NonNegativeInt], NonNegativeInt]]

    @classmethod
    def from_yaml_file(cls, file_name: str):
        try:
            with open(file_name) as f:
                data = yaml.load(f, Loader=SafeLoader)
                return YamlBinaryDump(**data)
        except FileNotFoundError:
            raise ReginaldException(f"File {file_name} not found")
        except ValidationError as e:
            raise ReginaldException(str(e))

    def flatten(self) -> Dict[NonNegativeInt, NonNegativeInt]:
        dump = {}  # type: Dict[NonNegativeInt, NonNegativeInt]

        for at, values in self.binary.items():

            if isinstance(values, List):
                for value in values:
                    if at in dump:
                        raise ReginaldException(f"YamlBinaryDump has two values at address {at}!")
                    dump[at] = value
                    at = at+1

            else:
                if at in dump:
                    raise ReginaldException(f"YamlBinaryDump has two values at address {at}!")
                dump[at] = values

        return dump


class Generator(OutputGenerator):
    @classmethod
    def description(cls):
        return "TODO"

    @classmethod
    def generate(cls, rmap: RegisterMap, cli: CLI):

        if len(cli.generator_args) != 1:
            raise ReginaldException("md_dumpanalysis requires a yaml binary dump as it's only argument")

        dump_yaml = YamlBinaryDump.from_yaml_file(cli.generator_args[0])
        dump = dump_yaml.flatten()
        adrs = sorted(dump.keys())

        out = []

        # Generate header:
        out.append(f"# {rmap.map_name} Register Dump Analysis")
        out.append(f"")
        out.append(f"")

        for adr in adrs:
            register_name = rmap.get_registername_at(adr)

            if register_name is not None:
                reg = rmap.registers[register_name]
                out.append(f"## 0x{adr:0X} - {register_name}")
                out.append(f"  - 0x{dump[adr]:X}")
                out.append(f"  - 0b{dump[adr]:b}")

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
                value_row = ["Value:"]
                decode_row = ["Decode:"]

                for bitrange in register_bitranges:
                    bitrow.append(str(bitrange))

                    field_val = bitrange.extract_this_field_from(dump[adr])

                    value_row.append(f"0x{field_val:X}")

                    # Retrieve field that coresponds to this range (if any):
                    field_name = reg.get_fieldname_at(bitrange.lsb_position)

                    if field_name is not None:
                        field_row.append(field_name)

                        # Lookup if this value in this value coresponds to an enum:
                        field = reg.fields[field_name]
                        if field.enum is not None:
                            enum_entryname = field.lookup_enum_entry_name(field_val)
                            if enum_entryname is not None:
                                decode_row.append(f"{enum_entryname} (0x{field_val:X})")
                            else:
                                decode_row.append(f"DECODE ERROR")
                        else:
                            decode_row.append(f"?")

                    elif reg.is_bit_always_write(bitrange.lsb_position):
                        val = reg.get_always_write_value(Bits.from_bitrange(bitrange))
                        field = f"Always write 0x{val:x}"
                        field_row.append(field)
                        if val == field_val:
                            decode_row.append(f"OK")
                        else:
                            decode_row.append(f"ERROR")
                    else:
                        field_row.append("?")
                        decode_row.append(f"?")

                out.append("")
                out.append(tabulate([bitrow, field_row, value_row, decode_row], headers="firstrow",
                                    tablefmt="pipe", numalign="center", stralign="center"))
                out.append("")

                # Field info:
                out.append(f"*Bitfields*:")

                for field_name, field in reg.fields.items():
                    field_val = field.bits.extract_this_field_from(dump[adr])
                    out.append(f"   - {field_name}: 0x{field_val:X}")
                    out.extend(field.docs.two_line(prefix="  -"))

                    if field.enum is not None:
                        enum_entryname = field.lookup_enum_entry_name(field_val)
                        if enum_entryname is not None:
                            entry = field.enum.entries[enum_entryname]
                            out.append(f"       - *SELECTED*: {enum_entryname}")
                            out.extend(entry.docs.two_line(prefix="         - "))
                        else:
                            decode_row.append(
                                f"       - *DECODE ERROR*: This field accepts an enum, but it's value does not correspond to any enum entry.")
                    else:
                        decode_row.append(f"?")

            else:
                out.append(f"## 0x{adr:0X} - ?")
                out.append(f"   - 0x{dump[adr]:X}")
                out.append(f"   - 0b{dump[adr]:b}")

            out.append(f"")
            out.append(f"")

        output_file = os.path.join(cli.output_path, f"{rmap.map_name.lower()}_analysis.md")
        with open(output_file, 'w') as outfile:
            outfile.write("\n".join(out))
        print(f"Generated {output_file}...")
