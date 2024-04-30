use reginald::TryFromBytes;

#[derive(TryFromBytes, Debug, PartialEq)]
#[reginald(width_bytes = 1)]
struct Reg {
    #[reginald(bits = [0..=1])]
    f0: u8,

    #[reginald(bits = [2..=3], trait_width_bytes=2)]
    f1: E,

    #[reginald(bits = [4..=5], trait_width_bytes=2)]
    f2: E,
}

#[derive(Debug, PartialEq, TryFromBytes, Clone, Copy)]
#[reginald(width_bytes = 2)]
#[repr(u8)]
enum E {
    Variant0 = 0,
    Variant1 = 1,
    Variant3 = 3,
}

#[cfg(test)]
mod tests {
    use super::*;
    use reginald::TryFromBytes;

    fn packed(f0: u8, f1: u8, f2: u8) -> [u8; 1] {
        [(f0 & 0x3) | ((f1 & 0x3) << 2) | ((f2 & 0x3) << 4)]
    }

    fn test_success(f0: u8, f1: E, f2: E) {
        let inp = packed(f0, f1 as u8, f2 as u8);
        let expected = Reg { f0, f1, f2 };
        assert_eq!(Reg::try_from_le_bytes(&inp).unwrap(), expected);
        assert_eq!(Reg::try_from_be_bytes(&inp).unwrap(), expected);
    }

    fn test_err(f0: u8, f1: u8, f2: u8, err_bitpos: usize) {
        let inp = packed(f0, f1, f2);
        assert_eq!(Reg::try_from_be_bytes(&inp).unwrap_err().pos, err_bitpos);
    }

    #[test]
    fn try_from_bytes() {
        test_success(3, E::Variant0, E::Variant3);
        test_success(0, E::Variant3, E::Variant0);
        test_success(0, E::Variant0, E::Variant0);
        test_success(1, E::Variant1, E::Variant1);

        test_err(1, 2, 0, 2);
        test_err(1, 0, 2, 4);
    }
}
