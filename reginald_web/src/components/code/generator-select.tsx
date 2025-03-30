import { Dispatch, SetStateAction } from "react";
import { OutputGenerator } from "./code-panel";
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectLabel,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip";

export function GeneratorSelecetor({
  generators,
  languages,
  selectedGenerator,
  setSelectedGenerator,
}: {
  generators: Record<string, OutputGenerator>;
  languages: Record<string, string>;
  selectedGenerator: string;
  setSelectedGenerator: Dispatch<SetStateAction<string>>;
}) {
  const groups = Object.keys(languages).map((lang_key) => {
    const options = Object.keys(generators).map((generator_key) => {
      const generator = generators[generator_key];
      if (generator_key.startsWith(lang_key)) {
        return (
          <Tooltip key={generator_key}>
            <SelectItem value={generator_key}>
              <TooltipTrigger>{generator.title}</TooltipTrigger>
            </SelectItem>
            <TooltipContent>{generator.description}</TooltipContent>
          </Tooltip>
        );
      } else {
        return null;
      }
    });
    return (
      <SelectGroup key={lang_key}>
        <SelectLabel>{languages[lang_key]}</SelectLabel>
        {options}
      </SelectGroup>
    );
  });

  return (
    <TooltipProvider>
      <Select
        value={selectedGenerator}
        onValueChange={(value) => {
          setSelectedGenerator(value);
        }}
      >
        <SelectTrigger className="w-[280px]" size="sm">
          <SelectValue placeholder="Select an output generator." />
        </SelectTrigger>
        <SelectContent>{groups}</SelectContent>
      </Select>
    </TooltipProvider>
  );
}
