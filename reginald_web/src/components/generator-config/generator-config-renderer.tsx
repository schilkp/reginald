import { ReginaldGenerator } from "@/reginald/generators";
import { GeneratorConfigRegistry } from "./generator-config-context";
import { GeneratorOption } from "@/reginald/generators-option";
import { Separator } from "@/components/ui/separator";
import { ConfigCheckbox } from "./config-elems/checkbox";
import ConfigStringListManager from "./config-elems/string-list-manager";
import { ConfigNumericInput } from "./config-elems/numeric-input";
import { ConfigSelectSingle } from "./config-elems/select-single";
import { ConfigStringInput } from "./config-elems/string-input";
//===----------------------------------------------------------------------===//
// Utils
//===----------------------------------------------------------------------===//

function existsCheck(
  generator: ReginaldGenerator,
  id: string,
  generatorConfig: GeneratorConfigRegistry,
): void {
  const specificConfig = generatorConfig[generator];
  if (!(id in specificConfig)) {
    const msg = "generatorConfig[" + generator + "] does not contain " + id;
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
  // @ts-expect-error: Checked dynamically.
  const value = generatorConfig[generator][id];
  if (!(typeof value === "boolean")) {
    const msg =
      "generatorConfig[" + generator + "][" + id + "] is not a boolean";
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
  // @ts-expect-error: Checked dynamically.
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
  newConfig[generator][id] = val;
  setGeneratorConfig(newConfig);
}

//===----------------------------------------------------------------------===//
// Number
//===----------------------------------------------------------------------===//

function numberTypeCheck(
  generator: ReginaldGenerator,
  id: string,
  generatorConfig: GeneratorConfigRegistry,
): void {
  existsCheck(generator, id, generatorConfig);
  // @ts-expect-error: Checked dynamically.
  const value = generatorConfig[generator][id];
  if (!(typeof value === "number")) {
    const msg =
      "generatorConfig[" + generator + "][" + id + "] is not a number";
    console.error(msg);
    throw msg;
  }
}

function getNumberOption(
  generator: ReginaldGenerator,
  id: string,
  generatorConfig: GeneratorConfigRegistry,
): number {
  numberTypeCheck(generator, id, generatorConfig);
  // @ts-expect-error: Checked dynamically.
  return generatorConfig[generator][id] as number;
}

function setNumberOption(
  generator: ReginaldGenerator,
  id: string,
  val: number,
  generatorConfig: GeneratorConfigRegistry,
  setGeneratorConfig: (a: GeneratorConfigRegistry) => void,
): void {
  const newConfig = JSON.parse(JSON.stringify(generatorConfig));
  numberTypeCheck(generator, id, generatorConfig);
  newConfig[generator][id] = val;
  setGeneratorConfig(newConfig);
}

//===----------------------------------------------------------------------===//
// String
//===----------------------------------------------------------------------===//

function stringTypeCheck(
  generator: ReginaldGenerator,
  id: string,
  generatorConfig: GeneratorConfigRegistry,
): void {
  existsCheck(generator, id, generatorConfig);
  // @ts-expect-error: Checked dynamically.
  const value = generatorConfig[generator][id];
  if (!(typeof value === "string")) {
    const msg =
      "generatorConfig[" + generator + "][" + id + "] is not a string";
    console.error(msg);
    throw msg;
  }
}

function getStringOption(
  generator: ReginaldGenerator,
  id: string,
  generatorConfig: GeneratorConfigRegistry,
): string {
  stringTypeCheck(generator, id, generatorConfig);
  // @ts-expect-error: Checked dynamically.
  return generatorConfig[generator][id] as string;
}

function setStringOption(
  generator: ReginaldGenerator,
  id: string,
  val: string,
  generatorConfig: GeneratorConfigRegistry,
  setGeneratorConfig: (a: GeneratorConfigRegistry) => void,
): void {
  const newConfig = JSON.parse(JSON.stringify(generatorConfig));
  stringTypeCheck(generator, id, generatorConfig);
  newConfig[generator][id] = val;
  setGeneratorConfig(newConfig);
}

//===----------------------------------------------------------------------===//
// String Enum
//===----------------------------------------------------------------------===//

function enumTypeCheck(
  generator: ReginaldGenerator,
  id: string,
  generatorConfig: GeneratorConfigRegistry,
  validOptions: string[],
): void {
  existsCheck(generator, id, generatorConfig);
  // @ts-expect-error: Checked dynamically.
  const value = generatorConfig[generator][id];
  if (!(typeof value === "string")) {
    const msg =
      "generatorConfig[" +
      generator +
      "][" +
      id +
      "] is not a string (for enum)";
    console.error(msg);
    throw msg;
  }
  if (!validOptions.includes(value)) {
    const msg =
      "generatorConfig[" +
      generator +
      "][" +
      id +
      "] is not a valid enum option. Value:" +
      value +
      "  options: " +
      validOptions;
    console.error(msg);
    throw msg;
  }
}

function getEnumOption(
  generator: ReginaldGenerator,
  id: string,
  generatorConfig: GeneratorConfigRegistry,
  validOptions: string[],
): string {
  enumTypeCheck(generator, id, generatorConfig, validOptions);
  // @ts-expect-error: Checked dynamically.
  return generatorConfig[generator][id] as string;
}

function setEnumOption(
  generator: ReginaldGenerator,
  id: string,
  val: string,
  generatorConfig: GeneratorConfigRegistry,
  setGeneratorConfig: (a: GeneratorConfigRegistry) => void,
  validOptions: string[],
): void {
  const newConfig = JSON.parse(JSON.stringify(generatorConfig));
  enumTypeCheck(generator, id, generatorConfig, validOptions);
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
  // @ts-expect-error: Checked dynamically.
  const value = generatorConfig[generator][id];
  if (
    !(Array.isArray(value) && value.every((item) => typeof item === "string"))
  ) {
    const msg =
      "generatorConfig[" + generator + "][" + id + "] is not a list of strings";
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
  // @ts-expect-error: Checked dynamically.
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
          <p className="text-medium font-medium">{option.title}</p>
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

    case "numeric":
      return (
        <ConfigNumericInput
          key={generator + "_" + option.id}
          id={generator + "_" + option.id}
          value={getNumberOption(generator, option.id, generatorConfig)}
          onValueChange={(value: number) =>
            setNumberOption(
              generator,
              option.id,
              value,
              generatorConfig,
              setGeneratorConfig,
            )
          }
          label={option.label}
          description={option.description}
          min={option.min}
          max={option.max}
        />
      );

    case "string":
      return (
        <ConfigStringInput
          key={generator + "_" + option.id}
          id={generator + "_" + option.id}
          value={getStringOption(generator, option.id, generatorConfig)}
          onValueChange={(value: string) =>
            setStringOption(
              generator,
              option.id,
              value,
              generatorConfig,
              setGeneratorConfig,
            )
          }
          label={option.label}
          description={option.description}
        />
      );

    case "select-single":
      return (
        <ConfigSelectSingle
          key={generator + "_" + option.id}
          id={generator + "_" + option.id}
          value={getEnumOption(
            generator,
            option.id,
            generatorConfig,
            Object.keys(option.options),
          )}
          onValueChange={(value: string) =>
            setEnumOption(
              generator,
              option.id,
              value,
              generatorConfig,
              setGeneratorConfig,
              Object.keys(option.options),
            )
          }
          label={option.label}
          description={option.description}
          options={option.options}
        />
      );

    case "string-list-manager":
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
