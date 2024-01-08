from math import log2

from reginald.datamodel import *
from reginald.utils import c_fitting_unsigned_type, c_sanitize


class NameGenerator():
    def __init__(self, rmap: RegisterMap, funcpack_options):
        self.rmap = rmap
        self.opt = funcpack_options

    def filename_regs(self) -> str:
        mapname_c = c_sanitize(self.rmap.map_name).lower()
        return f"{mapname_c}_regs.h"

    def filename_enums(self) -> str:
        mapname_c = c_sanitize(self.rmap.map_name).lower()
        return f"{mapname_c}_enums.h"

    def filename_reg_utils(self) -> str:
        mapname_c = c_sanitize(self.rmap.map_name).lower()
        return f"{mapname_c}_reg_utils.h"

    def adr_type(self) -> str:
        adrs = []
        for block in self.rmap.register_blocks.values():
            for instance_start in block.instances.values():
                for reg_template in block.register_templates.values():
                    adrs.append(instance_start+reg_template.adr)

        max_adr = max(adrs)
        return c_fitting_unsigned_type(round(log2(max_adr)+0.5))

    def reg_packed_type(self, reg: Register) -> str:
        return c_fitting_unsigned_type(reg.bitwidth)

    def reg_maximum_packed_type(self) -> str:

        widths = []
        for block in self.rmap.register_blocks.values():
            for reg_template in block.register_templates.values():
                widths.append(reg_template.bitwidth)
        max_width = max(widths)
        return c_fitting_unsigned_type(max_width)

    def reg_resetval_macro(self, block_name: str, template_name: str) -> str:
        mapname_macro = c_sanitize(self.rmap.map_name).upper()
        blockname_macro = c_sanitize(block_name).upper()
        templatename_macro = c_sanitize(template_name).upper()
        return f"{mapname_macro}_REG_{blockname_macro}{templatename_macro}__RESETVAL"

    def reg_alwayswrite_mask_macro(self, block_name: str, template_name: str) -> str:
        mapname_macro = c_sanitize(self.rmap.map_name).upper()
        blockname_macro = c_sanitize(block_name).upper()
        templatename_macro = c_sanitize(template_name).upper()
        return f"{mapname_macro}_REG_{blockname_macro}{templatename_macro}__ALWAYSWRITE_MASK"

    def reg_alwayswrite_val_macro(self, block_name: str, template_name: str) -> str:
        mapname_macro = c_sanitize(self.rmap.map_name).upper()
        blockname_macro = c_sanitize(block_name).upper()
        templatename_macro = c_sanitize(template_name).upper()
        return f"{mapname_macro}_REG_{blockname_macro}{templatename_macro}__ALWAYSWRITE_VAL"

    def reg_struct_name(self, block_name: str, template_name: str) -> str:
        mapname_c = c_sanitize(self.rmap.map_name).lower()
        regname_c = c_sanitize(block_name).lower() + c_sanitize(template_name).lower()
        return f"{mapname_c}_reg_{regname_c}"

    def reg_struct_member(self, field: Field) -> str:
        fieldname_c = c_sanitize(field.name).lower()
        return fieldname_c

    def reg_struct_member_type(self, block_name: str, template_name: str, field: Field) -> str:
        if field.enum is None:
            return c_fitting_unsigned_type(field.bits.total_width())
        else:
            if field.enum.is_shared:
                return "enum "+self.enum_shared(field.enum)
            else:
                return "enum "+self.enum_field(block_name, template_name, field.enum)

    def reg_modify_func(self, block_name: str, template_name: str) -> str:
        mapname_c = c_sanitize(self.rmap.map_name).lower()
        regname_c = c_sanitize(block_name).lower() + c_sanitize(template_name).lower()
        return f"{mapname_c}_modify_{regname_c}"

    def reg_pack_func(self, block_name: str, template_name: str) -> str:
        mapname_c = c_sanitize(self.rmap.map_name).lower()
        regname_c = c_sanitize(block_name).lower() + c_sanitize(template_name).lower()
        return f"{mapname_c}_pack_{regname_c}"

    def reg_unpack_func(self, block_name: str, template_name: str) -> str:
        mapname_c = c_sanitize(self.rmap.map_name).lower()
        regname_c = c_sanitize(block_name).lower() + c_sanitize(template_name).lower()
        return f"{mapname_c}_unpack_{regname_c}"

    def reg_unpack_macro(self, block_name: str, template_name: str) -> str:
        mapname_macro = c_sanitize(self.rmap.map_name).upper()
        regname_macro = c_sanitize(block_name).upper() + c_sanitize(template_name).upper()
        return f"{mapname_macro}_UNPACK_{regname_macro}"

    def enum_shared(self, enum: RegEnum) -> str:
        mapname_c = c_sanitize(self.rmap.map_name).lower()
        enumname_c = c_sanitize(enum.name).lower()
        return f"{mapname_c}_{enumname_c}"

    def enum_shared_entry(self, enum: RegEnum, entry: RegEnumEntry) -> str:
        mapname_macro = c_sanitize(self.rmap.map_name).upper()
        enumname_macro = c_sanitize(enum.name).upper()
        entryname_macro = c_sanitize(entry.name).upper()
        return f"{mapname_macro}_{enumname_macro}_{entryname_macro}"

    def enum_field(self, block_name: str, template_name: str, enum: RegEnum) -> str:
        mapname_c = c_sanitize(self.rmap.map_name).lower()
        regname_c = c_sanitize(block_name).lower() + c_sanitize(template_name).lower()
        enumname_c = c_sanitize(enum.name).lower()
        if self.opt.field_enum_prefix:
            return f"{mapname_c}_{regname_c}_{enumname_c}"
        else:
            return f"{mapname_c}_{enumname_c}"

    def enum_field_entry(self, block_name: str, template_name: str, enum: RegEnum, entry: RegEnumEntry) -> str:
        mapname_macro = c_sanitize(self.rmap.map_name).upper()
        enumname_macro = c_sanitize(enum.name).upper()
        regname_macro = c_sanitize(block_name).upper() + c_sanitize(template_name).upper()
        entryname_macro = c_sanitize(entry.name).upper()
        if self.opt.field_enum_prefix:
            return f"{mapname_macro}_{regname_macro}_{enumname_macro}_{entryname_macro}"
        else:
            return f"{mapname_macro}_{enumname_macro}_{entryname_macro}"

    def include_guard_macro(self, filename: str) -> str:
        return c_sanitize(filename).upper() + "_"

    def generic_modify_macro(self) -> str:
        mapname_macro = c_sanitize(self.rmap.map_name).upper()
        return f"{mapname_macro}_MODIFY"

    def generic_pack_macro(self) -> str:
        mapname_macro = c_sanitize(self.rmap.map_name).upper()
        return f"{mapname_macro}_PACK"

    def generic_unpack_macro(self) -> str:
        mapname_macro = c_sanitize(self.rmap.map_name).upper()
        return f"{mapname_macro}_UNPACK"

    def lookup_resetval_func(self) -> str:
        mapname_c = c_sanitize(self.rmap.map_name).lower()
        return f"{mapname_c}_lookup_resetval"

    def doxygroup_genericfuncs(self) -> str:
        mapname_macro = c_sanitize(self.rmap.map_name).upper()
        return f"{mapname_macro}_GENERICFUNCS"

    def doxygroup_regfuncs(self, block_name: str, template_name: str) -> str:
        mapname_macro = c_sanitize(self.rmap.map_name).upper()
        regname_macro = c_sanitize(block_name).upper() + c_sanitize(template_name).upper()
        return f"{mapname_macro}_{regname_macro}_FUNCS"
