from typing import Dict, List, Optional, Union

import pydantic
import yaml
from pydantic import (BaseModel, ConfigDict, NonNegativeInt, PositiveInt,
                      ValidationError)
from yaml.loader import SafeLoader

from reginald.error import ReginaldException

YAML_Bits = Union[List[Union[NonNegativeInt, str]], NonNegativeInt, str]
YAML_Access = Union[List[str], str]


class YAML_RegEnumEntry(BaseModel):
    model_config = ConfigDict(extra='forbid', strict=True)

    val: NonNegativeInt
    doc: Optional[str] = None
    brief: Optional[str] = None


class YAML_Enum(BaseModel):
    model_config = ConfigDict(extra='forbid', strict=True)

    enum: Dict[str, YAML_RegEnumEntry]
    doc: Optional[str] = None
    brief: Optional[str] = None


class YAML_Field(BaseModel):
    model_config = ConfigDict(extra='forbid', strict=True)

    bits: YAML_Bits
    access: Optional[YAML_Access] = None
    doc: Optional[str] = None
    brief: Optional[str] = None
    enum: Optional[Union[Dict[str, YAML_RegEnumEntry], str]] = None


class YAML_AlwaysWrite(BaseModel):
    model_config = ConfigDict(extra='forbid', strict=True)

    mask: NonNegativeInt
    val: NonNegativeInt


class YAML_Register(BaseModel):
    model_config = ConfigDict(extra='forbid', strict=True)

    fields: Dict[str, YAML_Field] = pydantic.Field(default_factory=dict)
    access: Optional[YAML_Access] = None
    adr: NonNegativeInt
    bitwidth: Optional[PositiveInt] = None
    reset_val: Optional[NonNegativeInt] = None
    always_write: Optional[YAML_AlwaysWrite] = None
    doc: Optional[str] = None
    brief: Optional[str] = None


class YAML_RegisterBlock(BaseModel):
    model_config = ConfigDict(extra='forbid', strict=True)

    instances: Dict[str, NonNegativeInt]
    brief: Optional[str] = None
    doc: Optional[str] = None
    registers: Dict[str, YAML_Register]


class YAML_RegisterMap(BaseModel):
    model_config = ConfigDict(extra='forbid', strict=True)

    map_name: str
    default_register_bitwidth: PositiveInt
    registers: Dict[str, Union[YAML_Register, YAML_RegisterBlock]]
    enums: Dict[str, YAML_Enum] = pydantic.Field(default_factory=dict)
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
