use reginald::FromMaskedBytes;

#[derive(Debug, PartialEq, FromMaskedBytes)]
#[reginald(width_bytes = 2)]
enum E {
    Variant0 = 0,
    Variant1 = 1,
    Variant2 = 2,
    Variant3 = 3,
}

#[cfg(test)]
mod tests {
    use super::*;
    use reginald::FromMaskedBytes;

    #[test]
    fn from_bytes_masked() {
        assert_eq!(E::Variant0, E::from_masked_le_bytes(&[0, 0]));
        assert_eq!(E::Variant0, E::from_masked_be_bytes(&[0, 0]));
        assert_eq!(E::Variant1, E::from_masked_le_bytes(&[1, 0]));
        assert_eq!(E::Variant1, E::from_masked_be_bytes(&[0, 1]));
        assert_eq!(E::Variant2, E::from_masked_le_bytes(&[2, 0]));
        assert_eq!(E::Variant2, E::from_masked_be_bytes(&[0, 2]));
        assert_eq!(E::Variant3, E::from_masked_le_bytes(&[3, 0]));
        assert_eq!(E::Variant3, E::from_masked_be_bytes(&[0, 3]));
    }
}
