import { useRef, lazy, Suspense, useEffect, RefObject } from "react";
import type * as monaco from "monaco-editor";
import * as wasm from "reginald_wasm";
import { toast } from "sonner";

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

export function ListingEditor({
  value,
  selectedLanguage,
  editorRef,
}: {
  value: string;
  selectedLanguage: string;
  editorRef: RefObject<monaco.editor.IStandaloneCodeEditor | null>;
}) {
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
    console.log("Setting current lang to: " + selectedLanguage);
    monacoRef.current.editor.setModelLanguage(model, selectedLanguage);

    const content = editorRef.current.getValue();
    let prev_format: wasm.ListingFormat;
    let new_format: wasm.ListingFormat;

    if (selectedLanguage === "yaml") {
      prev_format = wasm.ListingFormat.Json;
      new_format = wasm.ListingFormat.Yaml;
    } else {
      prev_format = wasm.ListingFormat.Yaml;
      new_format = wasm.ListingFormat.Json;
    }

    // Only try conversion if the listing is not already in a parseable form
    if (wasm.is_parseable_listing(content, new_format)) {
      return;
    }

    try {
      const new_content = wasm.convert_listing_format(
        content,
        prev_format,
        new_format,
      );
      editorRef.current.setValue(new_content);
    } catch (e) {
      const from_str = wasm.listing_format_to_string(prev_format);
      const to_str = wasm.listing_format_to_string(new_format);
      console.error(
        "Listing format conversion from " +
          from_str +
          " to " +
          to_str +
          " failed: " +
          e,
      );
      toast.error(
        "Invalid listing - could not convert " +
          from_str +
          " to " +
          to_str +
          "!",
      );
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
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
