export interface ConfigCheckbox {
  kind: "checkbox";
  id: string;
  label: string;
  description?: string;
}

export interface ConfigStringManager {
  kind: "string-manager";
  id: string;
  label: string;
  description?: string;
  ghost_text: string;
}

export interface ConfigSeparator {
  kind: "separator";
  id: string;
}

export interface ConfigHeader {
  kind: "header";
  id: string;
  title: string;
  subtitle?: string;
}

export type GeneratorOption =
  | ConfigCheckbox
  | ConfigStringManager
  | ConfigSeparator
  | ConfigHeader;
