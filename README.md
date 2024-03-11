# Reginald

Philipp Schilk
2022-2024

### TODOs:

- TEST NO-CONTINOUS FIELDS!
    - With enum!

- RS Nodeps:
    - options for what to derive for both struct & enum.
    - Rename to something "struct"

- RS uint:

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
