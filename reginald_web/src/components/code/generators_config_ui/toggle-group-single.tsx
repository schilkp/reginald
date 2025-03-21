import { Label } from "@/components/ui/label";
import { ToggleGroup } from "@/components/ui/toggle-group";
import { JSX } from "react";

export function ConfigToggleGroupSingle({
  id,
  value,
  onValueChange,
  label,
  description,
  children,
}: {
  id: string;
  value: string;
  onValueChange(value: string): void;
  label: string;
  description?: string;
  children: JSX.Element;
}) {
  return (
    <div className="grid gap-1.5">
      <Label htmlFor={id} className="font-medium">
        {label}
      </Label>
      <p className="text-sm text-muted-foreground mb-2">{description}</p>
      <ToggleGroup
        type="single"
        id={id}
        value={value}
        onValueChange={onValueChange}
        className="justify-start"
      >
        {children}
      </ToggleGroup>
    </div>
  );
}
