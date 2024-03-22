# Reginald

Philipp Schilk
2022-2024

### TODO [In-Order]:

- More complex field types
    - LE/BE Uints
    - LE/BE Enums
    - Arrays
    - Bytes? Or is just just an u8 array?
    - Signed int?

- No limit on max reg size?
    - YAML/Json limits -> Allow int & string in 'type value' fields?
    - What 'bigint' crate?
        - Probably rework convert/regmap + generators first?
        - Even needed? Just do everything as uint8 arrays?
    - Define maximum enum/field size?

- Input/processor option to stuff empty fields with reserved fields

- C: Emit both LE & BE version.

- C: Emit "UINT" version.

- Rust: Work out how to assosciate data to register?
    - Make each register a struct, that either has the layout fields directly contained, or
      newtypes to the shared layout?

- Emit 'unused bits'
    - Definetly as mask
    - Rust: "extract/set field" functions? Could be used by actual pack/unpack? Would likely replace RS UInt generator?

- Rust: Port to quote/syn/prettyplease
- !!Reginald crate with code-gen backing!!

quote = "1.0.35"
proc-macro2 = "1.0.79"
prettyplease = "0.2.16"
syn = "2.0.53"

- GUI:
    - Tauri
    - Features/Tabs:
        - Editor/Viewer
        - Dump Decoder/Encoder
        - Remote Control
            - Connection options:
                - port to "bridge" process
                - via UART
                - via BLE

### RUST API NOTES:

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

# Packed structs libs:

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


### References:
C++ Code generator with classes:
https://github.com/jedrzejboczar/regdef-py

RTL + C Code for mem mapped with DSL:
https://github.com/SystemRDL

Python + C++ code:
Also mentions another system: "JSPEC"
https://github.com/SystemRDL
