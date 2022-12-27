import argparse
import importlib.util
import sys
from dataclasses import dataclass
from pprint import pprint
from typing import List

import reginald.builtin_generators.c_macromap
import reginald.builtin_generators.markdown_doc
from reginald.error import ReginaldException
from reginald.generator import OutputGenerator


@dataclass
class CLI:
    input_file: str
    generator: OutputGenerator
    generator_args: List[str]


builtin_generators = {
    'c_macromap': reginald.builtin_generators.c_macromap.Generator,
    'markdown_doc': reginald.builtin_generators.markdown_doc.Generator
}


def parse_args():

    builtin_choices_text = []
    for name, generator in builtin_generators.items():
        builtin_choices_text.append(f"   {name}: {generator.description()}")

    builtin_choices_text = "\n".join(builtin_choices_text)

    parser = argparse.ArgumentParser(description='Test argparse', epilog="builtin generators: \n" +
                                     builtin_choices_text, formatter_class=argparse.RawDescriptionHelpFormatter,)
    parser.add_argument('input_file',
                        help="input register description yaml")
    parser.add_argument('output_generator',
                        help=f"select one of the builtin generators or provide a custom python file to use for output generator")
    parser.add_argument('generator_args', nargs=argparse.REMAINDER,
                        help="additioanl arguments passed to the selected output generator")

    args = parser.parse_args()

    selection = args.output_generator
    if selection in builtin_generators:
        generator = builtin_generators[selection]
    else:
        try:

            spec = importlib.util.spec_from_file_location("reginald.external_generator", selection)
            if spec is None:
                raise Exception

            gen_module = importlib.util.module_from_spec(spec)

            sys.modules["reginald.external_generator"] = gen_module

            if spec.loader is None:
                raise Exception

            spec.loader.exec_module(gen_module)

        except Exception:
            raise ReginaldException(
                "Error: Specified generator is not a builtin option, is not a python file that could be openend!")

        if not "Generator" in gen_module.__dict__:
            raise ReginaldException(
                "Error: Specified generator file does not contain a class named 'Generator'!")

        if not issubclass(gen_module.Generator, OutputGenerator):
            raise ReginaldException(
                "Error: Specified generator file's 'Generator' class does not inherit from reginald.OutputGenerator!")

        generator = gen_module.Generator

    return CLI(input_file=args.input_file, generator=generator, generator_args=args.generator_args)
