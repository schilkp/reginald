import { ChevronDown, ChevronUp } from "lucide-react";
import { JSX, useState } from "react";

export function CollapsibleMenu({
  title,
  children,
}: {
  title: string;
  children: JSX.Element;
}) {
  const [isCollapsed, setIsCollapsed] = useState(false);

  return (
    <div className="border-t">
      <div
        className="p-2 flex justify-between items-center cursor-pointer hover:bg-muted"
        onClick={() => setIsCollapsed(!isCollapsed)}
      >
        <span className="font-medium">{title}</span>
        {isCollapsed ? <ChevronDown size={18} /> : <ChevronUp size={18} />}
      </div>
      <div
        className="p-4 space-y-4"
        style={{
          display: isCollapsed ? "none" : "block",
        }}
      >
        {children}
      </div>
    </div>
  );
}
