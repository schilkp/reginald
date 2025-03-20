import { Code, Copy, Download, FileJson, Upload } from "lucide-react";
import { Button } from "@/components/ui/button";
import { ListingEditor } from "./listing-editor";
import { exampleYaml } from "./exampleYaml";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";
import { useState } from "react";

export function EditorPanel() {
  const [selectedLanguage, setSelectedLanguage] = useState("yaml");

  return (
    <div className="flex flex-col border-r w-full h-full">
      {/* Toolbar */}
      <div className="p-2 flex items-center justify-between bg-background">
        <h2 className="text-m">Register Listing</h2>
        <div className="flex items-center space-x-1">
          {/* Copy */}
          <Button variant="ghost" size="icon" aria-label="Copy to clipboard">
            <Copy className="h-4 w-4" />
          </Button>
          {/* Upload */}
          <Button variant="ghost" size="icon" aria-label="Upload">
            <Upload className="h-4 w-4" />
          </Button>
          {/* Save */}
          <Button variant="ghost" size="icon" aria-label="Save">
            <Download className="h-4 w-4" />
          </Button>

          {/* File type select */}
          <ToggleGroup
            variant="outline"
            type="single"
            defaultValue="yaml"
            value={selectedLanguage}
            onValueChange={(value) => {
              setSelectedLanguage(value);
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
      </div>
      {/* Editor */}
      <div className="flex-1 overflow-hidden">
        <ListingEditor value={exampleYaml} selectedLanguage={selectedLanguage} />
      </div>
    </div>
  );
}
