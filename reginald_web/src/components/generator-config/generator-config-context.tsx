import { createContext, useState, useContext, ReactNode } from "react";

import * as c_funcpack from "@/reginald/generators/c_funcpack";
import * as c_macromap from "@/reginald/generators/c_macromap";
import * as rs_structs from "@/reginald/generators/rs_structs";
import * as md_datasheet from "@/reginald/generators/md_datasheet";

import { ReginaldGenerator } from "@/reginald/generators";

export type GeneratorConfigs = {
  "c.funcpack": c_funcpack.ConfigData;
  "c.macromap": c_macromap.ConfigData;
  "rs.structs": rs_structs.ConfigData;
  "md.datasheet": md_datasheet.ConfigData;
};

export type GeneratorConfigRegistry = {
  [K in ReginaldGenerator]: GeneratorConfigs[K];
};

interface GeneratorConfigContextType {
  selectedGenerator: ReginaldGenerator;
  setSelectedGenerator(c: ReginaldGenerator): void;

  generatorConfig: GeneratorConfigRegistry;
  setGeneratorConfig(c: GeneratorConfigRegistry): void;
}

const GeneratorConfigContext = createContext<
  GeneratorConfigContextType | undefined
>(undefined);

export function GeneratorConfigContextProvider({
  children,
}: {
  children: ReactNode;
}) {
  const [selectedGenerator, setSelectedGenerator] =
    useState<ReginaldGenerator>("c.funcpack");

  const [generatorConfig, setGeneratorConfig] =
    useState<GeneratorConfigRegistry>({
      "c.funcpack": c_funcpack.defaultConfig,
      "c.macromap": c_macromap.defaultConfig,
      "rs.structs": rs_structs.defaultConfig,
      "md.datasheet": md_datasheet.defaultConfig,
    });

  return (
    <GeneratorConfigContext.Provider
      value={{
        generatorConfig,
        setGeneratorConfig,
        selectedGenerator,
        setSelectedGenerator,
      }}
    >
      {children}
    </GeneratorConfigContext.Provider>
  );
}

// eslint-disable-next-line react-refresh/only-export-components
export function useGeneratorConfigContext(): GeneratorConfigContextType {
  const context = useContext(GeneratorConfigContext);
  if (context === undefined) {
    throw new Error(
      "useGeneratorConfigContext must be used within an GeneratorConfigContextProvider",
    );
  }
  return context;
}
