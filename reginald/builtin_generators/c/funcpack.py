from reginald.jinja2_generator import JinjaGenerator

class Generator(JinjaGenerator):
    def __init__(self):
        super().__init__(desc="C header with register structs and packing/unpacking functions.", template_name="c/funcpack.h")
