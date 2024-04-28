#![allow(dead_code)]

use reginald_derive::{FromBytes, FromMaskedBytes};

#[derive(FromBytes, PartialEq, Debug)]
struct Reg {
    #[reginald(bits = 0..=1, is=trait_masked)]
    field0: E,
    #[reginald(bits = 2..=3, is=trait_masked)]
    field1: E,
    #[reginald(bits = 4..=5, is=trait_masked)]
    field2: E,
    #[reginald(bits = 6..=7, is=trait_masked)]
    field3: E,
}

#[derive(FromMaskedBytes, PartialEq, Debug)]
enum E {
    Variant0 = 0,
    Variant1 = 1,
    Variant2 = 2,
    Variant3 = 3,
}

#[cfg(test)]
mod tests {
    use super::*;
    use reginald::FromBytes;

    #[test]
    fn from_bytes() {
        let mut packed: [u8; 1] = [
            (0x0 << 0)  // Field 0
            | (0x1 << 2)  // Field 1
            | (0x2 << 4)  // Field 2
            | (0x3 << 6), // Field 3
        ];

        let reg_expected = Reg {
            field0: E::Variant0,
            field1: E::Variant1,
            field2: E::Variant2,
            field3: E::Variant3,
        };

        assert_eq!(reg_expected, Reg::from_le_bytes(&packed));
        packed.reverse();
        assert_eq!(reg_expected, Reg::from_be_bytes(&packed));
    }
}
