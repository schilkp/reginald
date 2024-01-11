{%- macro brief_doc(docs, prefix="", suffix="") -%}
{%   if docs.brief is not none %}{{prefix}}{{docs.brief}}{{suffix}}{% endif %}
{%- endmacro -%}

{% macro c_macro(orig) %}{{ c_sanitize(orig)|upper }}{% endmacro -%}

{%- macro c_header(name) -%}
// === {{ str_pad_to_length(name + " ", "=", 80-7) }}
{%- endmacro -%}

/*
 * {{ rmap.map_name }} Register Map.
 * Note: Do not edit: Generated using Reginald.
 */
#ifndef {{ c_macro(output_file) }}_
#define {{ c_macro(output_file) }}_

{% for block in rmap.register_blocks.values() %}
{%   for template in block.register_templates.values() %}

{{ c_header(block.name + template.name) }}
{%     if not block.docs.empty() %}
{{ block.docs.as_multi_line(prefix="// ")}}
{%     endif %}

{%     set generic_reg_name = c_macro(block.name + template.name) -%}

{%     for instance_name, instance_start in block.instances.items() %}
#define {{ c_macro(rmap.map_name) }}__REG_{{ c_macro(instance_name+template.name) }} ({{ hex(instance_start+template.adr) }}U) // Register address{{ brief_doc(template.docs, ' "','"') }}.
{%     endfor -%}
{%     if block.instances|length > 1 and block.register_templates|length > 1%}
#define {{ c_macro(rmap.map_name) }}__REG_{{ generic_reg_name }}__OFFSET ({{ hex(template.adr) }}U) // Offset of {{ block.name + template.name }} register from start of {{ block.name }} block.
{%     endif -%}
{%     if template.reset_val is not none %}
#define {{ c_macro(rmap.map_name) }}__REG_{{ generic_reg_name }}__RESET ({{ hex(template.reset_val) }}U) // Reset value.
{%     endif -%}
{%     if template.always_write is not none %}
#define {{ c_macro(rmap.map_name) }}__REG_{{ generic_reg_name }}__ALWAYS_WRITE_MASK ({{ hex(template.always_write.bits.get_bitmask()) }}U) // Always-write bit mask.
#define {{ c_macro(rmap.map_name) }}__REG_{{ generic_reg_name }}__ALWAYS_WRITE_VAL ({{ hex(template.always_write.value) }}U) //  Always-write value.
{%     endif -%}
{%     for field in template.fields.values() %}
#define {{ c_macro(rmap.map_name) }}__REG_{{ generic_reg_name }}__FIELD_{{ c_macro(field.name) }} ({{ hex(field.bits.get_bitmask()) }}U) // Field mask{{ brief_doc(field.docs, ' "','"') }}.
{%       if field.enum is not none %}
{%         for entry in field.enum.entries.values() %}
#define {{ c_macro(rmap.map_name) }}__REG_{{ generic_reg_name }}__FIELD_{{ c_macro(field.name) }}__CONST_{{c_macro(entry.name)}} ({{ hex(entry.value) }}U) // Constant{{ brief_doc(entry.docs, ' "','"') }}.
{%         endfor -%}
{%       endif -%}
{%     endfor -%}
{%   endfor -%}
{%   if block.instances|length > 1 and block.register_templates|length > 1%}

{{ c_header(block.name) }}
{%     if not block.docs.empty() %}
{{ block.docs.as_multi_line(prefix="// ")}}
{%     endif %}

{%     for instance_name, instance_start in block.instances.items() %}
#define {{ c_macro(rmap.map_name) }}__BLOCK_{{ c_macro(block.name) }}__START_{{ c_macro(instance_name) }} ({{ hex(instance_start) }}U) // Start of {{ instance_name }} register block.
{%     endfor %}
{%   endif %}
{% endfor %}

#endif /* {{ c_macro(output_file) }}_ */
