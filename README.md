# Reginald

Philipp Schilk
2022-2024

### HOW TO HANDLE Packing of non-byte multiple size fields? Truncation OK?

### HOW TO HANDLE Large enums - espc in C?
    Should there be an upper bound?
    In C, support enums-as-defines?
        - Instead of doing try-unpack in C, do only a validate? Or only expose a validate?
        - Provide options to:
            - Always make enums defines,
            - Only make enums above a given size defines,
    In C, ADD static assert to output that checks that all int is large enough to hold the max_enum_bitwidth? Or 
    that checks int is large enough to hold the max enum value that is smaller or eq. to max_enum_bitwidth?

### TODO:

- Derive crate impl
- rs-derive generator
- Port codegen generator tests to integration tests

### IDEAS:

- More complex field types
    - Arrays
    - Bytes? Or is just just an u8 array?
    - Signed int?
    - Sparse/Compressed Enum?
        - "Compress" list of values to linear repr
        - Binary values:
            - 0x00000 -> Stored as 0x0 in struct
            - 0x12312 -> Stored as 0x1 in struct
            - 0xFFFFF -> Stored as 0xF in struct
    - Sparse/Compressed Uint?

- No limit on max reg size?
    - YAML/Json limits -> Allow int & string in 'type value' fields?
    - What 'bigint' crate?
        - Probably rework convert/regmap + generators first?
        - Even needed? Just do everything as uint8 arrays?
    - Define maximum enum/field size -> Split into TypeFieldValue & TypeRegisterValue

- Input/processor option to stuff empty fields with reserved fields
- Input/processor option to stuff enums to allow full conversion

- C: Emit "UINT" version.

- GUI:
    - Tauri
    - Features/Tabs:
        - Graphical Editor/Viewer
        - Dump Decoder/Encoder
        - Remote Control
            - Connection options:
                - port to "bridge" process
                - via UART
                - via BLE


### NOT TODOs:

- Better syntax for bit ranges?
- Support more flexible register widths?
- Infer register/field width from underlying type?

- More complex field types
    - (( LE/BE Uints? Needed? ))
    - (( LE/BE Enums? Needed? ))

### Structure
    - `reginald_codegen`: CLI + Code generators
    - `reginald`: Traits
    - `reginald_derive`: Packed struct derive macros
    - `reginald_py`: Python distribution + bindings
    - `reginald_gui`: GUI Tool

### References:
C++ Code generator with classes:
https://github.com/jedrzejboczar/regdef-py

RTL + C Code for mem mapped with DSL:
https://github.com/SystemRDL

Python + C++ code:
Also mentions another system: "JSPEC"
https://github.com/SystemRDL
