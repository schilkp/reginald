use reginald_codegen::regmap::listing::RegisterMap;
use wasm_bindgen::prelude::*;
use web_sys::console;

#[wasm_bindgen]
#[derive(Copy, Clone)]
pub enum ListingFormat {
    Yaml,
    Json,
}

// TODO: Could not figure out how to make this an impl via bindgen?
#[wasm_bindgen]
pub fn listing_format_to_string(inp: ListingFormat) -> String {
    match inp {
        ListingFormat::Yaml => "yaml",
        ListingFormat::Json => "json",
    }
    .to_string()
}

#[wasm_bindgen]
pub fn is_parseable_listing(inp: String, format: ListingFormat) -> bool {
    match format {
        ListingFormat::Yaml => RegisterMap::from_yaml_str(&inp),
        ListingFormat::Json => RegisterMap::from_hjson_str(&inp),
    }
    .is_ok()
}

#[wasm_bindgen]
pub fn convert_listing_format(
    inp: String,
    in_format: ListingFormat,
    out_format: ListingFormat,
) -> Result<String, String> {
    let map: RegisterMap = match in_format {
        ListingFormat::Yaml => RegisterMap::from_yaml_str(&inp),
        ListingFormat::Json => RegisterMap::from_hjson_str(&inp),
    }
    .map_err(|e| e.to_string())?;

    match out_format {
        ListingFormat::Yaml => map.to_yaml(),
        ListingFormat::Json => map.to_json(),
    }
    .map_err(|e| e.to_string())
}

#[wasm_bindgen]
pub fn hello_world() {
    console_error_panic_hook::set_once();
    console::error_1(&JsValue::from_str("Helo from WASM!"));
}
