import { ChevronDown, ChevronUp } from "lucide-react";
import { JSX, useState, useEffect } from "react";
import { ScrollArea } from "@/components/ui/scroll-area";

export function CollapsibleMenu({
  title,
  children,
  defaultHeight = 400,
  minHeight = 100,
  maxHeight = 600,
}: {
  title: string;
  children: JSX.Element;
  defaultHeight?: number;
  minHeight?: number;
  maxHeight?: number;
}) {
  const [isCollapsed, setIsCollapsed] = useState(false);
  const [height, setHeight] = useState(defaultHeight);
  const [isResizing, setIsResizing] = useState(false);

  // Handle mouse events for resizing
  useEffect(() => {
    const handleMouseMove = (e: MouseEvent) => {
      if (!isResizing) return;

      const newHeight = window.innerHeight - e.clientY;
      setHeight(Math.min(Math.max(newHeight, minHeight), maxHeight));
    };

    const handleMouseUp = () => {
      setIsResizing(false);
    };

    if (isResizing) {
      document.addEventListener("mousemove", handleMouseMove);
      document.addEventListener("mouseup", handleMouseUp);
    }

    return () => {
      document.removeEventListener("mousemove", handleMouseMove);
      document.removeEventListener("mouseup", handleMouseUp);
    };
  }, [isResizing, minHeight, maxHeight]);

  return (
    <div className="border-t w-full mx-auto relative">
      {/* Resize handle */}
      {!isCollapsed && (
        <div
          className="absolute top-0 left-0 right-0 h-1 cursor-ns-resize hover:bg-primary/20"
          onMouseDown={() => setIsResizing(true)}
        />
      )}

      <div
        className="p-2 flex justify-between items-center cursor-pointer hover:bg-muted w-full"
        onClick={() => setIsCollapsed(!isCollapsed)}
      >
        <span className="font-medium">{title}</span>
        {isCollapsed ? <ChevronDown size={18} /> : <ChevronUp size={18} />}
      </div>

      {!isCollapsed && (
        <div
          className="w-full"
          style={{
            height: height - 40, // Subtract header height
          }}
        >
          <ScrollArea className="h-full w-full">
            <div className="p-4 space-y-4">{children}</div>
          </ScrollArea>
        </div>
      )}
    </div>
  );
}
