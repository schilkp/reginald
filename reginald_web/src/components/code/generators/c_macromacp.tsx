import { Checkbox } from "@/components/ui/checkbox";

export function GeneratorSettingsCMacroMap() {
  return (
    <div>
      <Checkbox id="clang_format_guard" />
      <label
        htmlFor="clang_format_guard"
        className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70"
      >
        Include clang-format guard.
      </label>
    </div>
  );
}
