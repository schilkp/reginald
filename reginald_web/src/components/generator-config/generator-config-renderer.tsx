import { ReginaldGenerator } from "@/reginald/generators";
import { GeneratorConfigRegistry } from "./generator-config-context";
import { ConfigCheckbox } from "../code/generators_config_ui/checkbox";
import { GeneratorOption } from "@/reginald/generators-option";
import { Separator } from "../ui/separator";
import ConfigStringListManager from "../code/generators_config_ui/string-list-manager";

//===----------------------------------------------------------------------===//
// Utils
//===----------------------------------------------------------------------===//

function existsCheck(
  generator: ReginaldGenerator,
  id: string,
  generatorConfig: GeneratorConfigRegistry,
): void {
  let specificConfig = generatorConfig[generator];
  if (!(id in specificConfig)) {
    let msg = "generatorConifg[" + generator + "] does not contain " + id;
    console.error(msg);
    throw msg;
  }
}

//===----------------------------------------------------------------------===//
// Booleans
//===----------------------------------------------------------------------===//

function booleanTypeCheck(
  generator: ReginaldGenerator,
  id: string,
  generatorConfig: GeneratorConfigRegistry,
): void {
  existsCheck(generator, id, generatorConfig);
  // @ts-ignore: Checked dynamically.
  let value = generatorConfig[generator][id];
  if (!(typeof value === "boolean")) {
    let msg = "generatorConifg[" + generator + "][" + id + "] is not a boolean";
    console.error(msg);
    throw msg;
  }
}

function getBooleanOption(
  generator: ReginaldGenerator,
  id: string,
  generatorConfig: GeneratorConfigRegistry,
): boolean {
  booleanTypeCheck(generator, id, generatorConfig);
  // @ts-ignore: Checked dynamically.
  return generatorConfig[generator][id] as boolean;
}

function setBooleanOption(
  generator: ReginaldGenerator,
  id: string,
  val: boolean,
  generatorConfig: GeneratorConfigRegistry,
  setGeneratorConfig: (a: GeneratorConfigRegistry) => void,
): void {
  const newConfig = JSON.parse(JSON.stringify(generatorConfig));
  booleanTypeCheck(generator, id, generatorConfig);
  // @ts-ignore: Checked dynamically.
  newConfig[generator][id] = val;
  setGeneratorConfig(newConfig);
}

//===----------------------------------------------------------------------===//
// String Lists
//===----------------------------------------------------------------------===//

function stringListTypeCheck(
  generator: ReginaldGenerator,
  id: string,
  generatorConfig: GeneratorConfigRegistry,
): void {
  existsCheck(generator, id, generatorConfig);
  // @ts-ignore: Checked dynamically.
  let value = generatorConfig[generator][id];
  if (
    !(Array.isArray(value) && value.every((item) => typeof item === "string"))
  ) {
    let msg =
      "generatorConifg[" + generator + "][" + id + "] is not a list of strings";
    console.error(msg);
    throw msg;
  }
}

function getStringListOption(
  generator: ReginaldGenerator,
  id: string,
  generatorConfig: GeneratorConfigRegistry,
): string[] {
  stringListTypeCheck(generator, id, generatorConfig);
  // @ts-ignore: Checked dynamically.
  return generatorConfig[generator][id] as string[];
}

function setStringListOption(
  generator: ReginaldGenerator,
  id: string,
  val: string[],
  generatorConfig: GeneratorConfigRegistry,
  setGeneratorConfig: (a: GeneratorConfigRegistry) => void,
): void {
  const newConfig = JSON.parse(JSON.stringify(generatorConfig));
  stringListTypeCheck(generator, id, generatorConfig);
  // @ts-ignore: Checked dynamically.
  newConfig[generator][id] = val;
  setGeneratorConfig(newConfig);
}

//===----------------------------------------------------------------------===//
// Render
//===----------------------------------------------------------------------===//

export function renderOption(
  option: GeneratorOption,
  generator: ReginaldGenerator,
  generatorConfig: GeneratorConfigRegistry,
  setGeneratorConfig: (a: GeneratorConfigRegistry) => void,
) {
  switch (option.kind) {
    case "header":
      return (
        <div key={generator + "_" + option.id} id={generator + "_" + option.id}>
          <p className="text-sm font-medium">{option.title}</p>
          {option.subtitle && (
            <p className="text-sm text-muted-foreground">{option.subtitle}</p>
          )}
        </div>
      );

    case "separator":
      return (
        <Separator
          key={generator + "_" + option.id}
          id={generator + "_" + option.id}
        />
      );

    case "checkbox":
      return (
        <ConfigCheckbox
          key={generator + "_" + option.id}
          id={generator + "_" + option.id}
          checked={getBooleanOption(generator, option.id, generatorConfig)}
          onCheckedChange={(checked: boolean) =>
            setBooleanOption(
              generator,
              option.id,
              checked,
              generatorConfig,
              setGeneratorConfig,
            )
          }
          label={option.label}
          description={option.description}
        />
      );

    case "string-manager":
      return (
        <ConfigStringListManager
          key={generator + "_" + option.id}
          id={generator + "_" + option.id}
          content={getStringListOption(generator, option.id, generatorConfig)}
          onContentChange={(val) =>
            setStringListOption(
              generator,
              option.id,
              val,
              generatorConfig,
              setGeneratorConfig,
            )
          }
          ghost_text={option.ghost_text}
          label={option.label}
          description={option.description}
        />
      );
  }
}
