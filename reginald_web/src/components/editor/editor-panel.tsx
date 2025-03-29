import { useEditorContext } from "./editor-context";

import { useRef, lazy, Suspense, useEffect } from "react";
import type * as monaco from "monaco-editor";
import * as wasm from "reginald_wasm";
import { toast } from "sonner";
import { exampleYaml } from "./exampleYaml";

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
  let { editorRef, setEditorContent, listingFormat } = useEditorContext();

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
    let prev_format: wasm.ListingFormat;
    let new_format: wasm.ListingFormat;

    if (listingFormat === "yaml") {
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

      let msg =
        "Invalid listing - could not convert " +
        from_str +
        " to " +
        to_str +
        "!";
      console.error(msg + e);
      toast.error(msg);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [listingFormat]);

  const debounceTimeoutRef = useRef<NodeJS.Timeout | null>(null);

  function handleEditorChange(value: string | undefined) {
    if (debounceTimeoutRef.current) {
      clearTimeout(debounceTimeoutRef.current);
    }
    debounceTimeoutRef.current = setTimeout(() => {
      if (value !== undefined) {
        setEditorContent(value);
      }
    }, 200);
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
