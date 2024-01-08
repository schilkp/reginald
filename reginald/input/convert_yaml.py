import re
from typing import Dict, List, Optional

from pydantic import NonNegativeInt, PositiveInt

from reginald.bits import Bits, fits_into_bitwidth
from reginald.datamodel import (AccessMode, AlwaysWrite, Docs, Field, RegEnum,
                                RegEnumEntry, Register, RegisterBlock,
                                RegisterMap)
from reginald.error import ReginaldException
from reginald.input.parse_yaml import (YAML_Access, YAML_AlwaysWrite,
                                       YAML_Bits, YAML_Field,
                                       YAML_RegEnumEntry, YAML_Register,
                                       YAML_RegisterBlock, YAML_RegisterMap)


class YAMLConverter:
    def __init__(self, yaml: YAML_RegisterMap):
        self.yaml = yaml

    def convert(self) -> RegisterMap:
        bt = f"{self.yaml.map_name}"
        self.rmap = RegisterMap(
            map_name=self.yaml.map_name,
            docs=self._convert_docs(self.yaml, bt),
            enums={},
            register_blocks={})

        # Order is critical: register conversion requires enums to be converted.
        self.rmap.enums = self._convert_enums(bt)
        self.rmap.register_blocks = self._convert_registers(bt)

        return self.rmap

    def _convert_docs(self, thing, bt: str) -> Docs:
        if thing.brief is not None:
            if len(thing.brief.strip()) == 0:
                raise ReginaldException(f"{bt} -> brief: brief does not contain text!")
            if len(thing.brief.splitlines()) > 1:
                raise ReginaldException(f"{bt} -> brief: brief may not contain more than one line!")

        if thing.doc is not None:
            if len(thing.doc.strip()) == 0:
                raise ReginaldException(f"{bt} -> doc: doc does not contain text!")

        return Docs(brief=thing.brief, doc=thing.doc)

    def _convert_bitwidth(self, bitwidth: Optional[PositiveInt], bt: str) -> PositiveInt:
        bt = bt + f" -> bitwidth"
        if bitwidth is not None:
            return bitwidth

        if self.yaml.default_register_bitwidth is not None:
            return self.yaml.default_register_bitwidth

        raise ReginaldException(f"{bt}: Register does not specify a bitwidth, and not default bitwidth is set")

    def _convert_enums(self, bt_orig: str) -> Dict[str, RegEnum]:
        result = {}
        for enum_name, enum in self.yaml.enums.items():
            bt = bt_orig + f" -> enums -> {enum_name}"

            docs = self._convert_docs(enum, bt)
            entries = {}
            for entry_name, entry in enum.enum.items():
                entries[entry_name] = self._convert_enum_entry(entry_name, entry, bt)

            result[enum_name] = RegEnum(
                name=enum_name,
                is_shared=True,
                docs=docs,
                entries=entries)

        return result

    def _convert_enum_entry(self, entry_name: str, entry: YAML_RegEnumEntry, bt: str) -> RegEnumEntry:
        bt = bt + f" -> {entry_name}"
        docs = self._convert_docs(entry, bt)
        return RegEnumEntry(name=entry_name, value=entry.val, docs=docs)

    def _convert_always_write(self, always_write: Optional[YAML_AlwaysWrite], bt: str) -> Optional[AlwaysWrite]:
        bt = bt + f" -> always_write"

        if always_write is None:
            return None

        bits = Bits.from_mask(always_write.mask)
        value = always_write.val

        return AlwaysWrite(bits=bits, value=value)

    def _convert_bits(self, bits: YAML_Bits, bt: str, allow_zero: bool) -> Bits:
        bt = bt + f" -> bits"
        bitlist = []

        if not isinstance(bits, list):
            bits = [bits]

        for bit in bits:
            if isinstance(bit, int):
                # single bit
                new_bits = [bit]
            else:
                # string: a range
                if not re.match(r"^[0-9]+-[0-9]+$", bit):
                    raise ReginaldException(f"{bt}: Invalid bits!")

                positions = bit.split("-")

                if len(positions) != 2:
                    raise ReginaldException(f"{bt}: Invalid bits!")

                pos_start = min([int(p) for p in positions])
                pos_stop = max([int(p) for p in positions])

                new_bits = list(range(pos_start, pos_stop+1))

            for new_bit in new_bits:
                if new_bit in bitlist:
                    raise ReginaldException(f"{bt}: Bits contains bit {new_bit} twice!")
            bitlist.extend(new_bits)

        if len(bitlist) == 0 and not allow_zero:
            raise ReginaldException(f"{bt}: Bits may not be zero")

        return Bits(bitlist=bitlist)

    def _convert_fields(self, fields: Dict[str, YAML_Field], bt_orig: str, default_access: List[AccessMode]) -> Dict[str, Field]:
        result = {}

        for field_name, field in fields.items():
            bt = bt_orig + f" -> {field_name}"

            bits = self._convert_bits(field.bits, bt, allow_zero=False)
            access = self._convert_access(field.access, bt)
            if len(access) == 0:
                access = default_access
            docs = self._convert_docs(field, bt)
            enum = self._convert_field_enum(field_name, field, bt)

            result[field_name] = Field(
                name=field_name,
                bits=bits,
                docs=docs,
                access=access,
                enum=enum)

        return result

    def _convert_field_enum(self, field_name: str, field: YAML_Field, bt: str) -> Optional[RegEnum]:
        bt = bt + " -> enum"
        if field.enum is None:
            return None

        if isinstance(field.enum, str):
            # References shared enun
            if field.enum not in self.rmap.enums:
                raise ReginaldException(f"{bt}: Register references shared enum that does not exists.")
            return self.rmap.enums[field.enum]
        else:
            # Inline enum
            enum_docs = self._convert_docs(field, bt)
            entries = {}
            for entry_name, entry in field.enum.items():
                entries[entry_name] = self._convert_enum_entry(entry_name, entry, bt)

            return RegEnum(name=field_name, docs=enum_docs, is_shared=False, entries=entries)

    def _convert_access(self, access: Optional[YAML_Access], bt: str) -> List[AccessMode]:
        bt = bt + " -> access"

        if access is None:
            return []

        if isinstance(access, str):
            access = [access]

        result = []

        for access_mode in access:
            match access_mode.lower():
                case "r":
                    result.append(AccessMode.READ)
                case "w":
                    result.append(AccessMode.WRITE)
                case _:
                    raise ReginaldException(f"{bt}: Unknown access mode {access_mode}.")

        return result

    def _convert_registers(self, bt: str) -> Dict[str, RegisterBlock]:
        result = {}
        bt = bt + f" -> registers"
        for name, r in self.yaml.registers.items():
            if isinstance(r, YAML_Register):
                result[name] = self._convert_register(name, r, bt)
            else:
                result[name] = self._convert_register_block(name, r, bt)
        return result

    def _convert_register(self,  name: str, r: YAML_Register, bt: str) -> RegisterBlock:
        bt = bt + f" -> name"
        adr = r.adr
        bitwidth = self._convert_bitwidth(r.bitwidth, bt)
        docs = self._convert_docs(r, bt)
        reset_val = r.reset_val
        always_write = self._convert_always_write(r.always_write, bt)
        access = self._convert_access(r.access, bt)
        fields = self._convert_fields(r.fields, bt, access)

        return RegisterBlock(
            name=name,
            docs=docs,
            instances={name: adr},
            register_templates={"": Register(
                name="", fields=fields, bitwidth=bitwidth, adr=0, always_write=always_write, reset_val=reset_val, docs=docs, is_block_template=True
            )}
        )

    def _convert_register_block(self, name: str, b: YAML_RegisterBlock, bt_orig: str) -> RegisterBlock:
        bt_orig = bt_orig + f" -> {name}"
        docs = self._convert_docs(b, bt_orig)

        registers = {}

        for reg_name, r in b.registers.items():
            bt = bt_orig + f"-> {reg_name}"
            adr = r.adr
            bitwidth = self._convert_bitwidth(r.bitwidth, bt)
            docs = self._convert_docs(r, bt)
            reset_val = r.reset_val
            always_write = self._convert_always_write(r.always_write, bt)
            access = self._convert_access(r.access, bt)
            fields = self._convert_fields(r.fields, bt, access)

            registers[reg_name] = Register(
                name=reg_name,
                fields=fields,
                bitwidth=bitwidth,
                is_block_template=True,
                adr=adr,
                always_write=always_write,
                reset_val=reset_val, docs=docs,
            )

        return RegisterBlock(
            name=name,
            docs=docs,
            instances=b.instances,
            register_templates=registers
        )
