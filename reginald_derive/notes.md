## TODO:

- Design attribute "interface"
- Implement attribute parsing (without extra crate, probably?)
- Implement actual derive logic


## What to derive: (STAGE 1)

- `ToBytes`
    - For structs
    - For enums
- `TryFromBytes`
    - For structs
    - For enums
- `FromBytes`
    - For structs
    - For enums
- `FromBytesMasked`
    - For enums

- attributes:
    - `trait_width_bytes` (usize)
        - Fields (non-primitve)
    - `masked`
        - Fields (enums)
    - `val` (u128 or byte array)
        - enum entries

## What to derive STAGE 2:

   - `default` (u128 or byte array)
       - Struct

- `ExtractField`
    - For structs
    - For 'phantim structs? or just proc macro?'
- `InsertField`
    - For structs
    - For 'phantim structs? or just proc macro?'
- Uint conversion traits
    - For structs, enums

## RUST API NOTES:

bitfield-struct:

```rust
#[bitfield(u64)]
#[derive(PartialEq, Eq)] // <- Attributes after `bitfield` are carried over
struct MyBitfield {
    /// Defaults to 16 bits for u16
    int: u16,
    /// Interpreted as 1 bit flag, with a custom default value
    #[bits(default = true)]
    flag: bool,
    /// Custom bit size
    #[bits(1)]
    tiny: u8,
    /// Sign extend for signed integers
    #[bits(13)]
    negative: i16,
    /// Supports any type with `into_bits`/`from_bits` functions
    #[bits(16)]
    custom: CustomEnum,
    /// Public field -> public accessor functions
    #[bits(10)]
    pub public: usize,
    /// Also supports read-only fields
    #[bits(1, access = RO)]
    read_only: bool,
    /// And write-only fields
    #[bits(1, access = WO)]
    write_only: bool,
    /// Padding
    #[bits(5)]
    __: u8,
}

let raw: u64 = val.into();
```

https://github.com/ProfFan/dw3000-ng/blob/RUST/src/ll.rs
https://github.com/jkelleyrtp/dw1000-rs
rust embedded matrix

## Packed structs libs:

### Packed Struct:
    - https://docs.rs/packed_struct/latest/packed_struct/
    - Strange API? What does read/write only do?

### Bitfield struct:
    - https://crates.io/crates/bitfield-struct

### Reginald features:
    - Support both big-endian and little-endian serialisation on the same struct.
    - Always-write is zero-size/zero overhead
    - Support also byte array extraction
    - Support non-try unpacking
    - YAML/autogen support.

    - CONSIDER:
        - Look at packed-structs msb0/lsb0 capabilites? Specify bit order? Specify 
