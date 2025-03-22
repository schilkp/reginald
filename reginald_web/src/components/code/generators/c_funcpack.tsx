import { Checkbox } from "@/components/ui/checkbox";
import { StoreApi, UseBoundStore } from "zustand";
import { ConfigStore } from "./c_funcpack_state";

export function GeneratorSettingsCFunpack({
  config,
}: {
  config: UseBoundStore<StoreApi<ConfigStore>>;
}) {
  return (
    <div>
      <Checkbox
        id="make_bitfields"
        checked={config().RegistersAsBitfields}
        onCheckedChange={(checked: boolean) =>
          config().set("RegistersAsBitfields", checked)
        }
      />
      <label
        htmlFor="make_bitfields"
        className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70"
      >
        Make structs bitfields.
      </label>
    </div>
  );
}
