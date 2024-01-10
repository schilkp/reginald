import argparse

from jinja2 import Environment, FileSystemLoader

from reginald.datamodel import *
from reginald.generator import OutputGenerator
from reginald.jinja2_generator import render_jinja2_template

class Generator(OutputGenerator):
    def description(self) -> str:
        return "Jinja2 template."

    def generate(self, rmap: RegisterMap, input_file: str, output_file: str, args: List[str]):
        parser = argparse.ArgumentParser(prog="jinja2", description=self.description())
        parser.add_argument('template',
                            help="jinj2 template file")
        parser.add_argument('template_args', nargs=argparse.REMAINDER,
                            help="additional arguments passed template")

        parsed_args = parser.parse_args(args)

        env = Environment(
            loader=FileSystemLoader("."),
            trim_blocks=True, lstrip_blocks=True
        )

        template = env.get_template(parsed_args.template)

        render_jinja2_template(template, rmap, input_file, output_file, parsed_args.template_args)
