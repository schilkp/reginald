import { useEditorContext } from "./editor-context";

import { useRef, lazy, Suspense, useEffect } from "react";
import type * as monaco from "monaco-editor";
import { toast } from "sonner";
import { exampleYaml } from "./exampleYaml";
import { convertListingFormat } from "@/reginald/listing";

// Import monaco and setup:
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

export function EditorPanel() {
  const { editorRef, setEditorContent, listingFormat } = useEditorContext();

  const monacoRef = useRef<typeof monaco | null>(null);

  const handleEditorDidMount = (
    editor: monaco.editor.IStandaloneCodeEditor,
    monaco: typeof import("monaco-editor"),
  ) => {
    editorRef.current = editor;
    monacoRef.current = monaco;
  };

  useEffect(() => {
    if (!editorRef.current || !monacoRef.current) {
      return;
    }
    const model = editorRef.current.getModel();
    if (!model) {
      return;
    }
    console.log("Setting current listing to: " + listingFormat);
    monacoRef.current.editor.setModelLanguage(model, listingFormat);

    const content = editorRef.current.getValue();
    const oldFormat = listingFormat === "yaml" ? "json" : "yaml";

    try {
      const new_content = convertListingFormat(
        content,
        oldFormat,
        listingFormat,
      );
      editorRef.current.setValue(new_content);
    } catch (e) {
      const msg =
        "Invalid listing - could not convert " +
        oldFormat +
        " to " +
        listingFormat +
        "!";
      console.error(msg + e);
      toast.error(msg);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [listingFormat]);

  // Propagate editor content to hook:
  function handleEditorChange(value: string | undefined) {
    if (value !== undefined) {
      setEditorContent(value);
    }
  }

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
          defaultLanguage={listingFormat}
          value={exampleYaml}
          onMount={handleEditorDidMount}
          onChange={handleEditorChange}
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
