import { useState } from "react";
import { GeneratorConfig } from "./config";

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

const defaultConfig: ConfigData = {
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

export type Config = GeneratorConfig<ConfigData> & ConfigData;

export function useConfigState(
  initialConfig: Partial<ConfigData> = {},
): Config {
  const [configValues, setConfigValues] = useState<ConfigData>({
    ...defaultConfig,
    ...initialConfig,
  });

  const updateProperty = <K extends keyof ConfigData>(
    propertyName: K,
    newValue: ConfigData[K],
  ) => {
    setConfigValues((prevConfig) => ({
      ...prevConfig,
      [propertyName]: newValue,
    }));
  };

  const reset = () => {
    setConfigValues({
      ...defaultConfig,
      ...initialConfig,
    });
  };

  const exportConfig = () => {
    return {
      ...configValues,
      exportedAt: new Date().toISOString(),
    };
  };

  return {
    ...configValues,
    updateProperty,
    reset,
    export: exportConfig,
  };
}
