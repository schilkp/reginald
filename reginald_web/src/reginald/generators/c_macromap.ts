import { GeneratorProps } from "../generators";
import * as wasm from "reginald_wasm";
import { ListingFormat, toWasmFormat } from "../listing";

export type ConfigData = {
  ClangFormatGuard: boolean;
  Includes: string[];
};

export const defaultConfig: ConfigData = {
  ClangFormatGuard: true,
  Includes: [],
};

export function generate(
  listing: string,
  listingFormat: ListingFormat,
  config: ConfigData,
): string {
  const opts: wasm.CMacromapOpts = new wasm.CMacromapOpts();
  opts.clang_format_guard = config.ClangFormatGuard;
  for (const include of config.Includes) {
    opts.add_include_push(include);
  }
  const input_format = toWasmFormat(listingFormat);
  return wasm.run_c_macromap(listing, input_format, opts);
}

export const properties: GeneratorProps = {
  title: "c.macromap",
  description: "C field mask/shift macros",
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
      id: "ClangFormatGuard",
      label: "Wrap file in a 'clang-format: off' guard.",
      description: "",
    },
    {
      kind: "string-list-manager",
      id: "Includes",
      label: "Add extra includes:",
      description: "",
      ghost_text: "Extra Include..",
    },
  ],
};
