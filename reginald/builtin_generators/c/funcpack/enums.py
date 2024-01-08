import os
from typing import Union

from reginald.builtin_generators.c.funcpack.name_generator import NameGenerator
from reginald.builtin_generators.c.funcpack.utils import doxy_comment
from reginald.datamodel import *
from reginald.utils import str_pad_to_length


def generate(rmap: RegisterMap, name: NameGenerator, output_file: str):
    out = []

    out.append(f"/*!")
    out.append(f" * \\file {name.filename_enums()}")
    out.append(f" * \\brief {rmap.map_name} Register Enums.")
    out.append(f" * \\note Do not edit: Generated using Reginald.")
    out.append(f" */")
    out.append(f"")
    out.append(f"")

    out.append(f"#ifndef {name.include_guard_macro(name.filename_enums())}")
    out.append(f"#define {name.include_guard_macro(name.filename_enums())}")
    out.append(f"")

    out.append(str_pad_to_length(f"// ==== Shared Enums ", "=", 80))
    out.append(f"")

    for enum in rmap.enums.values():
        out.extend(doxy_comment(enum.docs, prefix=""))
        out.append(f"enum {name.enum_shared(enum)}{{")

        for entry in enum.entries.values():
            out.extend(doxy_comment(entry.docs, prefix="  "))
            out.append(f"  {name.enum_shared_entry(enum, entry)} = 0x{entry.value:X}U,")

        out.append(f"}};")
        out.append(f"")

    for block_name, block in rmap.register_blocks.items():
        for template_name, template in block.register_templates.items():

            e = [field.enum for field in template.fields.values() if isinstance(field.enum, RegEnum) and not field.enum.is_shared]
            enum_count = len(e)

            if enum_count == 0:
                continue

            out.append(f"")
            out.append(str_pad_to_length(f"// ==== {block_name + template_name} Enums ", "=", 80))
            out.append(f"")

            for field in template.fields.values():
                if field.enum is not None and not field.enum.is_shared:
                    out.extend(doxy_comment(field.docs, prefix=""))
                    out.append(f"enum {name.enum_field(block_name, template_name, field.enum)}{{")
                    for entry in field.enum.entries.values():
                        out.extend(doxy_comment(entry.docs, prefix="  "))
                        out.append(f"  {name.enum_field_entry(block_name, template_name, field.enum, entry)} = 0x{entry.value:X}U,")
                    out.append(f"}};")
                    out.append(f"")

    out.append(f"#endif /* {name.include_guard_macro(name.filename_enums())} */")

    with open(output_file, 'w') as outfile:
        outfile.write("\n".join(out)+"\n")
    print(f"Generated {output_file}...")
