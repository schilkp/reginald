use super::{TypeBitwidth, TypeValue};

pub fn bit_mask(width: TypeBitwidth) -> TypeValue {
    if width == 0 {
        0
    } else {
        let mut result: TypeValue = 0;
        for i in 1..=width {
            result |= 1 << (i - 1);
        }
        result
    }
}

pub fn bit_mask_range(pos_a: TypeBitwidth, pos_b: TypeBitwidth) -> TypeValue {
    let lsb_pos = TypeBitwidth::min(pos_a, pos_b);
    let msb_pos = TypeBitwidth::max(pos_a, pos_b);
    assert!(lsb_pos <= msb_pos);
    let width = msb_pos - lsb_pos + 1;
    bit_mask(width) << lsb_pos
}

pub fn bit_mask_is_contigous(mask: TypeValue) -> bool {
    bit_mask_range(lsb_pos(mask), msb_pos(mask)) == mask
}

pub fn msb_pos(val: TypeValue) -> TypeBitwidth {
    if val == 0 {
        0
    } else {
        val.ilog2()
    }
}

pub fn lsb_pos(val: TypeValue) -> TypeBitwidth {
    if val == 0 {
        0
    } else {
        let mut i = 0;
        loop {
            if val & (1 << i) != 0 {
                return i;
            } else {
                i += 1;
            }
        }
    }
}

pub fn unpositioned_mask(mask: TypeValue) -> TypeValue {
    mask >> lsb_pos(mask)
}

pub fn mask_width(mask: TypeValue) -> TypeBitwidth {
    msb_pos(mask) - lsb_pos(mask) + 1
}

pub fn fits_into_bitwidth(val: TypeValue, bitwidth: TypeBitwidth) -> bool {
    (!bit_mask(bitwidth)) & val == 0
}

#[cfg(test)]
mod tests {
    use crate::regmap::MAX_BITWIDTH;

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_bit_mask() {
        assert_eq!(bit_mask(0), 0b00);
        assert_eq!(bit_mask(1), 0b01);
        assert_eq!(bit_mask(2), 0b11);
    }

    #[test]
    fn test_bit_mask_range() {
        assert_eq!(bit_mask_range(0, 0), 0b01);
        assert_eq!(bit_mask_range(1, 1), 0b10);
        assert_eq!(bit_mask_range(5, 3), 0b111000);
    }

    #[test]
    fn test_bit_mask_is_contigous() {
        assert_eq!(bit_mask_is_contigous(0b1), true);
        assert_eq!(bit_mask_is_contigous(0b10), true);
        assert_eq!(bit_mask_is_contigous(0b100), true);
        assert_eq!(bit_mask_is_contigous(0b111), true);
        assert_eq!(bit_mask_is_contigous(0b1110), true);
        assert_eq!(bit_mask_is_contigous(0b1010), false);
    }

    #[test]
    fn test_msb_pos() {
        assert_eq!(msb_pos(0x0), 0);
        assert_eq!(msb_pos(0x1), 0);
        assert_eq!(msb_pos(0x2), 1);
        assert_eq!(msb_pos(0x10), 4);
        assert_eq!(msb_pos(0x1F), 4);
        assert_eq!(msb_pos(0x20), 5);
        assert_eq!(msb_pos(bit_mask(5)), 5 - 1);
        assert_eq!(msb_pos(bit_mask(MAX_BITWIDTH)), MAX_BITWIDTH - 1);
    }

    #[test]
    fn test_lsb_pos() {
        assert_eq!(lsb_pos(0x0), 0);
        assert_eq!(lsb_pos(0x1), 0);
        assert_eq!(lsb_pos(0x2), 1);
        assert_eq!(lsb_pos(0x3), 0);
        assert_eq!(lsb_pos(0x10), 4);
        assert_eq!(lsb_pos(0x1F), 0);
        assert_eq!(lsb_pos(0x20), 5);
    }

    #[test]
    fn test_unpositioned_mask() {
        assert_eq!(unpositioned_mask(0b0), 0b0);
        assert_eq!(unpositioned_mask(0b1 << 0), 0b1);
        assert_eq!(unpositioned_mask(0b1 << 1), 0b1);
        assert_eq!(unpositioned_mask(0b1 << 2), 0b1);
        assert_eq!(unpositioned_mask(0b1 << 3), 0b1);
        assert_eq!(unpositioned_mask(0xdeadbeef << 15), 0xdeadbeef);
    }

    #[test]
    fn test_fits_into_bitwidth() {
        assert_eq!(fits_into_bitwidth(0b111, 0), false);
        assert_eq!(fits_into_bitwidth(0b111, 1), false);
        assert_eq!(fits_into_bitwidth(0b111, 2), false);
        assert_eq!(fits_into_bitwidth(0b111, 3), true);
        assert_eq!(fits_into_bitwidth(0b111, 4), true);
    }
}
