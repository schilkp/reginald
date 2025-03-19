import { ChevronDown, ChevronUp } from "lucide-react";
import { JSX, useState } from "react";

export function CollapsibleMenu({ content, title }: { content: JSX.Element, title: string }) {
    const [collapsedPanel, setCollapsedPanel] = useState(true)

    return (
        <div className="border-t">
            <div className="p-2 flex justify-between items-center cursor-pointer hover:bg-muted" onClick={() => setCollapsedPanel(!collapsedPanel)}>
                <span className="font-medium">{title}</span>
                {collapsedPanel ? <ChevronDown size={18} /> : <ChevronUp size={18} />}
            </div>
            {!collapsedPanel && (
                <div className="p-4 space-y-4">
                    {content}
                </div>
            )}
        </div>
    );
}

