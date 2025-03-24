import { GeneratorProps } from "../generators";
import * as wasm from "reginald_wasm";
import { ListingFormat, toWasmFormat } from "../listing";

export type ConfigData = {
  Endianness: "le" | "be" | "both";
  DeferToEndian: "le" | "be" | "off";
  RegistersAsBitfields: boolean;
  MaxEnumBitwidth: number;
  Includes: string[];
  FuncsStaticInline: boolean;
  FuncsAsPrototypes: boolean;
  ClangFormatGuard: boolean;
  IncludeGuards: boolean;
  GenerateEnums: boolean;
  GenerateEnumValidationMacros: boolean;
  GenerateStructs: boolean;
  GenerateStructConversionFuncs: boolean;
  GenerateRegisterProperties: boolean;
  GenerateGenericMacros: boolean;
};

export const defaultConfig: ConfigData = {
  Endianness: "le",
  DeferToEndian: "off",
  RegistersAsBitfields: true,
  MaxEnumBitwidth: 32,
  Includes: [],
  FuncsStaticInline: true,
  FuncsAsPrototypes: false,
  ClangFormatGuard: true,
  IncludeGuards: true,
  GenerateEnums: true,
  GenerateEnumValidationMacros: true,
  GenerateStructs: true,
  GenerateStructConversionFuncs: true,
  GenerateRegisterProperties: true,
  GenerateGenericMacros: true,
};

export function generate(
  listing: string,
  listingFormat: ListingFormat,
  config: ConfigData,
): string {
  const opts: wasm.CFuncpackOpts = new wasm.CFuncpackOpts();
  switch (config.Endianness) {
    case "le":
      opts.endianness = wasm.EndiannessImpl.Little;
      break;
    case "be":
      opts.endianness = wasm.EndiannessImpl.Big;
      break;
    default:
    case "both":
      opts.endianness = wasm.EndiannessImpl.Both;
      break;
  }
  switch (config.DeferToEndian) {
    case "le":
      opts.defer_to_endianness = wasm.Endianness.Little;
      break;
    case "be":
      opts.defer_to_endianness = wasm.Endianness.Big;
      break;
    default:
    case null:
      opts.defer_to_endianness = null;
      break;
  }
  opts.registers_as_bitfields = config.RegistersAsBitfields;
  opts.max_enum_bitwidth = config.MaxEnumBitwidth;
  opts.funcs_static_inline = config.FuncsStaticInline;
  opts.funcs_as_prototypes = config.FuncsAsPrototypes;
  opts.clang_format_guard = config.ClangFormatGuard;
  opts.include_guards = config.IncludeGuards;
  opts.gen_enums = config.GenerateEnums;
  opts.gen_enum_validation = config.GenerateEnumValidationMacros;
  opts.gen_structs = config.GenerateStructs;
  opts.gen_struct_conv = config.GenerateStructConversionFuncs;
  opts.gen_reg_properties = config.GenerateRegisterProperties;
  opts.gen_generics = config.GenerateGenericMacros;

  for (const include of config.Includes) {
    opts.add_include_push(include);
  }
  const input_format = toWasmFormat(listingFormat);
  return wasm.run_c_funcpack(listing, input_format, opts);
}

export const properties: GeneratorProps = {
  title: "c.funcpack",
  description: "C register structs with packing/unpacking functions",
  editor_lang: "c",
  file_extension: "h",
  menu_group: "C",
  generate: generate,
  options: [
    {
      kind: "header",
      id: "header",
      title: "Configuration",
      subtitle: "",
    },
    {
      kind: "checkbox",
      id: "RegistersAsBitfields",
      label: "Make structs bitfields",
      description: "",
    },
    {
      kind: "checkbox",
      id: "FuncsStaticInline",
      label: "Mark functions as static inline.",
      description: "",
    },
    {
      kind: "checkbox",
      id: "FuncsAsPrototypes",
      label: "Generate function protypes only.",
      description: "",
    },
    {
      kind: "checkbox",
      id: "ClangFormatGuard",
      label: "Wrap file in a 'clang-format: off' guard.",
      description: "",
    },
    {
      kind: "checkbox",
      id: "IncludeGuards",
      label: "Wrap file in include guards.",
      description: "",
    },
    {
      kind: "numeric",
      id: "MaxEnumBitwidth",
      label: "Maximum enum width:",
      description: "Larger enums are generated as macros",
      min: 1,
    },
    {
      kind: "select-single",
      id: "Endianness",
      label: "Endianness:",
      description: "Generate functions and enums with the given endianness.",
      options: { le: "Little", be: "Big", both: "Both" },
    },
    {
      kind: "select-single",
      id: "DeferToEndian",
      label: "Defer-to Endianness:",
      description:
        "For other endianness, generate only simple functions that defers to this implementation.",
      options: { le: "Little", be: "Big", off: "Off" },
    },
    {
      kind: "string-list-manager",
      id: "Includes",
      label: "Add extra includes:",
      description: "",
      ghost_text: "Extra Include..",
    },
    {
      kind: "separator",
      id: "sep1",
    },
    {
      kind: "header",
      id: "en_header",
      title: "Component Enable",
      subtitle:
        "Enable/disable different components of the generated source file.",
    },
    {
      kind: "checkbox",
      id: "GenerateEnums",
      label: "Enums",
    },
    {
      kind: "checkbox",
      id: "GenerateEnumValidationMacros",
      label: "Enum Validation Macros",
    },
    {
      kind: "checkbox",
      id: "GenerateStructs",
      label: "Register Layout Structs",
    },
    {
      kind: "checkbox",
      id: "GenerateStructConversionFuncs",
      label: "Layout Struct COnversion Functions",
    },
    {
      kind: "checkbox",
      id: "GenerateRegisterProperties",
      label: "Register Property Macros",
    },
    {
      kind: "checkbox",
      id: "GenerateGenericMacros",
      label: "Generic Macros",
    },
  ],
};
