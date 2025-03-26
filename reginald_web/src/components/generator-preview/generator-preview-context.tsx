import { createContext, useContext, ReactNode, useRef } from "react";
import type * as monaco from "monaco-editor";

interface GeneratorPreviewContextType {
  // Monaco viewer (once mounted)
  viewerRef: React.MutableRefObject<monaco.editor.IStandaloneCodeEditor | null>;
}

const GeneratorPreviewContext = createContext<
  GeneratorPreviewContextType | undefined
>(undefined);

export function GeneratorPreviewContextProvider({
  children,
}: {
  children: ReactNode;
}) {
  const viewerRef = useRef<monaco.editor.IStandaloneCodeEditor | null>(null);
  return (
    <GeneratorPreviewContext.Provider
      value={{
        viewerRef,
      }}
    >
      {children}
    </GeneratorPreviewContext.Provider>
  );
}

export function useGeneratorPreviewContext(): GeneratorPreviewContextType {
  const context = useContext(GeneratorPreviewContext);
  if (context === undefined) {
    throw new Error(
      "useGeneratorPreviewContext must be used within an GeneratorPreviewContextProvider",
    );
  }
  return context;
}
