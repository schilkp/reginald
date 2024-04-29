#![allow(dead_code)]

use reginald_derive::ToBytes;

#[derive(ToBytes)]
struct Reg {
    #[reginald(bits = 0..=2)]
    field0: E,
    #[reginald(bits = 3..=5)]
    field1: E,
}

#[derive(ToBytes)]
enum E {
    #[reginald(value = 0)]
    Variant0,
    #[reginald(value = 1)]
    Variant1,
    #[reginald(value = 2)]
    Variant2,
    #[reginald(value = 3)]
    Variant3,
}

#[cfg(test)]
mod tests {
    use super::*;
    use reginald::ToBytes;

    #[test]
    fn pack() {
        let reg = Reg {
            field0: E::Variant1,
            field1: E::Variant3,
        };

        let mut expected: [u8; 1] = [0b011 << 3 | 0b001 << 0];

        assert_eq!(expected, reg.to_le_bytes());
        expected.reverse();
        assert_eq!(expected, reg.to_be_bytes());
    }
}
