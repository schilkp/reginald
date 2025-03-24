export type ListingFormat = "yaml" | "json";
import * as wasm from "reginald_wasm";

export function toWasmFormat(f: ListingFormat): wasm.ListingFormat {
  switch (f) {
    case "json":
      return wasm.ListingFormat.Json;
    case "yaml":
      return wasm.ListingFormat.Yaml;
    default:
      throw "Unknown format";
  }
}

export function convertListingFormat(
  listing: string,
  from: ListingFormat,
  to: ListingFormat,
): string {
  const prev_format = toWasmFormat(from);
  const new_format = toWasmFormat(to);

  // Only try conversion if the listing is not already in a parseable form
  if (wasm.is_parseable_listing(listing, new_format)) {
    return listing;
  }

  return wasm.convert_listing_format(listing, prev_format, new_format);
}
