import { createContext, useState, useContext, ReactNode } from "react";

import * as c_funcpack from "@/reginald/c_funcpack";

interface GeneratorConfigContextType {
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

  return (
    <GeneratorConfigContext.Provider value={{ CFuncpack, setCFuncpack }}>
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
