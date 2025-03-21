export const exampleYaml = `name: Schilk1000
defaults:
  layout_bitwidth: 8

registers:
  CTRL: !Register
    adr: 0x02
    reset_val: 0x00
    layout: !Layout
      STATUS:
        bits: 0-1
        accepts: !Enum
          OK:
            val: 0
          WARNING:
            val: 1
          BAD:
            val: 2
          VERY_BAD:
            val: 3
      CTRL:
        bits: 2
        accepts: !UInt
      RESERVED1:
        bits: 3-4
        accepts: !Fixed 0x01
`;
