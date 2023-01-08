
from reginald.builtin_generators.c.funcpack import enums, regs, reg_utils
from reginald.builtin_generators.c.funcpack.name_generator import NameGenerator
from reginald.cli import CLI
from reginald.datamodel import *
from reginald.generator import OutputGenerator


class Generator(OutputGenerator):
    @classmethod
    def description(cls):
        return "TODO"

    @classmethod
    def generate(cls, map: RegisterMap, cli: CLI):

        name_gen = NameGenerator(map)
        enums.generate(map, name_gen, cli)
        regs.generate(map, name_gen, cli)
        reg_utils.generate(map, name_gen, cli)
