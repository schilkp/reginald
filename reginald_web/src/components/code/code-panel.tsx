import { Check, Copy, Download } from "lucide-react";
import { Button } from "@/components/ui/button";
import { CodeOutput } from "./code-viewer";
import { JSX, useState, useRef, useEffect } from "react";
import { GeneratorSettingsCFunpack } from "./generators/c_funcpack";
import { GeneratorSelecetor } from "./generator-select";
import { toast } from "sonner";
import type * as monaco from "monaco-editor";
import { GeneratorConfig } from "./generators/config";
import {
  useConfigState as useCFuncpackConfig,
  Config as cFuncpackConfig,
} from "./generators/c_funcpack_config";
import * as wasm from "reginald_wasm";
import { useEditorContext } from "../editor/editor-context";

const run_c_funcpack = (
  selectedGenerator: string,
  editorContent: string,
  selectedLanguage: "yaml" | "json",
  c_funcpack_config: cFuncpackConfig,
) => {
  if (selectedGenerator !== "c.funcpack") {
    return null;
  }

  const opts: wasm.CFuncpackOpts = new wasm.CFuncpackOpts();
  switch (c_funcpack_config.Endianess) {
    case "le":
      opts.endianess = wasm.EndianessImpl.Little;
      break;
    case "be":
      opts.endianess = wasm.EndianessImpl.Big;
      break;
    default:
    case "both":
      opts.endianess = wasm.EndianessImpl.Both;
      break;
  }
  switch (c_funcpack_config.DeferToEndian) {
    case "le":
      opts.defer_to_endianess = wasm.Endianess.Little;
      break;
    case "be":
      opts.defer_to_endianess = wasm.Endianess.Big;
      break;
    default:
    case null:
      opts.defer_to_endianess = null;
      break;
  }
  opts.registers_as_bitfields = c_funcpack_config.RegistersAsBitfields;
  opts.max_enum_bitwidth = c_funcpack_config.MaxEnumBitwidth;
  opts.funcs_static_inline = c_funcpack_config.FuncsStaticInline;
  opts.funcs_as_prototypes = c_funcpack_config.FuncsAsPrototypes;
  opts.clang_format_guard = c_funcpack_config.ClangFormatGuard;
  opts.include_guards = c_funcpack_config.IncludeGuards;
  opts.gen_enums = c_funcpack_config.GenerateEnums;
  opts.gen_enum_validation = c_funcpack_config.GenerateEnumValidationMacros;
  opts.gen_structs = c_funcpack_config.GenerateStructs;
  opts.gen_struct_conv = c_funcpack_config.GenerateStructConversionFuncs;
  opts.gen_reg_properties = c_funcpack_config.GenerateRegisterProperties;
  opts.gen_generics = c_funcpack_config.GenerateGenericMacros;

  for (const include of c_funcpack_config.Includes) {
    opts.add_include_push(include);
  }
  const format =
    selectedLanguage === "yaml"
      ? wasm.ListingFormat.Yaml
      : wasm.ListingFormat.Json;

  try {
    const new_content = wasm.run(editorContent, format, opts);
    return new_content;
  } catch (e) {
    console.error("Failed to run c.funcpack: " + e);
    toast.error("Failed to run c.funcpack: " + e);
    return null;
  }
};

export type OutputGenerator = {
  title: string;
  description: string;
  editor_lang: string;
  file_extension: string;
  config_panel: JSX.Element;
  config: GeneratorConfig<object>;
};

export function CodePanel() {
  const [selectedGenerator, setSelectedGenerator] = useState("c.funcpack");
  const viewerRef = useRef<monaco.editor.IStandaloneCodeEditor | null>(null);
  const [isCopied, setIsCopied] = useState(false);

  const { listingFormat: selectedLanguage, editorContent } = useEditorContext();

  const languages: Record<string, string> = {
    c: "C",
    rs: "Rust",
  };

  const c_funcpack_config = useCFuncpackConfig();
  const c_macromacp_config = useCFuncpackConfig();
  const rs_structs_config = useCFuncpackConfig();

  const generators: Record<string, OutputGenerator> = {
    "c.funcpack": {
      title: "c.funcpack",
      description: "C register structs with packing/unpacking functions",
      editor_lang: "c",
      file_extension: "c",
      config_panel: <GeneratorSettingsCFunpack config={c_funcpack_config} />,
      config: c_funcpack_config,
    },
    "c.macromap": {
      title: "c.macromap",
      description: "C field mask/shift macros",
      editor_lang: "c",
      file_extension: "c",
      config_panel: <GeneratorSettingsCFunpack config={c_macromacp_config} />,
      config: c_macromacp_config,
    },
    "rs.structs": {
      title: "rs.structs",
      description: "Rust module with register structs and no dependencies",
      editor_lang: "rust",
      file_extension: "rs",
      config_panel: <GeneratorSettingsCFunpack config={rs_structs_config} />,
      config: rs_structs_config,
    },
  };

  const downloadCode = () => {
    if (!viewerRef.current) {
      toast.error("Editor is not ready.");
      return;
    }
    const content = viewerRef.current.getValue();
    const blob = new Blob([content], { type: "text" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = "reginald." + generators[selectedGenerator].file_extension;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  };

  const copyToClipboard = async () => {
    if (!viewerRef.current) {
      toast.error("Editor is not ready.");
      return;
    }
    const content = viewerRef.current.getValue();
    try {
      await navigator.clipboard.writeText(content);
      setIsCopied(true);

      setTimeout(() => {
        setIsCopied(false);
      }, 500);
    } catch {
      toast.error("Failed to copy to clipboard.");
    }
  };

  useEffect(() => {
    const new_content = run_c_funcpack(
      selectedGenerator,
      editorContent,
      selectedLanguage,
      c_funcpack_config,
    );
    if (new_content && viewerRef.current) {
      viewerRef.current.setValue(new_content);
    }
  }, [
    selectedGenerator,
    c_funcpack_config,
    selectedLanguage,
    editorContent,
    viewerRef,
  ]);

  const initial_value =
    run_c_funcpack(
      selectedGenerator,
      editorContent,
      selectedLanguage,
      c_funcpack_config,
    ) || "";

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
          <Button
            variant="ghost"
            size="icon"
            aria-label="Copy to clipboard"
            onClick={copyToClipboard}
          >
            {isCopied ? (
              <Check className="h-4 w-4" />
            ) : (
              <Copy className="h-4 w-4" />
            )}
          </Button>
          {/* Save */}
          <Button
            variant="ghost"
            size="icon"
            aria-label="Save"
            onClick={downloadCode}
          >
            <Download className="h-4 w-4" />
          </Button>
        </div>
      </div>
      {/* Code Preview */}
      <div className="flex-1 overflow-hidden">
        <CodeOutput value={initial_value} language="c" viewerRef={viewerRef} />
      </div>
      {/* Generator Settings */}
      <CollapsibleMenu title="Generator Settings">
        <div className="w-full">
          <div className="mb-6">
            <h1 className="text-xl font-bold tracking-tight">
              {generators[selectedGenerator].title}
            </h1>
            <p className="text-muted-foreground">
              {generators[selectedGenerator].description}
            </p>
          </div>
          {generators[selectedGenerator].config_panel}
        </div>
      </CollapsibleMenu>
    </div>
  );
}
