import { EditorPanel } from "@/components/editor/editor-panel";
import { EditorContextProvider } from "./components/editor/editor-context";
import { EditorToolbar } from "./components/editor/editor-toolbar";

import { GeneratorPreviewPanel } from "./components/generator-preview/generator-preview-panel";
import { GeneratorPreviewToolbar } from "./components/generator-preview/generator-preview-toolbar";
import { GeneratorPreviewContextProvider } from "./components/generator-preview/generator-preview-context";

import { GeneratorConfigContextProvider } from "./components/generator-config/generator-config-context";
import { GeneratorConfigToolbar } from "./components/generator-config/generator-config-toolbar";
import { GeneratorConfigPanel } from "./components/generator-config/generator-config-panel";

import { Mosaic, MosaicWindow } from "react-mosaic-component";
import "react-mosaic-component/react-mosaic-component.css";
import "./mosaic-custom.css";
import { Header } from "./components/header";

export type View = {
  title: string;
  panel: JSX.Element;
  toolbar: JSX.Element | null;
};

function App() {
  const ELEMENT_MAP: { [viewId: string]: View } = {
    editor: {
      title: "Listing Editor",
      panel: <EditorPanel />,
      toolbar: <EditorToolbar />,
    },
    preview: {
      title: "Generator Preview",
      panel: <GeneratorPreviewPanel />,
      toolbar: <GeneratorPreviewToolbar />,
    },
    generator_config: {
      title: "Generator Config",
      panel: <GeneratorConfigPanel />,
      toolbar: <GeneratorConfigToolbar />,
    },
  };

  return (
    <GeneratorPreviewContextProvider>
      <EditorContextProvider>
        <GeneratorConfigContextProvider>
          <div className="h-screen w-full">
            <Header />
            <Mosaic<string>
              renderTile={(id, path) => (
                <MosaicWindow<string>
                  path={path}
                  title={ELEMENT_MAP[id].title}
                  toolbarControls={ELEMENT_MAP[id].toolbar}
                >
                  {ELEMENT_MAP[id].panel}
                </MosaicWindow>
              )}
              initialValue={{
                direction: "row",
                first: "editor",
                second: {
                  direction: "column",
                  first: "preview",
                  second: "generator_config",
                },
              }}
            />
          </div>
        </GeneratorConfigContextProvider>
      </EditorContextProvider>
    </GeneratorPreviewContextProvider>
  );
}
export default App;
