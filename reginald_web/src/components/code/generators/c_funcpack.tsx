import { Checkbox } from "@/components/ui/checkbox";

export function GeneratorSettingsCFunpack() {
  return (
    <div>
      <Checkbox id="make_bitfields" />
      <label
        htmlFor="make_bitfields"
        className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70"
      >
        Make structs bitfields.
      </label>
    </div>
  );
}
