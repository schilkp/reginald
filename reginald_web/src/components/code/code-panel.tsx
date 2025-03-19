import { CollapsibleMenu } from "../custom/collapsible-menu";
import CodeOutput from "./code-viewer";

export function CodePanel() {

    const content = <div> editor content </div>;

    return (
        <div className="flex flex-col border-r w-full h-full">
            {/* Toolbar */}
            <div className="p-2 flex items-center justify-between bg-background">
                <h2 className="text-m">Output Preview</h2>
            </div>
            {/* Code Preview */}
            <div className="flex-1 overflow-hidden">
                <CodeOutput value="hello, world!" language="c" />
            </div>
            {/* Settings */}
            <CollapsibleMenu content={content} title="Options" />
        </div>
    )
}
