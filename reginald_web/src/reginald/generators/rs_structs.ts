import { GeneratorProps } from "../generators";
import * as wasm from "reginald_wasm";
import { ListingFormat, toWasmFormat } from "../listing";

export type ConfigData = {
  AddressType: string;
  StructDerive: string[];
  EnumDerive: string[];
  AddUse: string[];
  AddAttribute: string[];
  ExternalTrait: string;
  GenerateUintConversion: boolean;
};

export const defaultConfig: ConfigData = {
  AddressType: "",
  StructDerive: [],
  EnumDerive: [],
  AddUse: [],
  AddAttribute: [],
  ExternalTrait: "",
  GenerateUintConversion: true,
};

export function generate(
  listing: string,
  listingFormat: ListingFormat,
  config: ConfigData,
): string {
  const opts: wasm.RsStructsOpts = new wasm.RsStructsOpts();

  opts.address_type = config.AddressType;

  for (const derive of config.StructDerive) {
    opts.struct_derive_push(derive);
  }
  for (const derive of config.EnumDerive) {
    opts.enum_derive_push(derive);
  }
  for (const use of config.AddUse) {
    opts.add_use_push(use);
  }
  for (const attr of config.AddAttribute) {
    opts.add_attribute_push(attr);
  }

  opts.external_traits = config.ExternalTrait;

  opts.generate_uint_conversion = config.GenerateUintConversion;

  const input_format = toWasmFormat(listingFormat);
  return wasm.run_rs_structs(listing, input_format, opts);
}

export const properties: GeneratorProps = {
  title: "rs.structs",
  description: "Rust structs with traits",
  editor_lang: "rust",
  file_extension: "rs",
  menu_group: "Rust",
  generate: generate,
  options: [
    {
      kind: "header",
      id: "header",
      title: "Configuration",
      subtitle: "",
    },
    {
      kind: "string",
      id: "AddressType",
      label: "Address Type:",
      description: "",
    },
    {
      kind: "string-list-manager",
      id: "StructDerive",
      label: "Add extra struct derive:",
      description: "",
      ghost_text: "Extra Derive..",
    },
    {
      kind: "string-list-manager",
      id: "EnumDerive",
      label: "Add extra enum derive:",
      description: "",
      ghost_text: "Extra Derive..",
    },
    {
      kind: "string-list-manager",
      id: "AddUse",
      label: "Add extra use statement:",
      description: "",
      ghost_text: "Extra Use..",
    },
    {
      kind: "string-list-manager",
      id: "AddAttribute",
      label: "Add extra use module attribute:",
      description: "",
      ghost_text: "Extra Attribute..",
    },
    {
      kind: "string",
      id: "ExternalTrait",
      label: "External Trait:",
      description: "",
    },
    {
      kind: "checkbox",
      id: "GenerateUintConversion",
      label: "Generate UInt Conversion",
      description: "",
    },
  ],
};
