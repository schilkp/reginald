import { Config } from "./c_funcpack_config";
import { ConfigCheckbox } from "../generators_config_ui/checkbox";
import ConfigStringListManager from "../generators_config_ui/string-list-manager";
import { NumericInput } from "../generators_config_ui/numeric-input";
import { ConfigToggleGroupSingle } from "../generators_config_ui/toggle-group-single";
import { ToggleGroupItem } from "@/components/ui/toggle-group";
import { Separator } from "@/components/ui/separator";

export function GeneratorSettingsCFunpack({ config }: { config: Config }) {
  return (
    <div className="space-y-8">
      {/* <h3 className="text-lg font-medium">General Settings</h3> */}

      <ConfigToggleGroupSingle
        id="c_funcpack_endian"
        value={config.Endianess}
        onValueChange={(value: string) => {
          if (value !== "le" && value !== "be" && value !== "both") return; // ...
          config.updateProperty("Endianess", value);
        }}
        label="Endianess"
        description="Endianess of functions and constants to generate."
      >
        <>
          <ToggleGroupItem value="le">Little</ToggleGroupItem>
          <ToggleGroupItem value="be">Big</ToggleGroupItem>
          <ToggleGroupItem value="both">Both</ToggleGroupItem>
        </>
      </ConfigToggleGroupSingle>

      <ConfigToggleGroupSingle
        id="c_funcpack_defer"
        value={config.DeferToEndian}
        onValueChange={(value: string) => {
          if (value !== "le" && value !== "be" && value !== "off") return; // ...
          config.updateProperty("DeferToEndian", value);
        }}
        label="Defer-to Endianess"
        description="For other endianess, generate only simple functions that defers to this implementation."
      >
        <>
          <ToggleGroupItem value="le">Little</ToggleGroupItem>
          <ToggleGroupItem value="be">Big</ToggleGroupItem>
          <ToggleGroupItem value="off">Off</ToggleGroupItem>
        </>
      </ConfigToggleGroupSingle>

      <ConfigCheckbox
        id="c_funcpack_bitfield"
        checked={config.RegistersAsBitfields}
        onCheckedChange={(checked: boolean) =>
          config.updateProperty("RegistersAsBitfields", checked)
        }
        label="Make register structs bitfields to reduce their memory size."
        description="May reduce performance. Note that their memory layout will not match the actual register and the (un)packing functions must still be used."
      />

      <ConfigCheckbox
        id="c_funcpack_static_inline"
        checked={config.FuncsStaticInline}
        onCheckedChange={(checked: boolean) =>
          config.updateProperty("FuncsStaticInline", checked)
        }
        label="Make all functions static inline."
        description="May be disabled if splitting code into header and source."
      />

      <ConfigCheckbox
        id="c_funcpack_prototypes"
        checked={config.FuncsAsPrototypes}
        onCheckedChange={(checked: boolean) =>
          config.updateProperty("FuncsAsPrototypes", checked)
        }
        label="Generate function prototypes instead of full implementations."
        description="May be enabled if splitting code into header and source."
      />

      <ConfigCheckbox
        id="c_funcpack_clang_format"
        checked={config.ClangFormatGuard}
        onCheckedChange={(checked: boolean) =>
          config.updateProperty("ClangFormatGuard", checked)
        }
        label="Surround file with a clang-format off guard"
      />

      <ConfigCheckbox
        id="c_funcpack_include_guards"
        checked={config.IncludeGuards}
        onCheckedChange={(checked: boolean) =>
          config.updateProperty("IncludeGuards", checked)
        }
        label="Generate an include guard."
        description="May be disabled if splitting code into header and source."
      />

      <ConfigStringListManager
        id="c_funcpack_includes"
        content={config.Includes}
        onContentChange={(content: string[]) =>
          config.updateProperty("Includes", content)
        }
        ghost_text="Add include..."
        label="Additional Includes"
        description="Header file that should be included at the top of the generated file. May be required when splitting generated code into multiple files."
      />

      <NumericInput
        id="c_funcpack_max_enum_width"
        value={config.MaxEnumBitwidth}
        onValueChange={(value: number) => {
          config.updateProperty("MaxEnumBitwidth", value);
        }}
        label="Maximum enum bitwidth"
      />

      <Separator />

      <div>
        <p className="text-sm font-medium">Components to Generate</p>
        <p className="text-sm text-muted-foreground">
          Allows different components of the generated content to be placed into
          separat files.
        </p>
      </div>

      <ConfigCheckbox
        id="c_funcpack_gen_enums"
        checked={config.GenerateEnums}
        onCheckedChange={(checked: boolean) =>
          config.updateProperty("GenerateEnums", checked)
        }
        label="Enums"
      />

      <ConfigCheckbox
        id="c_funcpack_gen_enum_validation"
        checked={config.GenerateEnumValidationMacros}
        onCheckedChange={(checked: boolean) =>
          config.updateProperty("GenerateEnumValidationMacros", checked)
        }
        label="Enum Validation Macros"
      />
      <ConfigCheckbox
        id="c_funcpack_gen_structs"
        checked={config.GenerateStructs}
        onCheckedChange={(checked: boolean) =>
          config.updateProperty("GenerateStructs", checked)
        }
        label="Register Structs"
      />
      <ConfigCheckbox
        id="c_funcpack_gen_struct_conv"
        checked={config.GenerateStructConversionFuncs}
        onCheckedChange={(checked: boolean) =>
          config.updateProperty("GenerateStructConversionFuncs", checked)
        }
        label="Register Struct Conversion Functions"
      />
      <ConfigCheckbox
        id="c_funcpack_gen_reg_props"
        checked={config.GenerateRegisterProperties}
        onCheckedChange={(checked: boolean) =>
          config.updateProperty("GenerateRegisterProperties", checked)
        }
        label="Register Properties"
      />
      <ConfigCheckbox
        id="c_funcpack_gen_enums"
        checked={config.GenerateGenericMacros}
        onCheckedChange={(checked: boolean) =>
          config.updateProperty("GenerateGenericMacros", checked)
        }
        label="Generic Macros"
      />
    </div>
  );
}
