import type * as monaco from "monaco-editor";
import { createContext, useState, useContext, ReactNode, useRef } from "react";
import { exampleYaml } from "./exampleYaml";
import { ListingFormat } from "@/reginald/listing";

interface EditorContextType {
  // Monaco editor (once mounted)
  editorRef: React.MutableRefObject<monaco.editor.IStandaloneCodeEditor | null>;

  // Content of the monaco editor (debounced)
  // Note: set only changes the debounced variable, not the editor!
  editorContent: string;
  setEditorContent: (content: string) => void;

  // Listing format
  listingFormat: ListingFormat;
  setListingFormat: (language: ListingFormat) => void;
}

const EditorContext = createContext<EditorContextType | undefined>(undefined);

export function EditorContextProvider({ children }: { children: ReactNode }) {
  const editorRef = useRef<monaco.editor.IStandaloneCodeEditor | null>(null);
  const [editorContent, setEditorContent] = useState<string>(exampleYaml);
  const [selectedLanguage, setSelectedLanguage] =
    useState<ListingFormat>("yaml");

  return (
    <EditorContext.Provider
      value={{
        editorRef,
        editorContent,
        setEditorContent,
        listingFormat: selectedLanguage,
        setListingFormat: setSelectedLanguage,
      }}
    >
      {children}
    </EditorContext.Provider>
  );
}

// eslint-disable-next-line react-refresh/only-export-components
export function useEditorContext(): EditorContextType {
  const context = useContext(EditorContext);
  if (context === undefined) {
    throw new Error(
      "useEditorContext must be used within an EditorContextProvider",
    );
  }
  return context;
}
