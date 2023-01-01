from typing import Dict, List, Union

from pydantic.dataclasses import dataclass
from tabulate import tabulate

from reginald.datamodel import *
from reginald.generator import OutputGenerator
from reginald.utils import str_oneline


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
    def generate(cls, map: RegisterMap, args: List[str]):

        if len(args) != 1:
            raise ReginaldException("md_dumpanalysis requires a yaml binary dump as it's only argument")

        dump_yaml = YamlBinaryDump.from_yaml_file(args[0])
        dump = dump_yaml.flatten()
        adrs = sorted(dump.keys())

        out = []

        # Generate header:
        out.append(f"# {map.device_name} Register Dump Analysis")
        out.append(f"")
        out.append(f"")

        for adr in adrs:
            register_name = map.get_registername_at(adr)

            if register_name is not None:
                register = map.registers[register_name]
                out.append(f"## 0x{adr:0X} - {register_name}")
                out.append(f"  - 0x{dump[adr]:X}")
                out.append(f"  - 0b{dump[adr]:b}")

                # Collect all bitranges that make up this register - field or not:
                register_bitranges = []  # type: List[BitRange]
                for field in register.fields.values():
                    for range in field.get_bitranges():
                        register_bitranges.append(range)

                register_bitranges.extend(register.get_unused_bitranges(map.register_bitwidth))

                # Sort bitranges:
                register_bitranges = sorted(register_bitranges, key=lambda x: x.lsb_position, reverse=True)

                bitrow = ["Bits:"]
                field_row = ["Field:"]
                value_row = ["Value:"]
                decode_row = ["Decode:"]

                for bitrange in register_bitranges:
                    bitrow.append(str(bitrange))

                    value_row.append(f"0x{bitrange.extract_this_field_from(dump[adr]):X}")

                    # Retrieve field that coresponds to this range (if any):
                    field_name = register.get_fieldname_at(bitrange.lsb_position)

                    if field_name is not None:
                        field_row.append(field_name)

                        # Lookup if this value in this value coresponds to an enum:
                        field = register.fields[field_name]
                        field_value = field.get_bits().extract_this_field_from(dump[adr])
                        enum_entryname = field.lookup_enum_entryname(map, field_value)

                        if enum_entryname is not None:
                            decode_row.append(f"{enum_entryname} (0x{field_value:X})")
                        else:
                            decode_row.append(f"?")

                    else:
                        field_row.append("?")
                        decode_row.append(f"?")

                out.append("")
                out.append(tabulate([bitrow, field_row, value_row, decode_row], headers="firstrow",
                                    tablefmt="pipe", numalign="center", stralign="center"))
                out.append("")

                # Field info:
                out.append(f"*Bitfields*:")

                for field_name, field in register.fields.items():
                    value = field.get_bits().extract_this_field_from(dump[adr])
                    out.append(f"   - {field_name}: 0x{value:X}")
                    if field.brief is not None:
                        out.append(f"       - {str_oneline(field.brief)}")
                    if field.doc is not None:
                        out.append(f"       - {str_oneline(field.doc)}")

                    enum_entryname = field.lookup_enum_entryname(map, value)

                    if enum_entryname is not None:
                        _, entry = field.get_enum(map, value)

                        if entry.brief is not None:
                            out.append(f"       - *SELECTED*: {enum_entryname} ({entry.brief})")
                        else:
                            out.append(f"       - *SELECTED*: {enum_entryname}")

                        if entry.doc is not None:
                            out.append(f"       - {str_oneline(entry.doc)}")

            else:
                out.append(f"## 0x{adr:0X} - ?")
                out.append(f"   - 0x{dump[adr]:X}")
                out.append(f"   - 0b{dump[adr]:b}")

            out.append(f"")
            out.append(f"")

        return "\n".join(out)
