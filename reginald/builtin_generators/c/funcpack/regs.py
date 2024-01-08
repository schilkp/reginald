from reginald.builtin_generators.c.funcpack.name_generator import NameGenerator
from reginald.builtin_generators.c.funcpack.utils import doxy_comment
from reginald.datamodel import *
from reginald.utils import str_pad_to_length


def generate(rmap: RegisterMap, name: NameGenerator, output_file: str):

    out = []

    out.append(f"/*!")
    out.append(f" * \\file {name.filename_regs()}")
    out.append(f" * \\brief {rmap.map_name} Registers.")
    out.append(f" * \\note Do not edit: Generated using Reginald.")
    out.append(f" */")
    out.append(f"")
    out.append(f"")

    out.append(f"#ifndef {name.include_guard_macro(name.filename_regs())}")
    out.append(f"#define {name.include_guard_macro(name.filename_regs())}")
    out.append(f"")
    out.append(f"#include <stdint.h>")
    out.append(f"")

    out.append(str_pad_to_length(f"// ==== Register Addresses ", "=", 80))
    for block in rmap.registers.values():
        for template_name, template in block.registers.items():
            for instance_name, instance_adr in block.instances.items():
                adr = template.offset + instance_adr
                if template.docs.brief is not None:
                    comment = f"//!< {instance_name+template_name} Address ({template.docs.brief})"
                else:
                    comment = f"//!< {instance_name+template_name} Address"
                out.append(f"#define {name.reg_adr_macro(instance_name)} (0x{adr:X}U) {comment}")
    out.append(f"")

    # TODO: Block start adrs + offset adrs

    for block_name, block in rmap.registers.items():
        for template_name, template in block.registers.items():

            out.append(str_pad_to_length(f"// ==== {block_name+template_name} ", "=", 80))
            out.append(f"")

            if template.reset_val is not None:
                out.append(f"#define {name.reg_resetval_macro(block_name, template_name)} (0x{template.reset_val:X}U) //!< {block_name+template_name} Reset Value")
                out.append(f"")

            if template.always_write is not None:
                mask = template.always_write.bits.get_bitmask()
                value = template.always_write.value
                out.append(f"#define {name.reg_alwayswrite_mask_macro(block_name, template_name)} (0x{mask:X}U) //!< {block_name+template_name} Always Write Mask")
                out.append(f"#define {name.reg_alwayswrite_val_macro(block_name, template_name)} (0x{value:X}U) //!< {block_name+template_name} Always Write Value")
                out.append(f"")

            if len(template.fields) == 0:
                # Don't generate structs + funcs if there are no fields.
                continue

            # Generate register struct:
            struct_explain = []
            struct_explain.append(f"Use \\ref {name.doxygroup_regfuncs(block_name, template_name)} or "
                                  f"\\ref {name.doxygroup_genericfuncs()} to convert this struct to "
                                  f"and from it's packed binary form.")
            struct_explain.extend(template.docs.as_multi_line(prefix=""))

            struct_doc = "\n".join(struct_explain)

            struct_docs = Docs(brief=f"{block_name+template_name} Register Struct", doc=struct_doc)
            out.extend(doxy_comment(struct_docs, prefix=""))
            out.append(f"struct {name.reg_struct_name(block_name, template_name)} {{")
            for field in template.fields.values():
                field_type = name.reg_struct_member_type(block_name, template_name, field)
                out.extend(doxy_comment(field.docs, prefix="  "))
                out.append(f"  {field_type} {name.reg_struct_member(field)} : {field.bits.total_width()};")
            out.append(f"}};")
            out.append(f"")

    out.append(f"")
    out.append(f"#endif /* {name.include_guard_macro(name.filename_regs())} */")

    with open(output_file, 'w') as outfile:
        outfile.write("\n".join(out)+"\n")
    print(f"Generated {output_file}...")
