from reginald.bits import fits_into_bitwidth
from reginald.datamodel import *
from reginald.error import ReginaldException


def validate_Field(self: Field, map: RegisterMap, backtrace: str):

    if self.get_bits().msb_position() + 1 > map.register_bitwidth:
        raise ReginaldException(
            f"{backtrace}: Field does not fit into register.")

    # Validate that a specified local enum will fit into this field:
    mask = self.get_bits().get_unpositioned_bits().get_bitmask()
    if self.enum is not None:
        for key, entry in self.enum.items():
            if entry.value & (~mask) != 0:
                raise ReginaldException(f"{backtrace} -> {key}: Enum does not fit into field.")

    # Validate that a specified external enum exists and will fit into this field:
    if self.accepts_enum is not None:
        if not self.accepts_enum in map.enums:
            raise ReginaldException(f"{backtrace}: Fields accepts enum {self.accepts_enum} that does not exist.")

        enum = map.enums[self.accepts_enum]

        for key, entry in enum.items():
            if entry.value & (~mask) != 0:
                raise ReginaldException(
                    f"{backtrace} -> {self.accepts_enum} -> {key}: Enum does not fit into field.")


def validate_Register(self: Register, map: RegisterMap, backtrace: str):
    # Validate all fields
    for name, field in self.fields.items():
        validate_Field(field, map, backtrace + f"-> {name}")

    # Validate that reset_val will fit into register_bitwidth:
    if self.reset_val is not None:
        if not fits_into_bitwidth(self.reset_val, map.register_bitwidth):
            raise ReginaldException(
                f"{backtrace}: reset_val does not fit into register.")

    # Validate that there are no field overlaps:
    previous_field_masks = {}
    for fieldname, field in self.fields.items():
        field_mask = field.get_bits().get_bitmask()

        for other_field, other_mask in previous_field_masks.items():
            if other_mask & field_mask != 0:
                raise ReginaldException(
                    f"{backtrace} -> {fieldname}: Field overlaps with field {other_field}! (Overlap: {hex(other_mask & field_mask)})")

        previous_field_masks[fieldname] = field_mask


def validate_RegisterMap(self: RegisterMap):
    for name, register in self.registers.items():
        validate_Register(register, self, name)

    adrs_present = []
    for name, register in self.registers.items():
        if register.adr is not None and register.adr in adrs_present:
            raise ReginaldException(f"Register {name}'s address 0x{register.adr:X} already exists!")
        adrs_present.append(register.adr)
