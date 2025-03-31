import { createContext, useState, useContext, ReactNode } from "react";

import * as c_funcpack from "@/reginald/c_funcpack";
import * as c_macromap from "@/reginald/c_macromap";
import { ReginaldGenerator } from "@/reginald/generators";

export type GeneratorConfigs = {
  "c.funcpack": c_funcpack.ConfigData;
  "c.macromap": c_macromap.ConfigData;
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
  let [selectedGenerator, setSelectedGenerator] =
    useState<ReginaldGenerator>("c.funcpack");

  let [generatorConfig, setGeneratorConfig] = useState<GeneratorConfigRegistry>(
    {
      "c.funcpack": c_funcpack.defaultConfig,
      "c.macromap": c_macromap.defaultConfig,
    },
  );

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

export function useGeneratorConfigContext(): GeneratorConfigContextType {
  const context = useContext(GeneratorConfigContext);
  if (context === undefined) {
    throw new Error(
      "useGeneratorConfigContext must be used within an GeneratorConfigContextProvider",
    );
  }
  return context;
}
