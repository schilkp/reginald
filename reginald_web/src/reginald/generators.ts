export interface GeneratorConfig {
  reset: () => void;
}

export type ReginaldGenerator = "c.funcpack" | "c.macromap" | "rs.structs";

export type EditorLang = "c" | "rust" | "markdown";

export type MenuGroup = "C" | "Rust";

export type GeneratorProps = {
  title: string;
  description: string;
  editor_lang: EditorLang;
  file_extension: string;
  menu_group: MenuGroup;
};

export const MenuGroups: MenuGroup[] = ["C", "Rust"];

export const generatorProps: Record<ReginaldGenerator, GeneratorProps> = {
  "c.funcpack": {
    title: "c.funcpack",
    description: "C register structs with packing/unpacking functions",
    editor_lang: "c",
    file_extension: "h",
    menu_group: "C",
  },
  "c.macromap": {
    title: "c.macromap",
    description: "C field mask/shift macros",
    editor_lang: "c",
    file_extension: "h",
    menu_group: "C",
  },
  "rs.structs": {
    title: "rs.structs",
    description: "Rust module with register structs and no dependencies",
    editor_lang: "rust",
    file_extension: "rs",
    menu_group: "Rust",
  },
};
