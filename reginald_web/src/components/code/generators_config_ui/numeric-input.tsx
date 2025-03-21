import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";

export function NumericInput({
  id,
  value,
  onValueChange,
  label,
  description,
}: {
  id: string;
  value: number;
  onValueChange(value: number): void;
  label: string;
  description?: string;
}) {
  const handleValueChanged = (e: React.ChangeEvent<HTMLInputElement>) => {
    const value = Number.parseInt(e.target.value);
    if (!isNaN(value)) {
      onValueChange(value);
    }
  };

  return (
    <div className="grid gap-1.5">
      <Label htmlFor={id} className="font-medium">
        {label}
      </Label>
      <p className="text-sm text-muted-foreground mb-2">{description}</p>
      <Input
        id={id}
        type="number"
        value={value}
        onChange={handleValueChanged}
        className={`max-w-[180px] focus-visible:ring-0`}
      />
    </div>
  );
}
