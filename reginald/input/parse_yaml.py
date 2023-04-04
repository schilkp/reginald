from typing import Dict, List, Optional, Union

import pydantic
import yaml
from pydantic import ConfigDict, NonNegativeInt, PositiveInt, ValidationError
from pydantic.config import Extra
from pydantic.dataclasses import dataclass
from yaml.loader import SafeLoader

from reginald.error import ReginaldException

YAML_Bits = Union[List[Union[NonNegativeInt, str]], NonNegativeInt, str]
YAML_Access = Union[List[str], str]


@dataclass(config=ConfigDict(anystr_strip_whitespace=True, extra=Extra.forbid))
class YAML_RegEnumEntry:
    val: NonNegativeInt
    doc: Optional[str] = None
    brief: Optional[str] = None


@dataclass(config=ConfigDict(anystr_strip_whitespace=True, extra=Extra.forbid))
class YAML_SharedEnum:
    enum: Dict[str, YAML_RegEnumEntry]
    doc: Optional[str] = None
    brief: Optional[str] = None


@dataclass(config=ConfigDict(anystr_strip_whitespace=True, extra=Extra.forbid))
class YAML_Field:
    bits: YAML_Bits
    access: Optional[YAML_Access] = None
    doc: Optional[str] = None
    brief: Optional[str] = None
    reset_val: Optional[NonNegativeInt] = None
    enum: Optional[Union[Dict[str, YAML_RegEnumEntry], str]] = None


@dataclass(config=ConfigDict(anystr_strip_whitespace=True, extra=Extra.forbid))
class YAML_AlwaysWrite:
    bits: YAML_Bits
    val: Union[NonNegativeInt, List[NonNegativeInt]]


@dataclass(config=ConfigDict(anystr_strip_whitespace=True, extra=Extra.forbid))
class YAML_BlockInstantiation:
    start_adr: Union[NonNegativeInt, Dict[str, NonNegativeInt]]
    register_block_template: str
    replace_str_with_instance_id: Optional[str]


@dataclass(config=ConfigDict(anystr_strip_whitespace=True, extra=Extra.forbid))
class YAML_RegisterTemplate:
    offset: NonNegativeInt
    fields: Dict[str, YAML_Field] = pydantic.Field(default_factory=dict)
    bitwidth: Optional[PositiveInt] = None
    reset_val: Optional[NonNegativeInt] = None
    always_write: Optional[YAML_AlwaysWrite] = None
    doc: Optional[str] = None
    brief: Optional[str] = None


@dataclass(config=ConfigDict(anystr_strip_whitespace=True, extra=Extra.forbid))
class YAML_RegisterBlockTemplate:
    registers: Dict[str, YAML_RegisterTemplate]
    doc: Optional[str] = None
    brief: Optional[str] = None


@dataclass(config=ConfigDict(anystr_strip_whitespace=True, extra=Extra.forbid))
class YAML_Register:
    fields: Dict[str, YAML_Field] = pydantic.Field(default_factory=dict)
    adr: Optional[NonNegativeInt] = None
    bitwidth: Optional[PositiveInt] = None
    reset_val: Optional[NonNegativeInt] = None
    always_write: Optional[YAML_AlwaysWrite] = None
    doc: Optional[str] = None
    brief: Optional[str] = None


@dataclass(config=ConfigDict(anystr_strip_whitespace=True, extra=Extra.forbid))
class YAML_RegisterMap:
    map_name: str
    default_register_bitwidth: PositiveInt
    registers: Dict[str, Union[YAML_Register, YAML_BlockInstantiation]]
    shared_enums: Dict[str, YAML_SharedEnum] = pydantic.Field(default_factory=dict)
    register_block_templates: Dict[str, YAML_RegisterBlockTemplate] = pydantic.Field(default_factory=dict)
    doc: Optional[str] = None
    brief: Optional[str] = None

    @classmethod
    def from_yaml_file(cls, file_name: str):
        try:
            with open(file_name) as f:
                data = yaml.load(f, Loader=SafeLoader)
                return YAML_RegisterMap(**data)
        except FileNotFoundError:
            raise ReginaldException(f"File {file_name} not found")
        except ValidationError as e:
            raise ReginaldException(str(e))
