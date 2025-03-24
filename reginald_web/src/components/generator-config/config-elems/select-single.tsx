import { Label } from "@/components/ui/label";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";

export function ConfigSelectSingle({
  id,
  value,
  onValueChange,
  label,
  description,
  options,
}: {
  id: string;
  value: string;
  onValueChange(value: string): void;
  label: string;
  description?: string;
  options: Record<string, string>; // {key: Name}
}) {
  const handleValueChanged = (e: string) => {
    if (Object.keys(options).includes(e)) {
      onValueChange(e);
    }
  };

  const items = Object.entries(options).map(([option_id, option_text]) => {
    return (
      <ToggleGroupItem key={option_id} value={option_id}>
        {option_text}
      </ToggleGroupItem>
    );
  });

  return (
    <div className="grid gap-1.5">
      <Label htmlFor={id} className="font-medium">
        {label}
      </Label>
      <p className="text-sm text-muted-foreground mb-0">{description}</p>
      <ToggleGroup
        id={id}
        type="single"
        value={value}
        onValueChange={handleValueChanged}
      >
        {items}
      </ToggleGroup>
    </div>
  );
}
