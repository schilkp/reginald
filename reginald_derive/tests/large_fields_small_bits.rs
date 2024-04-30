use reginald::ToBytes;

#[derive(ToBytes)]
struct Reg {
    #[reginald(bits = 0)]
    field0: u128,

    #[reginald(bits = [2..=5])]
    field1: u128,
}

#[cfg(test)]
mod tests {
    use super::*;
    use reginald::ToBytes;

    #[test]
    fn pack() {
        let reg = Reg { field0: 1, field1: 3 };

        let mut expected: [u8; 1] = [(0x1 << 0) | (3 << 2)];

        assert_eq!(expected, reg.to_le_bytes());
        expected.reverse();
        assert_eq!(expected, reg.to_be_bytes());
    }
}
