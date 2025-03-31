import { GeneratorOption } from "./generators-option";

export interface GeneratorConfig {
  reset: () => void;
}

export type ReginaldGenerator = "c.funcpack" | "c.macromap";

export type EditorLang = "c" | "rust" | "markdown";

export type MenuGroup = "C" | "Rust";

export type GeneratorProps = {
  title: string;
  description: string;
  editor_lang: EditorLang;
  file_extension: string;
  menu_group: MenuGroup;
  options: GeneratorOption[];
};

export const MenuGroups: MenuGroup[] = ["C", "Rust"];

export const generatorProps: Record<ReginaldGenerator, GeneratorProps> = {
  "c.funcpack": {
    title: "c.funcpack",
    description: "C register structs with packing/unpacking functions",
    editor_lang: "c",
    file_extension: "h",
    menu_group: "C",
    options: [
      // TODO: Endianess
      // TODO: DeferTo
      {
        kind: "checkbox",
        id: "RegistersAsBitfields",
        label: "Make structs bitfields",
        description: "",
      },
      {
        kind: "string-manager",
        id: "Includes",
        label: "Add extra includes",
        description: "",
        ghost_text: "Extra Include..",
      },
    ],
  },
  "c.macromap": {
    title: "c.macromap",
    description: "C field mask/shift macros",
    editor_lang: "c",
    file_extension: "h",
    menu_group: "C",
    options: [],
  },
};
