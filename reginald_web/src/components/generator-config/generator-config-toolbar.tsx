import {
  generatorProps,
  MenuGroups,
  ReginaldGenerator,
} from "@/reginald/generators";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectLabel,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { useGeneratorConfigContext } from "./generator-config-context";

export function GeneratorConfigToolbar() {
  let { selectedGenerator, setSelectedGenerator } = useGeneratorConfigContext();

  const groups = MenuGroups.map((lang) => {
    const options = Object.entries(generatorProps).map(([generator, props]) => {
      if (props.menu_group === lang) {
        return (
          <Tooltip key={generator}>
            <SelectItem value={generator}>
              <TooltipTrigger>{props.title}</TooltipTrigger>
            </SelectItem>
            <TooltipContent>{props.description}</TooltipContent>
          </Tooltip>
        );
      } else {
        return null;
      }
    });
    return (
      <SelectGroup key={lang}>
        <SelectLabel>{lang}</SelectLabel>
        {options}
      </SelectGroup>
    );
  });

  return (
    <div className="flex items-center space-x-1 pr-1">
      <TooltipProvider>
        <Select
          value={selectedGenerator}
          onValueChange={(value) => {
            setSelectedGenerator(value as ReginaldGenerator);
          }}
        >
          <SelectTrigger className="w-75">
            <SelectValue placeholder="Select an output generator." />
          </SelectTrigger>
          <SelectContent>{groups}</SelectContent>
        </Select>
      </TooltipProvider>
    </div>
  );
}
