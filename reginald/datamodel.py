from enum import Enum
from typing import Dict, List, Optional

from pydantic import NonNegativeInt, PositiveInt
from pydantic.dataclasses import dataclass

from reginald.bits import BitRange, Bits
from reginald.utils import str_oneline


class AccessMode(Enum):
    READ = 0
    WRITE = 1

    def to_str(self) -> str:
        match self:
            case AccessMode.READ:
                return "r"
            case AccessMode.WRITE:
                return "w"
            case _:
                raise ValueError()


@dataclass
class Docs:
    brief: Optional[str]
    doc: Optional[str]

    def multi_line(self, prefix: str) -> List[str]:
        result = []
        if self.brief is not None:
            result.append(prefix + self.brief)
        if self.doc is not None:
            for line in self.doc.splitlines():
                result.append(prefix + line)
        return result

    def two_line(self, prefix: str) -> List[str]:
        result = []
        if self.brief is not None:
            result.append(prefix + self.brief)
        if self.doc is not None:
            result.append(prefix + str_oneline(self.doc))
        return result


@dataclass
class RegEnumEntry:
    name: str
    value: NonNegativeInt
    docs: Docs


@dataclass
class RegEnum:
    name: str
    is_shared: bool
    docs: Docs
    entries: Dict[str, RegEnumEntry]


@dataclass
class AlwaysWrite:
    bits: Bits
    value: NonNegativeInt


@dataclass
class Field:
    name: str
    bits: Bits
    access: List[AccessMode]
    docs: Docs
    enum: Optional[RegEnum] = None

    def get_bitrange(self) -> BitRange:
        return self.bits.get_bitrange()

    def get_bitranges(self) -> List[BitRange]:
        return self.bits.get_bitranges()

    def access_str(self) -> str:
        modes = [mode.to_str() for mode in self.access]
        return "/".join(modes)

    def lookup_enum_entry_name(self, val: NonNegativeInt) -> Optional[str]:
        if self.enum is None:
            return None

        for entry in self.enum.entries.values():
            if entry.value == val:
                return entry.name

        return None


@dataclass
class RegisterTemplate:
    name: str
    instances: Dict[NonNegativeInt, str]
    register_block_name: str
    fields: Dict[str, Field]
    bitwidth: PositiveInt
    offset: NonNegativeInt
    always_write: Optional[AlwaysWrite]
    reset_val: Optional[int]
    docs: Docs

    def get_unused_bits(self, include_always_write: bool) -> Bits:

        bits = list(range(self.bitwidth))

        for field in self.fields.values():
            for bit in field.bits.bitlist:
                bits.remove(bit)

        if not include_always_write:
            if self.always_write is not None:
                for bit in self.always_write.bits.bitlist:
                    bits.remove(bit)

        return Bits(bitlist=bits)


@dataclass
class RegisterBlockTemplate:
    name: str
    instances: Dict[NonNegativeInt, str]
    docs: Docs
    registers: Dict[str, RegisterTemplate]


@dataclass
class Register:
    name: str
    fields: Dict[str, Field]
    bitwidth: PositiveInt
    docs: Docs
    adr: Optional[NonNegativeInt]
    always_write: Optional[AlwaysWrite]
    originates_from_template: Optional[RegisterBlockTemplate]
    reset_val: Optional[int]

    def get_unused_bits(self, include_always_write: bool) -> Bits:

        bits = list(range(self.bitwidth))

        for field in self.fields.values():
            for bit in field.bits.bitlist:
                bits.remove(bit)

        if not include_always_write:
            if self.always_write is not None:
                for bit in self.always_write.bits.bitlist:
                    bits.remove(bit)

        return Bits(bitlist=bits)

    def get_fieldname_at(self, bit: NonNegativeInt) -> Optional[str]:
        for field in self.fields.values():
            if bit in field.bits.bitlist:
                return field.name
        return None

    def is_bit_always_write(self, bit: NonNegativeInt) -> bool:
        if self.always_write is not None:
            if bit in self.always_write.bits.bitlist:
                return True
        return False

    def get_always_write_value(self, bits: Bits) -> NonNegativeInt:
        if self.always_write is None:
            raise ValueError()
        for bit in bits.bitlist:
            if not bit in self.always_write.bits.bitlist:
                raise ValueError()

        return (self.always_write.value & bits.get_bitmask()) >> bits.lsb_position()


@dataclass
class RegisterMap:
    map_name: str
    docs: Docs
    registers: Dict[str, Register]
    shared_enums: Dict[str, RegEnum]
    register_block_templates: Dict[str, RegisterBlockTemplate]

    def get_registername_at(self, adr: NonNegativeInt) -> Optional[str]:
        for reg in self.registers.values():
            if reg.adr is not None and reg.adr == adr:
                return reg.name
        return None
