import { useGeneratorConfigContext } from "./generator-config-context";
import { generatorProps } from "@/reginald/generators";
import { renderOption } from "./generator-config-renderer";
import { ScrollArea } from "../ui/scroll-area";
import { Separator } from "../ui/separator";

export function GeneratorConfigPanel() {
  const { generatorConfig, setGeneratorConfig, selectedGenerator } =
    useGeneratorConfigContext();

  const props = generatorProps[selectedGenerator];

  const configElems = props.options.map((opt) =>
    renderOption(opt, selectedGenerator, generatorConfig, setGeneratorConfig),
  );

  return (
    <ScrollArea className="h-full bg-white px-4">
      <div className="space-y-3 pt-3 pb-3">
        <div>
          {/* Title */}
          <p className="text-medium font-medium">{props.title}</p>
          <p className="text-medium text-muted-foreground">
            {props.description}
          </p>
        </div>
        <Separator />
        <div className="space-y-3">
          {/* Config Elems */}
          {configElems}
        </div>
      </div>
    </ScrollArea>
  );
}
