import { OutputGenerator } from "./code-panel";
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";

export function GeneratorConfigCard({
  generators,
  selectedGenerator,
}: {
  generators: Record<string, OutputGenerator>;
  selectedGenerator: string;
}) {
  const generator_options = Object.keys(generators).map((generator_key) => {
    const generator = generators[generator_key];
    return (
      <div
        style={{
          display: selectedGenerator === generator_key ? "block" : "none",
        }}
        key={generator_key}
      >
        {generator.config_panel}
      </div>
    );
  });

  return (
    <Card>
      <CardHeader>
        <CardTitle>{generators[selectedGenerator].title}</CardTitle>
        <CardDescription>
          {generators[selectedGenerator].description}
        </CardDescription>
      </CardHeader>
      <CardContent>{generator_options}</CardContent>
      <CardFooter className="flex justify-between"></CardFooter>
    </Card>
  );
}
