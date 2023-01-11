import re
from typing import Dict, List, Optional

from pydantic import NonNegativeInt, PositiveInt

from reginald.bits import Bits, fits_into_bitwidth
from reginald.datamodel import (AccessMode, AlwaysWrite, Docs, Field, RegEnum,
                                RegEnumEntry, Register, RegisterBlockTemplate,
                                RegisterMap, RegisterTemplate)
from reginald.error import ReginaldException
from reginald.input.parse_yaml import (YAML_Access, YAML_AlwaysWrite,
                                       YAML_Bits, YAML_Field,
                                       YAML_RegEnumEntry, YAML_Register,
                                       YAML_RegisterMap)


class YAMLConverter:
    def __init__(self, yaml: YAML_RegisterMap):
        self.yaml = yaml
        self.existing_adrs = {}  # Track existing register adrs, and what register they belong to

    def convert(self) -> RegisterMap:
        bt = f"{self.yaml.map_name}"
        self.rmap = RegisterMap(
            map_name=self.yaml.map_name,
            docs=self._convert_docs(self.yaml, bt),
            shared_enums={},
            register_block_templates={},
            registers={})

        # Order is critical:
        #  - register_block_templates require shared_enums to be converted
        #  - registers require both shared_enums and register_block_templates to be converted
        self._convert_shared_enums(bt)
        self._convert_register_block_templates(bt)
        self._convert_registers(bt)

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

    def _convert_shared_enums(self, bt_orig: str):
        result = {}
        for enum_name, enum in self.yaml.shared_enums.items():
            bt = bt_orig + f" -> shared enums -> {enum_name}"
            docs = self._convert_docs(enum, bt)
            entries = {}
            for entry_name, entry in enum.enum.items():
                entries[entry_name] = self._convert_enum_entry(entry_name, entry, bt)

            result[enum_name] = RegEnum(
                name=enum_name,
                is_shared=True,
                docs=docs,
                entries=entries)

        self.rmap.shared_enums = result

    def _convert_enum_entry(self, entry_name: str, entry: YAML_RegEnumEntry, bt: str) -> RegEnumEntry:
        bt = bt + f" -> {entry_name}"
        docs = self._convert_docs(entry, bt)
        return RegEnumEntry(name=entry_name, value=entry.val, docs=docs)

    def _convert_register_block_templates(self, bt_orig: str):
        block_templates = {}
        for block_name, block in self.yaml.register_block_templates.items():
            bt_block = bt_orig + f" -> register block templates -> {block_name}"
            block_docs = self._convert_docs(block, bt_block)

            offsets_seen = {}  # Remember which template was seen at a given offset

            register_templates = {}
            for template_name, template in block.registers.items():
                bt = bt_block + f" -> {template_name}"

                offset = template.offset
                if offset in offsets_seen:
                    raise ReginaldException(f"{bt}: Template has same offset as previous template {offsets_seen[offsets_seen]}!")
                offsets_seen[offset] = template_name

                bitwidth = self._convert_bitwidth(template.bitwidth, bt)
                docs = self._convert_docs(template, bt)
                reset_val = template.reset_val
                always_write = self._convert_always_write(template.always_write, bt)
                fields = self._convert_fields(template.fields, bt)

                register_templates[template_name] = RegisterTemplate(
                    name=template_name,
                    instances={},  # Instances populated during register & block instantiation conversion
                    fields=fields,
                    bitwidth=bitwidth,
                    offset=offset,
                    always_write=always_write,
                    reset_val=reset_val,
                    docs=docs)

            block_templates[block_name] = RegisterBlockTemplate(
                name=block_name,
                instances={},  # Instances populated during register & block instantiation conversion
                docs=block_docs,
                registers=register_templates)

        self.rmap.register_block_templates = block_templates

    def _convert_always_write(self, always_write: Optional[YAML_AlwaysWrite], bt: str) -> Optional[AlwaysWrite]:
        bt = bt + f" -> always_write"

        if always_write is None:
            return None
        bits = self._convert_bits(always_write.bits, bt, allow_zero=False)

        if isinstance(always_write.val, list):
            if not isinstance(always_write.bits, list):
                raise ReginaldException(f"{bt}: value may only be specified as list if bits are specified as list!")
            if len(always_write.bits) != len(always_write.val):
                raise ReginaldException(f"{bt}: Value and bits, when specified as lists, must be of equal length!")

            value = 0
            for bit_part, value_part in zip(always_write.bits, always_write.val):
                # Validate that specified value_part fits into bit_part:
                bit_part = self._convert_bits(bit_part,  "builtin", allow_zero=False)

                if not fits_into_bitwidth(value_part, bit_part.total_width()):
                    raise ReginaldException(f"{bt}: value component {value_part} does not fit into bit component {bit_part}!")

                value |= value_part << bit_part.lsb_position()
        else:
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
                    raise ReginaldException("{bt}: Invalid bits!")

                positions = bit.split("-")

                if len(positions) != 2:
                    raise ReginaldException("{bt}: Invalid bits!")

                pos_start = min([int(p) for p in positions])
                pos_stop = max([int(p) for p in positions])

                new_bits = list(range(pos_start, pos_stop+1))

            for new_bit in new_bits:
                if new_bit in bitlist:
                    raise ReginaldException("{bt}: Bits contains bit {new_bit} twice!")
            bitlist.extend(new_bits)

        if len(bitlist) == 0 and not allow_zero:
            raise ReginaldException("{bt}: Bits may not be zero")

        return Bits(bitlist=bitlist)

    def _convert_fields(self, fields: Dict[str, YAML_Field], bt_orig: str) -> Dict[str, Field]:
        result = {}

        for field_name, field in fields.items():
            bt = bt_orig + f" -> {field_name}"

            bits = self._convert_bits(field.bits, bt, allow_zero=False)
            access = self._convert_access(field.access, bt)
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
            if field.enum not in self.rmap.shared_enums:
                raise ReginaldException(f"{bt}: Register references shared enum that does not exists.")
            return self.rmap.shared_enums[field.enum]
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

    def _convert_registers(self, bt_orig: str):

        for name, r in self.yaml.registers.items():
            bt = bt_orig + f" -> registers -> {name}"
            if isinstance(r, YAML_Register):
                reg = r
                reg_name = name

                adr = reg.adr

                bitwidth = self._convert_bitwidth(reg.bitwidth, bt)
                docs = self._convert_docs(reg, bt)
                reset_val = reg.reset_val
                always_write = self._convert_always_write(reg.always_write, bt)
                fields = self._convert_fields(reg.fields, bt)

                self._add_register(Register(
                    name=reg_name,
                    fields=fields,
                    originates_from_template=None,
                    bitwidth=bitwidth,
                    adr=adr,
                    always_write=always_write,
                    reset_val=reset_val,
                    docs=docs), bt)
            else:
                inst = r
                inst_name = name

                if inst.register_block_template not in self.rmap.register_block_templates:
                    raise ReginaldException(f"{bt}: Block template {inst.register_block_template} unknown!")

                block_template = self.rmap.register_block_templates[inst.register_block_template]

                if isinstance(inst.start_adr, Dict):
                    # Multiple instantiations
                    for instance_id, start_adr in inst.start_adr.items():
                        if inst.replace_str_with_instance_id is None:
                            instance_name = inst_name+instance_id
                        else:
                            if inst_name.count(inst.replace_str_with_instance_id) == 0:
                                raise ReginaldException(f"{bt}: replace_str "
                                                        "\"{instantiation.replace_str_with_instance_id}\" "
                                                        "not found in \"{instantiation_name}\"!")

                            instance_name = inst_name.replace(inst.replace_str_with_instance_id, instance_id, 1)

                        self._add_block_template_instance(instance_name, start_adr, block_template, bt)

                else:
                    self._add_block_template_instance(inst_name, inst.start_adr, block_template, bt)

    def _add_register(self, reg: Register, bt: str):
        if reg.name in self.rmap.registers:
            raise ReginaldException(f"{bt}: Register with name {reg.name} already exists!")

        if reg.adr in self.existing_adrs:
            raise ReginaldException(f"{bt}: Another Register already exists at address {reg.adr}: {self.existing_adrs[reg.adr]}!")
        self.existing_adrs[reg.adr] = reg.name

        self.rmap.registers[reg.name] = reg

    def _add_block_template_instance(self, instance_name: str, start_adr: NonNegativeInt, template: RegisterBlockTemplate, bt: str):
        template.instances[start_adr] = instance_name
        for reg_templ in template.registers.values():
            reg_templ.instances[start_adr+reg_templ.offset] = instance_name+reg_templ.name
            self._add_register(Register(
                name=instance_name + reg_templ.name,
                fields=reg_templ.fields,
                originates_from_template=template,
                bitwidth=reg_templ.bitwidth,
                adr=start_adr + reg_templ.offset,
                always_write=reg_templ.always_write,
                reset_val=reg_templ.reset_val,
                docs=reg_templ.docs), bt)
