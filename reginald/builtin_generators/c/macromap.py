from reginald.jinja2_generator import JinjaGenerator


class Generator(JinjaGenerator):
    def __init__(self):
        super().__init__(desc="C header with traditional register and field macros.", template_name="c/macromap.h")
