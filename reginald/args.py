import argparse
import importlib.util
import sys

import reginald.builtin_generators.c.funcpack.funcpack
import reginald.builtin_generators.c.macromap
import reginald.builtin_generators.md.doc
import reginald.builtin_generators.md.regdumpanalysis
from reginald.cli import CLI
from reginald.error import ReginaldException
from reginald.generator import OutputGenerator

builtin_generators = {
    'c.macromap': reginald.builtin_generators.c.macromap.Generator,
    'c.funcpack': reginald.builtin_generators.c.funcpack.funcpack.Generator,
    'md.regdumpanalysis': reginald.builtin_generators.md.regdumpanalysis.Generator,
    'md.doc': reginald.builtin_generators.md.doc.Generator
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
    parser.add_argument('output_path',
                        help=f"path to store generated files at")
    parser.add_argument('generator_args', nargs=argparse.REMAINDER,
                        help="additioanl arguments passed to the selected output generator")

    args = parser.parse_args()

    selection = args.output_generator
    if selection.endswith(".py"):
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
                "Error: Specified generator is not a python file that could be openend!")

        if not "Generator" in gen_module.__dict__:
            raise ReginaldException(
                "Error: Specified generator file does not contain a class named 'Generator'!")

        if not issubclass(gen_module.Generator, OutputGenerator):
            raise ReginaldException(
                "Error: Specified generator file's 'Generator' class does not inherit from reginald.OutputGenerator!")

        generator = gen_module.Generator
    else:
        if selection in builtin_generators:
            generator = builtin_generators[selection]
        else:
            raise ReginaldException("Error: Unknown generator.")

    return CLI(input_file=args.input_file, generator=generator, generator_args=args.generator_args, output_path=args.output_path)
