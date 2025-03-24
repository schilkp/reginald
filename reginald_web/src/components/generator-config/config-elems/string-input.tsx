import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";

export function ConfigStringInput({
  id,
  value,
  onValueChange,
  label,
  description,
}: {
  id: string;
  value: string;
  onValueChange(value: string): void;
  label: string;
  description?: string;
}) {
  const handleValueChanged = (e: React.ChangeEvent<HTMLInputElement>) => {
    onValueChange(e.target.value);
  };

  return (
    <div className="grid gap-1.5">
      <Label htmlFor={id} className="font-medium">
        {label}
      </Label>
      <p className="text-sm text-muted-foreground mb-0">{description}</p>
      <Input
        id={id}
        value={value}
        onChange={handleValueChanged}
        className={`max-w-[180px] focus-visible:ring-0`}
      />
    </div>
  );
}
