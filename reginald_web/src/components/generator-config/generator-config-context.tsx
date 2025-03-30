import { createContext, useState, useContext, ReactNode } from "react";

import * as c_funcpack from "@/reginald/c_funcpack";
import { ReginaldGenerator } from "@/reginald/generators";

interface GeneratorConfigContextType {
  selectedGenerator: ReginaldGenerator;
  setSelectedGenerator(c: ReginaldGenerator): void;

  CFuncpack: c_funcpack.ConfigData;
  setCFuncpack(v: c_funcpack.ConfigData): void;
}

const GeneratorConfigContext = createContext<
  GeneratorConfigContextType | undefined
>(undefined);

export function GeneratorConfigContextProvider({
  children,
}: {
  children: ReactNode;
}) {
  let [CFuncpack, setCFuncpack] = useState(c_funcpack.defaultConfig);
  let [selectedGenerator, setSelectedGenerator] =
    useState<ReginaldGenerator>("c.funcpack");

  return (
    <GeneratorConfigContext.Provider
      value={{
        CFuncpack,
        setCFuncpack,
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
