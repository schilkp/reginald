import { useRef, lazy, Suspense, useEffect } from "react";
import type * as monaco from "monaco-editor";

// Import Monaco setup and editor together
const Editor = lazy(async () => {
  // We load both in parallel, but we only return the editor module
  const [, editorModule] = await Promise.all([
    import("../../lib/monaco-setup").then((module) => {
      module.setupMonaco();
      return module;
    }),
    import("@monaco-editor/react"),
  ]);
  return editorModule;
});

interface ListingEditorProps {
  value: string;
  selectedLanguage: string;
}

export function ListingEditor({ value, selectedLanguage }: ListingEditorProps) {
  const editorRef = useRef<monaco.editor.IStandaloneCodeEditor | null>(null);
  const monacoRef = useRef<typeof monaco | null>(null);

  const handleEditorDidMount = (
    editor: monaco.editor.IStandaloneCodeEditor,
    monaco: typeof import("monaco-editor"),
  ) => {
    editorRef.current = editor;
    monacoRef.current = monaco;
  };

  useEffect(() => {
    if (editorRef.current && monacoRef.current) {
      const model = editorRef.current.getModel();
      if (model) {
        monacoRef.current.editor.setModelLanguage(model, selectedLanguage);
        console.log("set current lang to : " + selectedLanguage);
      }
    }
    // TODO: Translate map if valid
  }, [selectedLanguage]);

  return (
    <div className="h-full w-full">
      <Suspense
        fallback={
          <div className="flex items-center justify-center h-full">
            Loading editor...
          </div>
        }
      >
        <Editor
          height="100%"
          defaultLanguage={selectedLanguage}
          value={value}
          onMount={handleEditorDidMount}
          options={{
            minimap: { enabled: false },
            scrollBeyondLastLine: true,
            fontSize: 12,
            wordWrap: "off",
            automaticLayout: true,
          }}
        />
      </Suspense>
    </div>
  );
}
