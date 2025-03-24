import { useState } from "react";
//import { Header } from "@/components/header";
import { EditorPanel } from "@/components/editor/editor-panel";
import { CodePanel } from "@/components/code/code-panel";
import { exampleYaml } from "./components/editor/exampleYaml";

import { Mosaic, MosaicWindow } from "react-mosaic-component";

import "react-mosaic-component/react-mosaic-component.css";
import "@blueprintjs/core/lib/css/blueprint.css";
import "@blueprintjs/icons/lib/css/blueprint-icons.css";

export type View = {
  title: string;
  content: JSX.Element;
};

function App() {
  const [editorContent, setEditorContent] = useState<string>(exampleYaml);
  const [selectedLanguage, setSelectedLanguage] = useState<"yaml" | "json">(
    "yaml",
  );

  const ELEMENT_MAP: { [viewId: string]: View } = {
    editor: {
      title: "Listing Editor",
      content: (
        <EditorPanel
          setEditorContent={setEditorContent}
          selectedLanguage={selectedLanguage}
          setSelectedLanguage={setSelectedLanguage}
        />
      ),
    },
    code: {
      title: "Code Preview",
      content: (
        <CodePanel
          editorContent={editorContent}
          selectedLanguage={selectedLanguage}
        />
      ),
    },
  };

  return (
    <div className="h-screen w-full">
      <Mosaic<string>
        renderTile={(id, path) => (
          <MosaicWindow<string> path={path} title={ELEMENT_MAP[id].title}>
            {ELEMENT_MAP[id].content}
          </MosaicWindow>
        )}
        initialValue={{
          direction: "row",
          first: "editor",
          second: "code",
        }}
      />
    </div>
  );
}
export default App;
