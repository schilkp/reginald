import argparse

from reginald.builtin_generators.c.funcpack import enums, reg_utils, regs
from reginald.builtin_generators.c.funcpack.name_generator import NameGenerator
from reginald.cli import CLI
from reginald.datamodel import *
from reginald.generator import OutputGenerator


class Generator(OutputGenerator):
    @classmethod
    def description(cls):
        return "TODO"

    @classmethod
    def generate(cls, rmap: RegisterMap, cli: CLI):

        # Options:
        # No comments on pack/unpack/modify macros (Default: Comments)
        # Field Enum: Prefix with register name (Default: Yes)

        parser = argparse.ArgumentParser(
            prog="c.funcpack",
            description="C Output generator, using functions for register management.")

        # parser.add_argument
        parser.add_argument('--field-enum-prefix', action=argparse.BooleanOptionalAction,
                            help="prefix a field enum with the register name", default=True)
        parser.add_argument('--short-packfunc-comment', action=argparse.BooleanOptionalAction,
                            help="Generate much shorter doxygen comments for all individual register function.", default=False)

        funcpack_options = parser.parse_args(cli.generator_args)

        name_gen = NameGenerator(rmap, funcpack_options)
        enums.generate(rmap, name_gen, cli, funcpack_options)
        regs.generate(rmap, name_gen, cli, funcpack_options)
        reg_utils.generate(rmap, name_gen, cli, funcpack_options)
