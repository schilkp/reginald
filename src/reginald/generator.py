from abc import ABC, abstractmethod
from typing import List

from reginald.datamodel import RegisterMap


class OutputGenerator(ABC):
    @abstractmethod
    def generate(self, rmap: RegisterMap, input_file: str, output_file: str, args: List[str]) -> List[str]:
        raise NotImplementedError

    @abstractmethod
    def description(self) -> str:
        raise NotImplementedError
