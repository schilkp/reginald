import { Checkbox } from "@/components/ui/checkbox";
import { Label } from "@/components/ui/label";
import { CheckedState } from "@radix-ui/react-checkbox";

export function ConfigCheckbox({
  id,
  checked,
  onCheckedChange,
  label,
  description,
}: {
  id: string;
  checked: CheckedState;
  onCheckedChange(checked: CheckedState): void;
  label: string;
  description?: string;
}) {
  return (
    <div className="flex items-start space-x-2">
      <Checkbox id={id} checked={checked} onCheckedChange={onCheckedChange} />
      <div className="grid gap-1.5 leading-none">
        <Label htmlFor={id} className="font-medium">
          {label}
        </Label>
        {description && description !== "" && (
          <p className="text-sm text-muted-foreground">{description}</p>
        )}
      </div>
    </div>
  );
}
