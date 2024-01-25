import argparse
from dataclasses import dataclass
from typing import List, Tuple

import reginald.builtin_generators.c.funcpack
import reginald.builtin_generators.c.macromap
import reginald.builtin_generators.external_jinja2_template
import reginald.builtin_generators.md.doc
import reginald.builtin_generators.md.regdumpanalysis
from reginald.error import ReginaldException
from reginald.generator import OutputGenerator

builtin_generators = {
    'c.macromap': reginald.builtin_generators.c.macromap.Generator(),
    'c.funcpack': reginald.builtin_generators.c.funcpack.Generator(),
    'md.regdumpanalysis': reginald.builtin_generators.md.regdumpanalysis.Generator(),
    'md.doc': reginald.builtin_generators.md.doc.Generator(),
    'jinja2': reginald.builtin_generators.external_jinja2_template.Generator()
}


@dataclass
class CLI:
    input_file: str
    output_file: str
    generator_args: List[str]


def parse_args() -> Tuple[CLI, OutputGenerator]:

    builtin_choices_text = []
    for name, generator in builtin_generators.items():
        builtin_choices_text.append(f"   {name}: {generator.description()}")

    builtin_choices_text = "\n".join(builtin_choices_text)

    parser = argparse.ArgumentParser(prog="Reginald",
                                     description='Register map utility.\nPhilipp Schilk, 2022-2023',
                                     epilog="builtin generators: \n" + builtin_choices_text,
                                     formatter_class=argparse.RawDescriptionHelpFormatter)

    parser.add_argument('--version', action='version', version='reginald ' + reginald.__version__)
    parser.add_argument('input_file',
                        help="input register description yaml")
    parser.add_argument('output_file',
                        help=f"name of file to be generated")
    parser.add_argument('output_generator',
                        help=f"builtin generator to use")
    parser.add_argument('generator_args', nargs=argparse.REMAINDER,
                        help="additional arguments passed to the selected output generator")

    args = parser.parse_args()

    selection = args.output_generator
    if selection in builtin_generators:
        generator = builtin_generators[selection]
    else:
        raise ReginaldException("Error: Unknown generator.")

    return CLI(input_file=args.input_file,
               output_file=args.output_file,
               generator_args=args.generator_args), generator
