import { useGeneratorPreviewContext } from "./generator-preview-context";
import { useRef, lazy, Suspense } from "react";
import type * as monaco from "monaco-editor";

// Import monaco and setup:
const Viewer = lazy(async () => {
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

export function GeneratorPreviewPanel() {
  let { viewerRef } = useGeneratorPreviewContext();

  const monacoRef = useRef<typeof monaco | null>(null);

  const handleEditorDidMount = (
    viewer: monaco.editor.IStandaloneCodeEditor,
    monaco: typeof import("monaco-editor"),
  ) => {
    viewerRef.current = viewer;
    monacoRef.current = monaco;
  };
  
  const initial_value = ""; // TODO
  //const initial_value =
  //  run_c_funcpack(
  //    selectedGenerator,
  //    editorContent,
  //    selectedLanguage,
  //    c_funcpack_config,
  //  ) || "";

  return (
    <div className="h-full w-full">
      <Suspense
        fallback={
          <div className="flex items-center justify-center h-full">
            Loading viewer...
          </div>
        }
      >
        <Viewer
          height="100%"
          defaultLanguage={"c"}
          value={initial_value}
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
