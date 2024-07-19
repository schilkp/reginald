{{!

              ____   _    ____ _____ ___    _    _     ____  
             |  _ \ / \  |  _ \_   _|_ _|  / \  | |   / ___| 
             | |_) / _ \ | |_) || |  | |  / _ \ | |   \___ \ 
             |  __/ ___ \|  _ < | |  | | / ___ \| |___ ___) |
             |_| /_/   \_\_| \_\|_| |___/_/   \_\_____|____/ 

~}}

{{! 
    Layout defines (Fixed bits, fields, field enums). Context: Register 
~}}
{{ #*inline "file_header" }}
{{ #if opts.clang_format_guard ~}} // clang-format off {{~ /if }}
/**
 * @file {{ output_file }}
 * @brief {{ map.name }}
 * @note do not edit directly: generated using reginald{{#if map.from_file}} from {{ map.from_file }}{{/if}}.
 *
 * Generator: c.macromap
{{#if map.author}}
 *
 * Listing file author: {{ map.author }}
{{/if}}
{{#if map.notice}}
 *
 * Listing file notice:
{{#prefix_lines_with " *   "}}{{map.notice}}{{/prefix_lines_with}}{{/if}} */
#ifndef REGINALD_{{c_macro output_file}}
#define REGINALD_{{c_macro output_file}}

#include <stdint.h>
{{ #each opts.add_include }}
#include "{{this}}"
{{ /each }}

{{/inline ~}}


{{! 
    Layout defines (Fixed bits, fields, field enums). Context: Register 
}}
{{#*inline "layout_defines"}}
{{~ #if (layout_contains_fixed_bits this.layout) }}

{{#align_lines " " 4 false}}
#define {{c_macro @root.map.name}}_{{c_macro this.name}}_ALWAYSWRITE_MASK (0x{{hex (layout_fixed_bits_mask this.layout) }}U) //!< {{ this.name }} register always write mask
#define {{c_macro @root.map.name}}_{{c_macro this.name}}_ALWAYSWRITE_VALUE (0x{{hex (layout_fixed_bits_val this.layout) }}U) //!< {{ this.name }} register always write value
{{/align_lines}}
{{ /if }}

// Fields: 
{{#align_lines " " 4 false}}
{{ #each (layout_nested_fields_with_content this.layout)}}
#define {{c_macro @root.map.name}}_{{c_macro ../name}}_{{c_macro (join this.name "_") }}_MASK (0x{{hex this.mask }}U) //!< {{ ../name }}.{{ join this.name " "}}: bit mask (shifted)
#define {{c_macro @root.map.name}}_{{c_macro ../name}}_{{c_macro (join this.name "_") }}_SHIFT ({{ mask_lsb_pos this.mask }}U) //!< {{ ../name }}.{{ join this.name " "}}: bit shift
{{ #each (field_enum_entries this.field) }}
#define {{c_macro @root.map.name}}_{{c_macro ../../name}}_{{c_macro (join ../name "_") }}_VAL_{{ c_macro this.name }} (0x{{hex this.value }}U) //!< {{ ../../name }}.{{ join ../name " "}}: Value {{ this.name }}
{{ /each }}

{{ /each }}
{{/align_lines}}
{{/inline ~}}

{{!

     __  __    _    ___ _   _   _____ _____ __  __ ____  _        _  _____ _____ 
    |  \/  |  / \  |_ _| \ | | |_   _| ____|  \/  |  _ \| |      / \|_   _| ____|
    | |\/| | / _ \  | ||  \| |   | | |  _| | |\/| | |_) | |     / _ \ | | |  _|  
    | |  | |/ ___ \ | || |\  |   | | | |___| |  | |  __/| |___ / ___ \| | | |___ 
    |_|  |_/_/   \_\___|_| \_|   |_| |_____|_|  |_|_|   |_____/_/   \_\_| |_____|
    
}}
{{> file_header}}

{{#each map.registers}}
{{#unless from_block}}
{{c_section_header_comment (concat this.name " Register")}}

{{#align_lines " " 4 false}}
#define {{c_macro @root.map.name}}_{{c_macro this.name}}_ADDRESS (0x{{hex this.adr }}U) //!< {{ this.name }} register address
{{#unless (is_null this.reset_val)~}}
#define {{c_macro @root.map.name}}_{{c_macro this.name}}_RESET (0x{{hex this.reset_val }}U) //!< {{ this.name }} register reset value
{{~/unless}}
{{/align_lines~}}
{{> layout_defines this }}

{{ /unless }}
{{ /each }}

#endif /* REGINALD_{{c_macro output_file}} */
{{#if opts.clang_format_guard~}} // clang-format on {{ /if }}
