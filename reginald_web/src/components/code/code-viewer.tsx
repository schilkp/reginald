import { lazy, Suspense } from "react";
import type * as monaco from "monaco-editor";

// Import Monaco setup and editor together
const Editor = lazy(async () => {
  // We load both in parallel, but we only return the editor module
  const [, editorModule] = await Promise.all([
    import("@/utils/monaco-setup").then((module) => {
      module.setupMonaco();
      return module;
    }),
    import("@monaco-editor/react"),
  ]);
  return editorModule;
});

interface CodeOutputProps {
  value: string;
  language: string;
  viewerRef: React.MutableRefObject<monaco.editor.IStandaloneCodeEditor | null>;
}

export function CodeOutput({ value, language, viewerRef }: CodeOutputProps) {
  const handleEditorDidMount = (
    editor: monaco.editor.IStandaloneCodeEditor,
  ) => {
    viewerRef.current = editor;
  };

  return (
    <div className="h-full w-full">
      <Suspense
        fallback={
          <div className="flex items-center justify-center h-full">
            Loading viewer...
          </div>
        }
      >
        <Editor
          height="100%"
          defaultLanguage={language}
          value={value}
          onMount={handleEditorDidMount}
          options={{
            readOnly: true,
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
