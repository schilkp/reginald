from typing import Union

from reginald.datamodel import (Field, RegEnum, RegEnumEntry, Register,
                                RegisterMap, RegisterTemplate)
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

    def reg_packed_type(self, reg: Union[Register, RegisterTemplate]) -> str:
        return c_fitting_unsigned_type(reg.bitwidth)

    def reg_adr_macro(self, regname: str) -> str:
        mapname_macro = c_sanitize(self.rmap.map_name).upper()
        regname_macro = c_sanitize(regname).upper()
        return f"{mapname_macro}_reg_{regname_macro}"

    def reg_resetval_macro(self, regname: str) -> str:
        mapname_macro = c_sanitize(self.rmap.map_name).upper()
        regname_macro = c_sanitize(regname).upper()
        return f"{mapname_macro}_reg_{regname_macro}__RESETVAL"

    def reg_alwayswrite_mask_macro(self, regname: str) -> str:
        mapname_macro = c_sanitize(self.rmap.map_name).upper()
        regname_macro = c_sanitize(regname).upper()
        return f"{mapname_macro}_reg_{regname_macro}__ALWAYSWRITE_MASK"

    def reg_alwayswrite_val_macro(self, regname: str) -> str:
        mapname_macro = c_sanitize(self.rmap.map_name).upper()
        regname_macro = c_sanitize(regname).upper()
        return f"{mapname_macro}_reg_{regname_macro}__ALWAYSWRITE_VAL"

    def reg_struct_name(self, regname: str) -> str:
        mapname_c = c_sanitize(self.rmap.map_name).lower()
        regname_c = c_sanitize(regname).lower()
        return f"{mapname_c}_reg_{regname_c}"

    def reg_struct_member(self, field: Field) -> str:
        fieldname_c = c_sanitize(field.name).lower()
        return fieldname_c

    def reg_struct_member_type(self, regname: str, field: Field) -> str:
        if field.enum is None:
            return c_fitting_unsigned_type(field.bits.total_width())
        else:
            if field.enum.is_shared:
                return "enum "+self.enum_shared(field.enum)
            else:
                return "enum "+self.enum_field(regname, field.enum)

    def reg_modify_func(self, regname: str) -> str:
        mapname_c = c_sanitize(self.rmap.map_name).lower()
        regname_c = c_sanitize(regname).lower()
        return f"{mapname_c}_modify_{regname_c}"

    def reg_pack_func(self, regname: str) -> str:
        mapname_c = c_sanitize(self.rmap.map_name).lower()
        regname_c = c_sanitize(regname).lower()
        return f"{mapname_c}_pack_{regname_c}"

    def reg_unpack_func(self, regname: str) -> str:
        mapname_c = c_sanitize(self.rmap.map_name).lower()
        regname_c = c_sanitize(regname).lower()
        return f"{mapname_c}_unpack_{regname_c}"

    def reg_unpack_macro(self, regname: str) -> str:
        mapname_macro = c_sanitize(self.rmap.map_name).upper()
        regname_macro = c_sanitize(regname).upper()
        return f"{mapname_macro}_UNPACK_{regname_macro}"

    def block_offset_macro(self, regname: str) -> str:
        mapname_macro = c_sanitize(self.rmap.map_name).upper()
        regname_macro = c_sanitize(regname).upper()
        return f"{mapname_macro}_{regname_macro}__OFFSET"

    def block_instance_start_macro(self, blockname: str, instance_name: str) -> str:
        mapname_macro = c_sanitize(self.rmap.map_name).upper()
        blockname_macro = c_sanitize(blockname).upper()
        instance_name_macro = c_sanitize(instance_name).upper()
        return f"{mapname_macro}_{blockname_macro}_{instance_name_macro}__START"

    def enum_shared(self, enum: RegEnum) -> str:
        mapname_c = c_sanitize(self.rmap.map_name).lower()
        enumname_c = c_sanitize(enum.name).lower()
        return f"{mapname_c}_{enumname_c}"

    def enum_shared_entry(self, enum: RegEnum, entry: RegEnumEntry) -> str:
        mapname_macro = c_sanitize(self.rmap.map_name).upper()
        enumname_macro = c_sanitize(enum.name).upper()
        entryname_macro = c_sanitize(entry.name).upper()
        return f"{mapname_macro}_{enumname_macro}_{entryname_macro}"

    def enum_field(self, reg_name: str, enum: RegEnum) -> str:
        mapname_c = c_sanitize(self.rmap.map_name).lower()
        regname_c = c_sanitize(reg_name).lower()
        enumname_c = c_sanitize(enum.name).lower()
        if self.opt.field_enum_prefix:
            return f"{mapname_c}_{regname_c}_{enumname_c}"
        else:
            return f"{mapname_c}_{enumname_c}"

    def enum_field_entry(self, reg_name: str, enum: RegEnum, entry: RegEnumEntry) -> str:
        mapname_macro = c_sanitize(self.rmap.map_name).upper()
        enumname_macro = c_sanitize(enum.name).upper()
        regname_macro = c_sanitize(reg_name).upper()
        entryname_macro = c_sanitize(entry.name).upper()
        if self.opt.field_enum_prefix:
            return f"{mapname_macro}_{regname_macro}_{enumname_macro}_{entryname_macro}"
        else:
            return f"{mapname_macro}_{enumname_macro}_{entryname_macro}"

    def include_guard_macro(self, filename: str) -> str:
        return c_sanitize(filename).upper() + "_"

    def generic_modify_macro(self) -> str:
        mapname_macro = c_sanitize(self.rmap.map_name).upper()
        return f"{mapname_macro}_REG_MODIFY"

    def generic_pack_macro(self) -> str:
        mapname_macro = c_sanitize(self.rmap.map_name).upper()
        return f"{mapname_macro}_REG_PACK"

    def generic_unpack_macro(self) -> str:
        mapname_macro = c_sanitize(self.rmap.map_name).upper()
        return f"{mapname_macro}_REG_UNPACK"

    def doxygroup_genericfuncs(self) -> str:
        mapname_macro = c_sanitize(self.rmap.map_name).upper()
        return f"{mapname_macro}_GENERICFUNCS"

    def doxygroup_regfuncs(self, regname: str) -> str:
        mapname_macro = c_sanitize(self.rmap.map_name).upper()
        regname_macro = c_sanitize(regname).upper()
        return f"{mapname_macro}_{regname_macro}_FUNCS"
