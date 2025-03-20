import { Copy, Download } from "lucide-react";
import { CollapsibleMenu } from "@/components/custom/collapsible-menu";
import { Button } from "@/components/ui/button";
import { CodeOutput } from "./code-viewer";
import { example_code } from "./example_code";
import { JSX, useState } from "react";
import { GeneratorSettingsCFunpack } from "./generators/c_funcpack";
import { GeneratorSettingsCMacroMap } from "./generators/c_macromacp";
import { GeneratorSelecetor } from "./generator-select";
import { GeneratorConfigCard } from "./generator-config-card";

export type OutputGenerator = {
  title: string;
  description: string;
  editor_lang: string;
  config_panel: JSX.Element;
};

export function CodePanel() {
  const [selectedGenerator, setSelectedGenerator] = useState("c.funcpack");

  const languages: Record<string, string> = {
    c: "C",
    rs: "Rust",
  };

  const generators: Record<string, OutputGenerator> = {
    "c.funcpack": {
      title: "c.funcpack",
      description:
        "C header with register structs, and packing/unpacking functions",
      editor_lang: "c",
      config_panel: <GeneratorSettingsCFunpack />,
    },
    "c.macromap": {
      title: "c.macromap",
      description: "C header with field mask/shift macros",
      editor_lang: "c",
      config_panel: <GeneratorSettingsCMacroMap />,
    },
    "rs.structs": {
      title: "rs.structs",
      description: "Rust module with register structs and no dependencies",
      editor_lang: "rust",
      config_panel: <GeneratorSettingsCMacroMap />,
    },
  };

  return (
    <div className="flex flex-col border-r w-full h-full">
      {/* Toolbar */}
      <div className="p-2 flex items-center justify-between bg-background">
        <h2 className="text-m">Output Preview</h2>
        <div className="flex items-center space-x-1">
          {/* Generator Selector */}
          <GeneratorSelecetor
            generators={generators}
            languages={languages}
            selectedGenerator={selectedGenerator}
            setSelectedGenerator={setSelectedGenerator}
          />
          {/* Copy */}
          <Button variant="ghost" size="icon" aria-label="Copy to clipboard">
            <Copy className="h-4 w-4" />
          </Button>
          {/* Save */}
          <Button variant="ghost" size="icon" aria-label="Save">
            <Download className="h-4 w-4" />
          </Button>
        </div>
      </div>
      {/* Code Preview */}
      <div className="flex-1 overflow-hidden">
        <CodeOutput value={example_code} language="c" />
      </div>
      {/* Generator Settings */}
      <CollapsibleMenu title="Generator Settings">
        <GeneratorConfigCard
          generators={generators}
          selectedGenerator={selectedGenerator}
        />
      </CollapsibleMenu>
    </div>
  );
}
