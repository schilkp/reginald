from os import path
from typing import List

from jinja2 import Environment, FileSystemLoader

import reginald.utils
from reginald.datamodel import RegisterMap
from reginald.generator import OutputGenerator


class JinjaGenerator(OutputGenerator):
    def __init__(self, desc: str, template_name: str):
        self.desc = desc
        self.template_name = template_name

    def description(self) -> str:
        return self.desc

    def generate(self, rmap: RegisterMap, input_file: str, output_file: str, args: List[str]):
        # TODO: This should be a package loader.
        env = Environment(
            loader=FileSystemLoader("reginald/builtin_templates"),
            trim_blocks=True, lstrip_blocks=True
        )

        template = env.get_template(self.template_name)

        result = template.render(
            rmap=rmap,
            input_file_full=input_file,
            input_file=path.basename(input_file),
            output_file_full=output_file,
            output_file=path.basename(output_file),
            args=args,
            c_sanitize=reginald.utils.c_sanitize,
            c_fitting_unsigned_type=reginald.utils.c_fitting_unsigned_type,
            str_pad_to_length=reginald.utils.str_pad_to_length,
            hex=hex,
        )

        with open(output_file, 'w') as outfile:
            outfile.write(result)
