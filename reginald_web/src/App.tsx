import { useState, Dispatch, SetStateAction } from "react";
import { Header } from "@/components/header";
import { EditorPanel } from "@/components/editor/editor-panel";
import { CodePanel } from "@/components/code/code-panel";
import { exampleYaml } from "./components/editor/exampleYaml";

export type Panel = {
  title: string;
  visible: boolean;
  setVisible: Dispatch<SetStateAction<boolean>>;
};

function App() {
  const [editorVisible, setEditorVisible] = useState(true);
  const [codeViewerVisible, setCodeViewerVisible] = useState(true);
  const panels = {
    editor: {
      title: "Editor",
      visible: editorVisible,
      setVisible: setEditorVisible,
    },
    code: {
      title: "Output",
      visible: codeViewerVisible,
      setVisible: setCodeViewerVisible,
    },
  };

  const [editorContent, setEditorContent] = useState<string>(exampleYaml);
  const [selectedLanguage, setSelectedLanguage] = useState<"yaml" | "json">(
    "yaml",
  );

  return (
    <div className="flex flex-col h-screen">
      <Header panels={panels} />

      {/* TODO: The scaling here is very hacky/hardcoded to two panels */}
      <div className="flex flex-1 overflow-hidden">
        <div
          className={`h-full ${panels.code.visible ? "w-1/2" : "w-full"}`}
          style={{
            display: panels.editor.visible ? "block" : "none",
          }}
        >
          <EditorPanel
            setEditorContent={setEditorContent}
            selectedLanguage={selectedLanguage}
            setSelectedLanguage={setSelectedLanguage}
          />
        </div>
        <div
          className={`h-full ${panels.editor.visible ? "w-1/2" : "w-full"}`}
          style={{
            display: panels.code.visible ? "block" : "none",
          }}
        >
          <CodePanel
            editorContent={editorContent}
            selectedLanguage={selectedLanguage}
          />
        </div>
      </div>
    </div>
  );
}
export default App;
