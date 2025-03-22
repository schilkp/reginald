import { create } from "zustand";
import { ToWasmConfigConvertible } from "./state";

type Config = {
  EndianEnableLe: boolean;
  EndianEnableBe: boolean;
  DeferToEndian: "le" | "be" | null;
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

export type ConfigStore = Config & {
  update: (config: Partial<Config>) => void;
  set: <K extends keyof Config>(key: K, value: Config[K]) => void;
  reset: () => void;
} & ToWasmConfigConvertible;

const defaultConfig: Config = {
  EndianEnableLe: true,
  EndianEnableBe: false,
  DeferToEndian: null,
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

export const createConfigStore = (initialConfig?: Partial<Config>) => {
  const initialState = { ...defaultConfig, ...(initialConfig || {}) };

  return create<ConfigStore>((set, get) => ({
    ...initialState,

    update: (config) => set((state) => ({ ...state, ...config })),

    set: <K extends keyof Config>(key: K, value: Config[K]) =>
      set({ [key]: value } as Partial<Config>),

    reset: () => set(initialState),

    to_wasm: () => {
      const state = get();
      // Extract only the config properties (not the methods)
      const configKeys = Object.keys(defaultConfig) as Array<keyof Config>;
      const wasmObject: Record<string, any> = {};

      configKeys.forEach((key) => {
        wasmObject[key] = state[key];
      });

      return wasmObject;
    },
  }));
};
