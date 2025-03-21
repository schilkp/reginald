import type React from "react";

import { useState } from "react";
import { X, Plus, Trash } from "lucide-react";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Label } from "@/components/ui/label";

interface StringListManagerProps {
  id: string;
  content: string[];
  onContentChange(content: string[]): void;
  ghost_text: string;
  label: string;
  description?: string;
}

export default function ConfigStringListManager({
  id,
  content,
  onContentChange: onContentChanged,
  ghost_text,
  label,
  description,
}: StringListManagerProps) {
  const [newString, setNewString] = useState("");

  const addString = () => {
    if (newString.trim() !== "") {
      onContentChanged([...content, newString.trim()]);
      setNewString("");
    }
  };

  const clearStrings = () => {
    onContentChanged([]);
  };

  const removeString = (index: number) => {
    onContentChanged(content.filter((_, i) => i !== index));
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter") {
      addString();
    }
  };

  return (
    <>
      <div className="grid gap-1.5">
        <div className="grid gap-1.5 leading-none">
          <Label htmlFor={id} className="font-medium">
            {label}
          </Label>
          {description && description !== "" && (
            <p className="text-sm text-muted-foreground">{description}</p>
          )}
        </div>

        <div className="flex flex-wrap gap-2 mb-2">
          {content.map((string, index) => (
            <Badge
              key={index}
              variant="secondary"
              className="flex items-center gap-1 px-3 py-1.5 text-sm"
            >
              {string}
              <Button
                variant="ghost"
                size="icon"
                className="h-4 w-4 p-0 ml-1 text-muted-foreground hover:text-foreground"
                onClick={() => removeString(index)}
              >
                <X className="h-3 w-3" />
                <span className="sr-only">Remove {string}</span>
              </Button>
            </Badge>
          ))}
        </div>

        <div className="flex gap-2">
          <Input
            id={id}
            value={newString}
            onChange={(e) => setNewString(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder={ghost_text}
            className="flex-1 focus-visible:ring-0"
          />
          <Button onClick={addString} size="icon">
            <Plus className="h-4 w-4" />
            <span className="sr-only">Add</span>
          </Button>
          <Button onClick={clearStrings} size="icon">
            <Trash className="h-4 w-4" />
            <span className="sr-only">Clear</span>
          </Button>
        </div>
      </div>
    </>
  );
}
