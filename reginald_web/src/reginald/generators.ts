import { GeneratorOption } from "./generators-option";
import * as c_funcpack from "./generators/c_funcpack";
import * as c_macromap from "./generators/c_macromap";
import * as rs_structs from "./generators/rs_structs";
import * as md_datasheet from "./generators/md_datasheet";
import { ListingFormat } from "./listing";

export interface GeneratorConfig {
  reset: () => void;
}

export type ReginaldGenerator =
  | "c.funcpack"
  | "c.macromap"
  | "rs.structs"
  | "md.datasheet";

export type EditorLang = "c" | "rust" | "markdown";

export type MenuGroup = "C" | "Rust" | "Markdown";
export const MenuGroups: MenuGroup[] = ["C", "Rust", "Markdown"];

export type GeneratorProps = {
  title: string;
  description: string;
  editor_lang: EditorLang;
  file_extension: string;
  menu_group: MenuGroup;
  options: GeneratorOption[];
  generate: (
    listing: string,
    listingFormat: ListingFormat,
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    config: any,
  ) => string;
};

export const generatorProps: Record<ReginaldGenerator, GeneratorProps> = {
  "c.funcpack": c_funcpack.properties,
  "c.macromap": c_macromap.properties,
  "rs.structs": rs_structs.properties,
  "md.datasheet": md_datasheet.properties,
};
