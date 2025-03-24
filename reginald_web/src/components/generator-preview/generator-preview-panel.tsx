import { useGeneratorPreviewContext } from "./generator-preview-context";
import { useRef, lazy, Suspense, useMemo, useEffect, useState } from "react";
import type * as monaco from "monaco-editor";
import { useGeneratorConfigContext } from "../generator-config/generator-config-context";
import { generatorProps } from "@/reginald/generators";
import { useEditorContext } from "../editor/editor-context";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { AlertCircle } from "lucide-react";

// Import monaco and setup:
const Viewer = lazy(async () => {
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

export function GeneratorPreviewPanel() {
  const { listingFormat, editorContent } = useEditorContext();
  const { viewerRef } = useGeneratorPreviewContext();
  const { selectedGenerator, generatorConfig } = useGeneratorConfigContext();

  const [alertMsg, setAlertMsg] = useState<string | null>(null);

  const monacoRef = useRef<typeof monaco | null>(null);

  const handleEditorDidMount = (
    viewer: monaco.editor.IStandaloneCodeEditor,
    monaco: typeof import("monaco-editor"),
  ) => {
    viewerRef.current = viewer;
    monacoRef.current = monaco;
  };

  const initial_value = useMemo(() => {
    return generatorProps[selectedGenerator].generate(
      editorContent,
      listingFormat,
      generatorConfig[selectedGenerator],
    );
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []); // No deps; only calculated once at startup.

  // Update preview if inputs change:
  const debounceTimeoutRef = useRef<NodeJS.Timeout | null>(null);
  useEffect(() => {
    if (debounceTimeoutRef.current) {
      clearTimeout(debounceTimeoutRef.current);
    }
    debounceTimeoutRef.current = setTimeout(() => {
      if (!viewerRef.current || !monacoRef.current) {
        return;
      }
      const model = viewerRef.current.getModel();
      if (!model) {
        return;
      }
      monacoRef.current.editor.setModelLanguage(
        model,
        generatorProps[selectedGenerator].editor_lang,
      );
      try {
        const new_content = generatorProps[selectedGenerator].generate(
          editorContent,
          listingFormat,
          generatorConfig[selectedGenerator],
        );
        viewerRef.current.setValue(new_content);
        setAlertMsg(null);
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
      } catch (e: any) {
        console.error(e);
        setAlertMsg(e);
      }
    }, 100);
  }, [
    viewerRef,
    selectedGenerator,
    editorContent,
    listingFormat,
    generatorConfig,
  ]);

  return (
    <div className="h-full w-full relative">
      {alertMsg && (
        <div className="absolute bottom-4 left-4 right-4 z-50">
          <Alert variant="destructive">
            <AlertCircle className="h-4 w-4" />
            <AlertTitle>Error!</AlertTitle>
            <AlertDescription>{alertMsg}</AlertDescription>
          </Alert>
        </div>
      )}
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
