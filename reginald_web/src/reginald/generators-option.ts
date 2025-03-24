export interface ConfigCheckbox {
  kind: "checkbox";
  id: string;
  label: string;
  description?: string;
}

export interface ConfigNumeric {
  kind: "numeric";
  id: string;
  label: string;
  description?: string;
  min?: number;
  max?: number;
}

export interface ConfigString {
  kind: "string";
  id: string;
  label: string;
  description?: string;
}

export interface ConfigSelectSingle {
  kind: "select-single";
  id: string;
  label: string;
  description?: string;
  options: Record<string, string>; // {key: Name}
}

export interface ConfigStringListManager {
  kind: "string-list-manager";
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
  | ConfigNumeric
  | ConfigString
  | ConfigSelectSingle
  | ConfigStringListManager
  | ConfigSeparator
  | ConfigHeader;
