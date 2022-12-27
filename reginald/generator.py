from abc import ABC, abstractclassmethod
from typing import List

from reginald.datamodel import RegisterMap


class OutputGenerator(ABC):
    @abstractclassmethod
    def generate(cls, map: RegisterMap, args: List[str]):
        raise NotImplementedError

    @abstractclassmethod
    def description(cls) -> str:
        raise NotImplementedError
