{% macro brief_doc(docs, prefix="", suffix="") %}
{%   if docs.brief is not none %}{{prefix}}{{docs.brief}}{{suffix}}{% endif %}
{%- endmacro -%}

{%- macro c_macro(orig) %}{{ c_sanitize(orig)|upper }}{%- endmacro -%}

{%- macro c_code(orig) %}{{ c_sanitize(orig)|lower }}{%- endmacro -%}

{% macro c_header(name) %}
// === {{ str_pad_to_length(name + " ", "=", 80-7) }}
{%- endmacro -%}

{%- macro c_doxy_comment(docs, prefix="") -%}
{%   if docs.brief is not none and docs.doc is none %}
{{ prefix }}/** @brief {{docs.brief}} */
{%-   elif docs.doc is not none %}
{{ prefix }}/**
{%     if docs.brief is not none %}
{{ prefix }} * @brief {{docs.brief}}
{%     endif %}
{%     for line in docs.doc.splitlines() %}
{{ prefix }} * {{line}}
{%     endfor %}
{{ prefix }} */
{%-   endif %}
{%- endmacro -%}

{%- macro shared_enum_name(enum, rmap) -%}
{{ c_code(rmap.map_name) }}_{{ c_code(enum.name) }}
{%- endmacro -%}

{%- macro register_enum_name(block, template, enum, rmap) -%}
{%-   if "--no-field-enum-prefix" in rmap.args -%}
{{ c_code(rmap.map_name) }}_{{ c_code(enum.name) }}
{%-   else -%}
{{ c_code(rmap.map_name) }}_{{ c_code(block.name + template.name) }}_{{ c_code(enum.name) }}
{%-   endif %}
{%- endmacro -%}

{%- macro register_member_type(block, template, field, rmap) -%}
{%-   if field.enum is none -%}
{{ c_fitting_unsigned_type(field.bits.total_width()) }}
{%-   else -%}
{%-     if field.enum.is_shared -%}
enum {{ shared_enum_name(field.enum, rmap) }}
{%-     else -%}
enum {{ register_enum_name(block, template, field.enum, rmap) }}
{%-     endif -%}
{%-   endif -%}
{%- endmacro -%}

{%- macro generate_shared_enum(enum) %}

{{ c_doxy_comment(enum.docs) }}
enum {{ shared_enum_name(enum, rmap) }} {
{%   for entry in enum.entries.values() %}
{{ c_doxy_comment(entry.docs, prefix="  ") }}
  {{ c_macro(rmap.map_name) }}_{{ c_macro(enum.name) }}_{{ c_macro(entry.name) }} = {{ hex(entry.value) }}U,
{%   endfor %}
};
{%- endmacro -%}

{%- macro generate_register_enum(block, template, enum, rmap) %}

{{ c_doxy_comment(enum.docs) }}
enum {{ register_enum_name(block, template, enum, rmap) }} {
{%       for entry in enum.entries.values() %}
{{ c_doxy_comment(entry.docs, prefix="  ") }}
  {{ c_macro(register_enum_name(block, template, enum, rmap)) }}_{{ c_macro(entry.name) }} = {{ hex(entry.value) }}U,
{%       endfor %}
};
{%- endmacro -%}

{%- macro generate_register_defines(block, template, rmap) %}
{%   for instance_name, instance_start in block.instances.items() %}
#define {{ c_macro(rmap.map_name) }}__REG_{{ c_macro(instance_name+template.name) }} ({{ hex(instance_start+template.adr) }}U) // Register address{{ brief_doc(template.docs, ' "','"') }}.
{%   endfor -%}
{%   if block.instances|length > 1 and block.register_templates|length > 1%}

#define {{ c_macro(rmap.map_name) }}__REG_{{ c_macro(block.name + template.name) }}__OFFSET ({{ hex(template.adr) }}U) // Offset of {{ block.name + template.name }} register from start of {{ block.name }} block.
{%   endif -%}
{%   if template.reset_val is not none %}

#define {{ c_macro(rmap.map_name) }}__REG_{{ c_macro(block.name + template.name) }}__RESET ({{ hex(template.reset_val) }}U) // Reset value.
{%   endif -%}
{%   if template.always_write is not none %}

#define {{ c_macro(rmap.map_name) }}__REG_{{ c_macro(block.name + template.name) }}__ALWAYS_WRITE_MASK ({{ hex(template.always_write.bits.get_bitmask()) }}U) // Always-write bit mask.
#define {{ c_macro(rmap.map_name) }}__REG_{{ c_macro(block.name + template.name) }}__ALWAYS_WRITE_VAL ({{ hex(template.always_write.value) }}U) //  Always-write value.
{%   endif -%}
{%- endmacro -%}

{%- macro generate_register_struct(block, template, rmap) %}

// TODO: note about packing funcs
{{   c_doxy_comment(template.docs) }}
struct {{ c_code(rmap.map_name) }}_{{ c_code(c_macro(block.name + template.name)) }} {
{%   for field in template.fields.values() %}
{{ c_doxy_comment(field.docs, prefix="  ") }}
  {{ register_member_type(block, template, field, rmap) }} {{ c_code(field.name) }} : {{ field.bits.total_width() }};
{%   endfor %}
};
{%- endmacro -%}

/*!
 * @file {{output_file}}
 * @brief {{rmap.map_name}} Register Enums.
 * @note Do not edit: Generated using Reginald from {{input_file}}.
 */
#ifndef {{ c_macro(output_file) }}_
#define {{ c_macro(output_file) }}_

#include <stdint.h>

{{ c_header("Enums") }}

{% for enum in rmap.enums.values() %}
{{   generate_shared_enum(enum) }}
{% endfor %}

{% for block in rmap.register_blocks.values() %}
{%   for template in block.register_templates.values() %}
{{     c_header(block.name + template.name+ " register") }}
{%     if not block.docs.empty() %}
{{        block.docs.as_multi_line(prefix="// ")}}
{%     endif %}
{{     generate_register_defines(block, template, rmap) }}
{%     for enum in template.get_local_enums() %}
{{       generate_register_enum(block, template, enum, rmap) }}
{%     endfor -%}
{%     if template.fields|length > 0 %}
{{       generate_register_struct(block, template, rmap) }}
{%     endif %}
{%   endfor -%}
{%   if block.instances|length > 1 and block.register_templates|length > 1 %}
{{     c_header(block.name + " register blocks") }}
{%     if not block.docs.empty() %}
{{       block.docs.as_multi_line(prefix="// ")}}
{%     endif %}
{%     for instance_name, instance_start in block.instances.items() %}
#define {{ c_macro(rmap.map_name) }}__BLOCK_{{ c_macro(block.name) }}__START_{{ c_macro(instance_name) }} ({{ hex(instance_start) }}U) // Start of {{ instance_name }} register block.
{%     endfor %}
{%   endif %}
{% endfor %}

#endif /* {{ c_macro(output_file) }}_ */
