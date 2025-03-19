import { Panel } from "@/App"
import {
    ToggleGroup,
    ToggleGroupItem,
} from "@/components/ui/toggle-group"


export function Toolbar({ panels }: { panels: Record<string, Panel> }) {
    const defaultValues = Object.keys(panels)
        .filter(key => panels[key].visible)
        .map(key => panels[key].title);

    const items = Object.keys(panels).map(panel_key => {
        const panel = panels[panel_key];
        return <ToggleGroupItem
            key={panel_key}
            value={panel.title}
            aria-label={"Toggle " + panel.title}
            onClick={() => panel.setVisible(!panel.visible)}
        >
            {panel.title}
        </ToggleGroupItem>
    }
    );

    return (
        <div className="border-b p-2 flex items-center justify-between bg-background">
            <h1 className="text-xl font-bold">Reginald</h1>
            <div className="flex items-center space-x-2">
                <ToggleGroup
                    variant="outline"
                    type="multiple"
                    defaultValue={defaultValues}
                >
                    {items}
                </ToggleGroup>
            </div>
        </div>
    )
}
