from collections.abc import Callable
from dataclasses import dataclass
from typing import List

from typing_extensions import Self

from reginald.datamodel import RegisterMap


@dataclass
class CLI:
    input_file: str
    output_path: str
    generator: Callable[[RegisterMap, Self], None]
    generator_args: List[str]
