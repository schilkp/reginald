from reginald.jinja2_generator import BuiltinJinjaGenerator


class Generator(BuiltinJinjaGenerator):
    def __init__(self):
        super().__init__(desc="C header with traditional register and field macros.", template_name="c/macromap.h")
