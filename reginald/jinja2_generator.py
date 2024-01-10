from os import path
from typing import List

from jinja2 import Environment, PackageLoader

import reginald.utils
from reginald.datamodel import RegisterMap
from reginald.generator import OutputGenerator


class BuiltinJinjaGenerator(OutputGenerator):
    def __init__(self, desc: str, template_name: str):
        self.desc = desc
        self.template_name = template_name

    def description(self) -> str:
        return self.desc

    def generate(self, rmap: RegisterMap, input_file: str, output_file: str, args: List[str]):
        env = Environment(
            loader=PackageLoader("reginald", "builtin_templates"),
            trim_blocks=True, lstrip_blocks=True
        )

        template = env.get_template(self.template_name)

        render_jinja2_template(template, rmap, input_file, output_file, args)


def render_jinja2_template(template, rmap: RegisterMap, input_file: str, output_file: str, args: List[str]):

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
    print(f"Generated {output_file}...")
