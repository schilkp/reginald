from typing import Dict, List, Optional, Tuple, Union

import pydantic
import yaml
from pydantic import NonNegativeInt, PositiveInt, ValidationError
from pydantic.dataclasses import dataclass
from yaml.loader import SafeLoader

from reginald.bits import BitRange, Bits
from reginald.error import ReginaldException


@dataclass
class RegEnumEntry:
    value: NonNegativeInt
    doc: Optional[str] = None
    brief: Optional[str] = None

    def __post_init_post_parse__(self):
        if self.brief is not None:
            self.brief = " ".join(self.brief.splitlines()).strip()


@dataclass
class InputBitRange:
    lsb_position: NonNegativeInt
    width: PositiveInt


@dataclass
class Field:
    bits: Union[List[int], InputBitRange]
    access: Optional[str] = None
    doc: Optional[str] = None
    brief: Optional[str] = None
    accepts_enum: Optional[str] = None
    enum: Optional[Dict[str, RegEnumEntry]] = None

    def __post_init_post_parse__(self):
        if self.brief is not None:
            self.brief = " ".join(self.brief.splitlines()).strip()

        if isinstance(self.bits, InputBitRange):
            self._compiled_bits = Bits.from_position(self.bits.lsb_position, self.bits.width)
        else:
            self._compiled_bits = Bits.from_bitlist(self.bits)

    def get_bits(self) -> Bits:
        return self._compiled_bits

    def get_bitrange(self) -> BitRange:
        return self._compiled_bits.get_bitrange()

    def get_bitranges(self) -> List[BitRange]:
        return self._compiled_bits.get_bitranges()

    def lookup_enum_entryname(self, map, value: NonNegativeInt) -> Optional[str]:
        if self.enum is not None:
            for entry_name, entry in self.enum.items():
                if entry.value == value:
                    return entry_name
        if self.accepts_enum is not None:
            enum = map.enums[self.accepts_enum]
            for entry_name, entry in enum.items():
                if entry.value == value:
                    return entry_name

    def get_enum(self, map, value: NonNegativeInt) -> Tuple[str, RegEnumEntry]:
        if self.enum is not None:
            for entry_name, entry in self.enum.items():
                if entry.value == value:
                    return entry_name, entry
        if self.accepts_enum is not None:
            enum = map.enums[self.accepts_enum]
            for entry_name, entry in enum.items():
                if entry.value == value:
                    return entry_name, entry

        raise KeyError()


@dataclass
class Register:
    fields: Dict[str, Field] = pydantic.Field(default_factory=dict)
    adr: Optional[int] = None
    reset_val: Optional[int] = None
    doc: Optional[str] = None
    brief: Optional[str] = None

    def __post_init_post_parse__(self):
        if self.brief is not None:
            self.brief = " ".join(self.brief.splitlines()).strip()

    def get_fieldname_at(self, bit: int) -> Optional[str]:
        for name in self.fields:
            mask = self.fields[name].get_bits().get_bitmask()
            if (1 << bit) & mask != 0:
                return name
        return None

    def get_field_at(self, bit: int) -> Optional[Field]:
        name = self.get_fieldname_at(bit)
        if name is None:
            return None
        else:
            return self.fields[name]

    def get_unused_bits(self, register_bitwidth: int) -> Bits:
        mask = Bits.zero()

        for field in self.fields.values():
            mask = Bits.bitwise_or(mask, field.get_bits())

        return mask.bitwise_not(register_bitwidth)

    def get_unused_bitranges(self, register_bitwidth: int) -> List[BitRange]:
        return self.get_unused_bits(register_bitwidth).get_bitranges()


@dataclass
class RegisterMap:
    device_name: str
    register_bitwidth: PositiveInt
    registers: Dict[str, Register]
    enums: Dict[str, Dict[str, RegEnumEntry]] = pydantic.Field(default_factory=dict)
    doc: Optional[str] = None
    brief: Optional[str] = None

    def __post_init_post_parse__(self):
        if self.brief is not None:
            self.brief = " ".join(self.brief.splitlines()).strip()

    def get_registername_at(self, adr: NonNegativeInt) -> Optional[str]:
        for reg_name, reg in self.registers.items():
            if reg.adr == adr:
                return reg_name

        return None

    def get_register_at(self, adr: NonNegativeInt) -> Optional[Register]:
        reg_name = self.get_registername_at(adr)
        if reg_name is not None:
            return self.registers[reg_name]
        else:
            return None

    @classmethod
    def from_yaml_file(cls, file_name: str):
        try:
            with open(file_name) as f:
                data = yaml.load(f, Loader=SafeLoader)
                return RegisterMap(**data)
        except FileNotFoundError:
            raise ReginaldException(f"File {file_name} not found")
        except ValidationError as e:
            raise ReginaldException(str(e))
