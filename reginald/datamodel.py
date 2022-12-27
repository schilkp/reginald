from typing import Dict, List, Optional, Union

import pydantic
import yaml
from pydantic import NonNegativeInt, PositiveInt, ValidationError
from pydantic.dataclasses import dataclass
from yaml.loader import SafeLoader

from reginald.bits import Bits
from reginald.error import ReginaldException


@dataclass
class RegEnumEntry:
    value: int
    doc: Optional[str] = None


@dataclass
class InputBitRange:
    lsb_position: NonNegativeInt
    width: PositiveInt


@dataclass
class Field:
    bits: Union[List[int], InputBitRange]
    access: Optional[str] = None
    doc: Optional[str] = None
    accepts_enum: Optional[str] = None
    enum: Optional[Dict[str, RegEnumEntry]] = None

    def __post_init_post_parse__(self):
        if isinstance(self.bits, InputBitRange):
            self._compiled_bits = Bits.from_range(self.bits.lsb_position, self.bits.width)
        else:
            self._compiled_bits = Bits.from_bitlist(self.bits)

    def get_bits(self) -> Bits:
        return self._compiled_bits


@dataclass
class Register:
    fields: Dict[str, Field] = pydantic.Field(default_factory=dict)
    adr: Optional[int] = None
    reset_val: Optional[int] = None
    doc: Optional[str] = None

    def fieldname_at(self, bit: int) -> Optional[str]:
        for name in self.fields:
            mask = self.fields[name].get_bits().get_bitmask()
            if (1 << bit) & mask != 0:
                return name
        return None

    def field_at(self, bit: int) -> Optional[Field]:
        name = self.fieldname_at(bit)
        if name is None:
            return None
        else:
            return self.fields[name]

    def get_unused_bits(self, register_bitwidth: int) -> Bits:
        mask = Bits.zero()

        for field in self.fields.values():
            mask = Bits.bitwise_or(mask, field.get_bits())

        return mask.bitwise_not(register_bitwidth)


@dataclass
class RegisterMap:
    device_name: str
    register_bitwidth: PositiveInt
    registers: Dict[str, Register]
    enums: Dict[str, Dict[str, RegEnumEntry]] = pydantic.Field(default_factory=dict)

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
