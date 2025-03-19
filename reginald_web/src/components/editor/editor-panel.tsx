import ListingEditor, { exampleYaml } from "./listing-editor";

export function EditorPanel() {
    return (
        <div className="flex flex-col border-r w-full h-full">
            {/* Toolbar */}
            <div className="p-2 flex items-center justify-between bg-background">
                <h2 className="text-m">Register Listing</h2>
            </div>
            {/* Editor */}
            <div className="flex-1 overflow-hidden">
                <ListingEditor value={exampleYaml} language="yaml" />
            </div>
        </div>
    )
}
