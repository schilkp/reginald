export interface GeneratorConfig {
  reset: () => void;
}

export type Generator = "c.funcpack" | "c.macromap" | "rs.structs";

export type EditorLang = "c" | "rust" | "markdown";

export type GeneratorProps = {
  title: string;
  description: string;
  editor_lang: EditorLang;
  file_extension: string;
};

export const generatorProps: Record<Generator, GeneratorProps> = {
  "c.funcpack": {
    title: "c.funcpack",
    description: "C register structs with packing/unpacking functions",
    editor_lang: "c",
    file_extension: "h",
  },
  "c.macromap": {
    title: "c.macromap",
    description: "C field mask/shift macros",
    editor_lang: "c",
    file_extension: "h",
  },
  "rs.structs": {
    title: "rs.structs",
    description: "Rust module with register structs and no dependencies",
    editor_lang: "rust",
    file_extension: "rs",
  },
};
