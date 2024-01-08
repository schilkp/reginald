

from reginald.bits import fits_into_bitwidth
from reginald.datamodel import *
from reginald.error import ReginaldException


class MapValidator:
    def __init__(self, rmap: RegisterMap):
        self.rmap = rmap

    def validate(self):
        # Validate all registers:
        for block in self.rmap.registers.values():
            for template in block.registers.values():
                self._validate_template(template)

    def _validate_template(self, reg: Register):
        bt = f"registers -> {reg.name}"

        # Validate all fields:
        for field in reg.fields.values():
            self._validate_field(reg, field, bt)

        # Validate that resetval fits into this registers:
        if reg.reset_val is not None:
            if not fits_into_bitwidth(reg.reset_val, reg.bitwidth):
                raise ReginaldException(f"{bt}: reset_val does not fit into register!")

        # Validate that no fields overlap:
        field_at_bit = {}
        for field in reg.fields.values():
            for bit in field.bits.bitlist:
                if bit in field_at_bit:
                    raise ReginaldException(f"Field {field.name} overalaps with field {field_at_bit[bit]} at bit {bit}!")
                field_at_bit[bit] = field.name

        if reg.always_write is not None:
            # Validate that always_write fits into register:
            if reg.always_write.bits.msb_position() + 1 > reg.bitwidth:
                raise ReginaldException(f"{bt}: always_write does not fit into register!")

            # Validate that always_write does not overlap with fields:
            for bit in reg.always_write.bits.bitlist:
                if bit in field_at_bit:
                    raise ReginaldException(f"{bt}: always_write overlaps with field {field_at_bit[bit]} at bit {bit}")

    def _validate_field(self, reg: Register, field: Field, bt: str):
        bt = bt + f" -> {field.name}"

        # Validate that the field fits into the register:
        if field.bits.msb_position() + 1 > reg.bitwidth:
            raise ReginaldException(f"{bt}: Field does not fit into register!")

        # Validate that each enum entry actually fits into field:
        if field.enum is not None:
            for enum_entry in field.enum.entries.values():
                mask = field.bits.get_unpositioned_bits().get_bitmask()
                if enum_entry.value & mask != enum_entry.value:
                    raise ReginaldException("{bt}: Enum does not fit into field!")
