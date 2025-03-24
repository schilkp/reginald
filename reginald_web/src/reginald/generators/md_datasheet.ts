import { GeneratorProps } from "../generators";
import * as wasm from "reginald_wasm";
import { ListingFormat, toWasmFormat } from "../listing";

export type ConfigData = Record<never, never>;

export const defaultConfig: ConfigData = {};

export function generate(
  listing: string,
  listingFormat: ListingFormat,
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  _config: ConfigData,
): string {
  const input_format = toWasmFormat(listingFormat);
  return wasm.run_md_datasheet(listing, input_format);
}

export const properties: GeneratorProps = {
  title: "md.datasheet",
  description: "Markdown datasheet",
  editor_lang: "markdown",
  file_extension: "md",
  menu_group: "Markdown",
  generate: generate,
  options: [],
};
