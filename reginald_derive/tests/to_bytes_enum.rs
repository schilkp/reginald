use reginald_derive::ToBytes;

#[derive(ToBytes)]
#[reginald(width_bytes = 2)]
enum E {
    Variant0 = 0,
    Variant1 = 1,

    #[reginald(value = 2)]
    Variant2,

    #[reginald(value = 5)]
    Variant5,
}

#[cfg(test)]
mod tests {
    use super::*;
    use reginald::ToBytes;

    #[test]
    fn to_bytes() {
        assert_eq!(E::Variant0.to_le_bytes(), [0, 0]);
        assert_eq!(E::Variant0.to_be_bytes(), [0, 0]);
        assert_eq!(E::Variant1.to_le_bytes(), [1, 0]);
        assert_eq!(E::Variant1.to_be_bytes(), [0, 1]);
        assert_eq!(E::Variant2.to_le_bytes(), [2, 0]);
        assert_eq!(E::Variant2.to_be_bytes(), [0, 2]);
        assert_eq!(E::Variant5.to_le_bytes(), [5, 0]);
        assert_eq!(E::Variant5.to_be_bytes(), [0, 5]);
    }
}
