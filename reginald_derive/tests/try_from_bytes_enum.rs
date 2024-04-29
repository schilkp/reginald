use reginald_derive::TryFromBytes;

#[derive(Debug, PartialEq, TryFromBytes)]
#[reginald(width_bytes = 2)]
enum E {
    Variant0 = 0,
    Variant1 = 1,
    Variant3 = 3,
}

#[cfg(test)]
mod tests {
    use super::*;
    use reginald::TryFromBytes;

    #[test]
    fn try_from_bytes() {
        assert_eq!(E::Variant0, E::try_from_le_bytes(&[0, 0]).unwrap());
        assert_eq!(E::Variant0, E::try_from_be_bytes(&[0, 0]).unwrap());
        assert_eq!(E::Variant1, E::try_from_le_bytes(&[1, 0]).unwrap());
        assert_eq!(E::Variant1, E::try_from_be_bytes(&[0, 1]).unwrap());
        assert_eq!(E::Variant3, E::try_from_le_bytes(&[3, 0]).unwrap());
        assert_eq!(E::Variant3, E::try_from_be_bytes(&[0, 3]).unwrap());

        for i in 0..=u16::MAX {
            if i == 0 || i == 1 || i == 3 {
                continue;
            }

            assert_eq!(0, E::try_from_le_bytes(&[2, 0]).unwrap_err().pos);
        }

        assert_eq!(0, E::try_from_be_bytes(&[0, 2]).unwrap_err().pos);
        assert_eq!(0, E::try_from_le_bytes(&[4, 0]).unwrap_err().pos);
        assert_eq!(0, E::try_from_be_bytes(&[0, 4]).unwrap_err().pos);
    }
}
