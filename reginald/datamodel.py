import functools
from copy import deepcopy
from enum import Enum
from typing import Dict, List, Optional, Self

from pydantic import BaseModel, NonNegativeInt, PositiveInt

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


class Docs(BaseModel):
    brief: Optional[str]
    doc: Optional[str]

    def as_multi_line(self, prefix: str) -> str:
        result = []
        if self.brief is not None:
            result.append(prefix + self.brief)
        if self.doc is not None:
            for line in self.doc.splitlines():
                result.append(prefix + line)
        return "\n".join(result).strip()

    def as_two_line(self, prefix: str) -> List[str]:
        result = []
        if self.brief is not None:
            result.append(prefix + self.brief)
        if self.doc is not None:
            result.append(prefix + str_oneline(self.doc))
        return result

    def empty(self) -> bool:
        return (self.brief is None) and (self.doc is None)


class RegEnumEntry(BaseModel):
    name: str
    value: NonNegativeInt
    docs: Docs


class RegEnum(BaseModel):
    name: str
    is_shared: bool
    docs: Docs
    entries: Dict[str, RegEnumEntry]


class AlwaysWrite(BaseModel):
    bits: Bits
    value: NonNegativeInt


class Field(BaseModel):
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


class Register(BaseModel):
    name: str
    fields: Dict[str, Field]
    bitwidth: PositiveInt
    is_block_template: bool
    adr: NonNegativeInt
    always_write: Optional[AlwaysWrite]
    reset_val: Optional[NonNegativeInt]
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

    def get_populated_template(self, instance_name: str, instance_adr: NonNegativeInt) -> Self:
        if not self.is_block_template:
            raise ValueError()

        adr = self.adr + instance_adr
        name = instance_name + self.name
        reg = deepcopy(self)
        reg.is_block_template = False
        reg.adr = adr
        reg.name = name
        return reg

    def get_local_enums(self) -> List[RegEnum]:
        return [field.enum for field in self.fields.values() if isinstance(field.enum, RegEnum) and not field.enum.is_shared]


class RegisterBlock(BaseModel):
    name: str
    instances: Dict[str, NonNegativeInt]
    docs: Docs
    register_templates: Dict[str, Register]


class RegisterMap(BaseModel):
    map_name: str
    docs: Docs
    register_blocks: Dict[str, RegisterBlock]
    enums: Dict[str, RegEnum]

    @functools.cached_property
    def physical_registers(self) -> Dict[NonNegativeInt, Register]:
        regs = {}
        for block in self.register_blocks.values():
            for template in block.register_templates.values():
                for instance_name, instance_adr in block.instances.items():
                    reg = template.get_populated_template(instance_name, instance_adr)
                    regs[reg.adr] = reg

        return regs

    @functools.cached_property
    def addresses(self) -> List[NonNegativeInt]:
        return list(self.physical_registers.keys())

    @functools.cached_property
    def max_address(self) -> NonNegativeInt:
        return max(self.addresses)

    @functools.cached_property
    def max_register_bitwidth(self) -> NonNegativeInt:
        return max([reg.bitwidth for block in self.register_blocks.values() for reg in block.register_templates.values()])
