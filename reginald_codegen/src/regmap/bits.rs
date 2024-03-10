use std::ops::RangeInclusive;

use crate::utils::numbers_as_ranges;

use super::{TypeBitwidth, TypeValue, MAX_BITWIDTH};

pub fn bitmask_from_width(width: TypeBitwidth) -> TypeValue {
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

pub fn bitmask_from_range(range: &RangeInclusive<TypeBitwidth>) -> TypeValue {
    let width = range.end() - range.start() + 1;
    bitmask_from_width(width) << range.start()
}

pub fn bitmask_is_contigous(mask: TypeValue) -> bool {
    bitmask_from_range(&(lsb_pos(mask)..=msb_pos(mask))) == mask
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
    if mask == 0 {
        return 0;
    } else {
        msb_pos(mask) - lsb_pos(mask) + 1
    }
}

pub fn fits_into_bitwidth(val: TypeValue, bitwidth: TypeBitwidth) -> bool {
    (!bitmask_from_width(bitwidth)) & val == 0
}

pub fn mask_to_bits(mask: TypeValue) -> Vec<TypeBitwidth> {
    let mut bits = vec![];

    for bitpos in 0..MAX_BITWIDTH {
        if mask & (0x1 << bitpos) != 0 {
            bits.push(bitpos);
        }
    }

    bits
}

pub fn mask_to_bit_ranges(mask: TypeValue) -> Vec<RangeInclusive<TypeBitwidth>> {
    numbers_as_ranges(mask_to_bits(mask))
}

#[cfg(test)]
mod tests {
    use crate::regmap::MAX_BITWIDTH;

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_bitmask_from_witdth() {
        assert_eq!(bitmask_from_width(0), 0b00);
        assert_eq!(bitmask_from_width(1), 0b01);
        assert_eq!(bitmask_from_width(2), 0b11);
    }

    #[test]
    fn test_bitmask_from_ranges() {
        assert_eq!(bitmask_from_range(&(0..=0)), 0b01);
        assert_eq!(bitmask_from_range(&(1..=1)), 0b10);
        assert_eq!(bitmask_from_range(&(3..=5)), 0b111000);
    }

    #[test]
    fn test_bitmask_is_contigous() {
        assert_eq!(bitmask_is_contigous(0b1), true);
        assert_eq!(bitmask_is_contigous(0b10), true);
        assert_eq!(bitmask_is_contigous(0b100), true);
        assert_eq!(bitmask_is_contigous(0b111), true);
        assert_eq!(bitmask_is_contigous(0b1110), true);
        assert_eq!(bitmask_is_contigous(0b1010), false);
    }

    #[test]
    fn test_msb_pos() {
        assert_eq!(msb_pos(0x0), 0);
        assert_eq!(msb_pos(0x1), 0);
        assert_eq!(msb_pos(0x2), 1);
        assert_eq!(msb_pos(0x3), 1);
        assert_eq!(msb_pos(0x10), 4);
        assert_eq!(msb_pos(0x1F), 4);
        assert_eq!(msb_pos(0x20), 5);
        assert_eq!(msb_pos(bitmask_from_width(5)), 5 - 1);
        assert_eq!(msb_pos(bitmask_from_width(MAX_BITWIDTH)), MAX_BITWIDTH - 1);
        for msb_position in [4_u32, 6, 10] {
            for lsbs in 0..(2_u32.pow(msb_position)) {
                let val = (1 << msb_position) | lsbs;
                assert_eq!(msb_pos(val.into()), msb_position);
            }
        }
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

    #[test]
    fn test_mask_to_bits() {
        assert_eq!(mask_to_bits(0b0), vec![]);
        assert_eq!(mask_to_bits(0b111), vec![0, 1, 2]);
        assert_eq!(mask_to_bits(0b1110), vec![1, 2, 3]);
        assert_eq!(mask_to_bits(0b1101110), vec![1, 2, 3, 5, 6]);
    }

    #[test]
    fn test_mask_to_bit_ranges() {
        assert_eq!(mask_to_bit_ranges(0b0), vec![]);
        assert_eq!(mask_to_bit_ranges(0b111), vec![0..=2]);
        assert_eq!(mask_to_bit_ranges(0b1110), vec![1..=3]);
        assert_eq!(mask_to_bit_ranges(0b1101110), vec![1..=3, 5..=6]);
    }

    #[test]
    fn test_mask_width() {
        assert_eq!(mask_width(0b0), 0);
        assert_eq!(mask_width(0b1), 1);
        assert_eq!(mask_width(0b11), 2);
        assert_eq!(mask_width(0b1100), 2);
        assert_eq!(mask_width(0b1101), 4);
    }
}
