use reginald::ToBytes;

#[derive(ToBytes)]
#[reginald(width_bytes = 2)]
#[reginald(fixed_bits = ([6..=15], 0xFF))]
struct Reg {
    #[reginald(bits = 0)]
    field0: bool,

    #[reginald(bits = [2..=5])]
    field1: u8,
}

#[cfg(test)]
mod tests {
    use super::*;
    use reginald::ToBytes;

    #[test]
    fn pack() {
        let reg = Reg {
            field0: true,
            field1: 0xA,
        };

        let mut expected: [u8; 2] = [
            (0x1 << 0)  // Field 0
            | (0xA << 2) // Field 1
            | (0b11 << 6), // Fixed bits
            0b111111, // Fixed bits
        ];

        assert_eq!(expected, reg.to_le_bytes());
        expected.reverse();
        assert_eq!(expected, reg.to_be_bytes());
    }
}
