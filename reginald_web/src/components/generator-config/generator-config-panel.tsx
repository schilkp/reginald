import { useGeneratorConfigContext } from "./generator-config-context";
import { generatorProps } from "@/reginald/generators";
import { renderOption } from "./generator-config-renderer";

export function GeneratorConfigPanel() {
  let { generatorConfig, setGeneratorConfig, selectedGenerator } =
    useGeneratorConfigContext();

  let props = generatorProps[selectedGenerator];

  let configElems = props.options.map((opt) =>
    renderOption(opt, selectedGenerator, generatorConfig, setGeneratorConfig),
  );

  return (
    <div className="space-y-3">
      <div>
        <p className="text-medium font-medium">{props.title}</p>
        <p className="text-medium text-muted-foreground">{props.description}</p>
      </div>
      {configElems}
    </div>
  );
}
