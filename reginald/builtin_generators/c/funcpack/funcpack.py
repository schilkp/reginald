import argparse

from reginald.builtin_generators.c.funcpack import enums, regs, reg_utils
from reginald.builtin_generators.c.funcpack.name_generator import NameGenerator
from reginald.datamodel import *
from reginald.generator import OutputGenerator

class GeneratorRegs(OutputGenerator):
    def description(self) -> str:
        return "C header with register structs."

    def generate(self, rmap: RegisterMap, input_file: str, output_file: str, args: List[str]):
        _ = input_file
        funcpack_options = parse_args(args)
        name_gen = NameGenerator(rmap, funcpack_options)
        regs.generate(rmap, name_gen,  output_file)

class GeneratorRegUtils(OutputGenerator):
    def description(self) -> str:
        return "C header with register packing/unpacking functions."

    def generate(self, rmap: RegisterMap, input_file: str, output_file: str, args: List[str]):
        _ = input_file
        funcpack_options = parse_args(args)
        name_gen = NameGenerator(rmap, funcpack_options)
        reg_utils.generate(rmap, name_gen,  output_file, funcpack_options)

class GeneratorEnums(OutputGenerator):
    def description(self) -> str:
        return "C header register field enums."

    def generate(self, rmap: RegisterMap, input_file: str, output_file: str, args: List[str]):
        _ = input_file
        funcpack_options = parse_args(args)
        name_gen = NameGenerator(rmap, funcpack_options)
        enums.generate(rmap, name_gen,  output_file)

def parse_args(args: List[str]):

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

    return parser.parse_args(args)
