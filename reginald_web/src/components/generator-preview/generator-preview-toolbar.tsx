import { Check, Copy, Download } from "lucide-react";
import { Button } from "@/components/ui/button";
import { toast } from "sonner";
import { useState } from "react";
import { useGeneratorPreviewContext } from "./generator-preview-context";

export function GeneratorPreviewToolbar() {
  let { viewerRef } = useGeneratorPreviewContext();

  const [isCopied, setIsCopied] = useState(false);

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
    a.download = "reginald.c"; // TODO: EXTENSION
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

  return (
    <div className="flex items-center space-x-1">
      {/* Copy */}
      <Button
        variant="ghost"
        className="h-6 w-6 p-0.5"
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
        className="h-6 w-6 p-0.5"
        size="icon"
        aria-label="Save"
        onClick={downloadCode}
      >
        <Download className="h-4 w-4" />
      </Button>
    </div>
  );
}
