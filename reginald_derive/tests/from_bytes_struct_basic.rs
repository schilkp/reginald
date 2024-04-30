use reginald::FromBytes;

#[derive(FromBytes, Debug, PartialEq)]
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
    use reginald::FromBytes;

    #[test]
    fn from_bytes() {
        let mut packed: [u8; 2] = [
            (0x1 << 0)  // Field 0
            | (0xA << 2), // Field 1,
            0,
        ];

        let reg_expected = Reg {
            field0: true,
            field1: 0xA,
        };

        assert_eq!(reg_expected, Reg::from_le_bytes(&packed));
        packed.reverse();
        assert_eq!(reg_expected, Reg::from_be_bytes(&packed));
    }
}
