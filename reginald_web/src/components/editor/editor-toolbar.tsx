import { Check, Code, Copy, Download, FileJson, Upload } from "lucide-react";
import { Button } from "@/components/ui/button";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";
import { toast } from "sonner";
import { useEditorContext } from "./editor-context";
import { useState } from "react";

export function EditorToolbar() {
  let {
    editorRef,
    listingFormat: selectedLanguage,
    setListingFormat: setSelectedLanguage,
  } = useEditorContext();

  let [isCopied, setIsCopied] = useState(false);

  const downloadListing = () => {
    if (!editorRef.current) {
      toast.error("Editor is not ready.");
      return;
    }
    const content = editorRef.current.getValue();
    const type = selectedLanguage === "yaml" ? "text/yaml" : "application/json";
    const blob = new Blob([content], { type: type });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download =
      selectedLanguage === "yaml" ? "reginald_map.yaml" : "reginald_map.json";
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  };

  const uploadListing = (e: React.ChangeEvent<HTMLInputElement>) => {
    if (!editorRef.current) {
      toast.error("Editor is not ready.");
      return;
    }
    const file = e.target.files?.[0];
    if (file) {
      const reader = new FileReader();
      reader.onload = (e) => {
        const content = e.target?.result as string;
        editorRef.current?.setValue(content);
      };
      reader.readAsText(file);
    } else {
      toast.error("Invalid file.");
    }
  };

  const copyToClipboard = async () => {
    if (!editorRef.current) {
      toast.error("Editor is not ready.");
      return;
    }
    const content = editorRef.current.getValue();
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

  return (
    <div className="flex items-center space-x-1 pr-1">
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
      {/* Upload */}
      <Button
        variant="ghost"
        size="icon"
        aria-label="Upload"
        onClick={() => document.getElementById("upload-listing")?.click()}
      >
        <Upload className="h-4 w-4" />
      </Button>
      <input
        id="upload-listing"
        type="file"
        accept=".yaml,.yml,.json,.hjson"
        className="hidden"
        onChange={uploadListing}
      />
      {/* Save */}
      <Button
        variant="ghost"
        size="icon"
        aria-label="Save"
        onClick={downloadListing}
      >
        <Download className="h-4 w-4" />
      </Button>
      {/* File type select */}
      <ToggleGroup
        variant="outline"
        type="single"
        defaultValue="yaml"
        value={selectedLanguage}
        onValueChange={(value) => {
          if (value !== null && (value === "yaml" || value === "json")) {
            setSelectedLanguage(value);
          }
        }}
      >
        <ToggleGroupItem value="yaml" aria-label="YAML editor">
          <Code /> YAML
        </ToggleGroupItem>
        <ToggleGroupItem value="json" aria-label="JSON editor">
          <FileJson /> JSON
        </ToggleGroupItem>
      </ToggleGroup>
    </div>
  );
}
