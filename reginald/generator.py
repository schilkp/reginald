from abc import ABC, abstractmethod
from typing import List

from reginald.datamodel import RegisterMap


class OutputGenerator(ABC):
    @abstractmethod
    def generate(cls, rmap: RegisterMap, input_file: str, output_file: str, args: List[str]):
        raise NotImplementedError

    @abstractmethod
    def description(cls) -> str:
        raise NotImplementedError
