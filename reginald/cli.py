import argparse
from dataclasses import dataclass
from typing import Dict, List, Tuple

import reginald.builtin_generators.c.funcpack.funcpack
import reginald.builtin_generators.c.macromap
import reginald.builtin_generators.md.doc
import reginald.builtin_generators.md.regdumpanalysis
from reginald.error import ReginaldException
from reginald.generator import OutputGenerator

builtin_generators = {
    'c.macromap': reginald.builtin_generators.c.macromap.Generator(),
    'c.funcpack.regs': reginald.builtin_generators.c.funcpack.funcpack.GeneratorRegs(),
    'c.funcpack.regutils': reginald.builtin_generators.c.funcpack.funcpack.GeneratorRegUtils(),
    'c.funcpack.enums': reginald.builtin_generators.c.funcpack.funcpack.GeneratorEnums(),
    'md.regdumpanalysis': reginald.builtin_generators.md.regdumpanalysis.Generator(),
    'md.doc': reginald.builtin_generators.md.doc.Generator()
}  # type: Dict[str, OutputGenerator]


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
                                     description='Register map utility', epilog="builtin generators: \n" +
                                     builtin_choices_text, formatter_class=argparse.RawDescriptionHelpFormatter,)
    parser.add_argument('input_file',
                        help="input register description yaml")
    parser.add_argument('output_file',
                        help=f"name of file to be generated")
    parser.add_argument('output_generator',
                        help=f"builtin generator to use")
    parser.add_argument('generator_args', nargs=argparse.REMAINDER,
                        help="additioanl arguments passed to the selected output generator")

    args = parser.parse_args()

    selection = args.output_generator
    if selection in builtin_generators:
        generator = builtin_generators[selection]
    else:
        raise ReginaldException("Error: Unknown generator.")

    return CLI(input_file=args.input_file,
               output_file=args.output_file,
               generator_args=args.generator_args), generator
