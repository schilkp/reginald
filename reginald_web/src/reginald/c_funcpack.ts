export type ConfigData = {
  Endianess: "le" | "be" | "both";
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
  Endianess: "le",
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
