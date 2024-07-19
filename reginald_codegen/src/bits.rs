use std::ops::RangeInclusive;

use crate::regmap::{TypeBitwidth, TypeValue, MAX_BITWIDTH};
use reginald_utils::numbers_as_ranges;

/// Generate a bitmask of specified width
///
/// Example:
/// ```rust
/// # use reginald_codegen::bits::bitmask_from_width;
/// assert_eq!(bitmask_from_width(0), 0b00);
/// assert_eq!(bitmask_from_width(1), 0b01);
/// assert_eq!(bitmask_from_width(2), 0b11);
/// ```
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

/// Generate a bitmask that covers the (inclusive) range of bits given as an
/// argument.
///
/// Example:
/// ```rust
/// # use reginald_codegen::bits::bitmask_from_range;
/// assert_eq!(bitmask_from_range(&(0..=3)), 0b1111);
/// assert_eq!(bitmask_from_range(&(1..=2)), 0b0110);
/// assert_eq!(bitmask_from_range(&(3..=3)), 0b1000);
/// ```
pub fn bitmask_from_range(range: &RangeInclusive<TypeBitwidth>) -> TypeValue {
    let width = range.end() - range.start() + 1;
    bitmask_from_width(width) << range.start()
}

/// Checks if a given bitmask is contigous (All '1' bits, if any, are adjacent
/// without any gaps)
///
/// Example:
/// ```rust
/// # use reginald_codegen::bits::bitmask_is_contigous;
/// assert_eq!(bitmask_is_contigous(0b00000), true);
/// assert_eq!(bitmask_is_contigous(0b00110), true);
/// assert_eq!(bitmask_is_contigous(0b00111), true);
/// assert_eq!(bitmask_is_contigous(0b01110), true);
/// assert_eq!(bitmask_is_contigous(0b11101), false);
/// ```
pub fn bitmask_is_contigous(mask: TypeValue) -> bool {
    if mask == 0 {
        true
    } else {
        bitmask_from_range(&(lsb_pos(mask)..=msb_pos(mask))) == mask
    }
}

/// Determines the position of the most significant '1'
///
/// Example:
/// ```rust
/// # use reginald_codegen::bits::msb_pos;
/// assert_eq!(msb_pos(0b00000), 0);
/// assert_eq!(msb_pos(0b00001), 0);
/// assert_eq!(msb_pos(0b00110), 2);
/// assert_eq!(msb_pos(0b00111), 2);
/// assert_eq!(msb_pos(0b01110), 3);
/// assert_eq!(msb_pos(0b11101), 4);
/// ```
pub fn msb_pos(val: TypeValue) -> TypeBitwidth {
    if val == 0 {
        0
    } else {
        val.ilog2()
    }
}

/// Determines the position of the least significant '1'
///
/// Example:
/// ```rust
/// # use reginald_codegen::bits::lsb_pos;
/// assert_eq!(lsb_pos(0b00000), 0);
/// assert_eq!(lsb_pos(0b00001), 0);
/// assert_eq!(lsb_pos(0b00110), 1);
/// assert_eq!(lsb_pos(0b00111), 0);
/// assert_eq!(lsb_pos(0b01110), 1);
/// assert_eq!(lsb_pos(0b11100), 2);
/// ```
pub fn lsb_pos(val: TypeValue) -> TypeBitwidth {
    if val == 0 {
        0
    } else {
        let mut i = 0;
        loop {
            if val & (1 << i) != 0 {
                return i;
            }
            i += 1;
        }
    }
}

/// Shifts a bit mask right, until the least significant bit that is '1' is at
/// position 0.
///
/// Example:
/// ```rust
/// # use reginald_codegen::bits::unpositioned_mask;
/// assert_eq!(unpositioned_mask(0b00000), 0b0);
/// assert_eq!(unpositioned_mask(0b00001), 0b1);
/// assert_eq!(unpositioned_mask(0b00110), 0b11);
/// assert_eq!(unpositioned_mask(0b10100), 0b101);
/// ```
pub fn unpositioned_mask(mask: TypeValue) -> TypeValue {
    mask >> lsb_pos(mask)
}

/// Determines the bit-width of  a mask:
///
/// Example:
/// ```rust
/// # use reginald_codegen::bits::mask_width;
/// assert_eq!(mask_width(0b00000), 0);
/// assert_eq!(mask_width(0b00001), 1);
/// assert_eq!(mask_width(0b00110), 2);
/// assert_eq!(mask_width(0b10100), 3);
/// ```
pub fn mask_width(mask: TypeValue) -> TypeBitwidth {
    if mask == 0 {
        0
    } else {
        msb_pos(mask) - lsb_pos(mask) + 1
    }
}

/// Check if a given value fits into a field of given width without truncation
///
/// Example:
/// ```rust
/// # use reginald_codegen::bits::fits_into_bitwidth;
/// assert_eq!(fits_into_bitwidth(0b000000, 0), true);
/// assert_eq!(fits_into_bitwidth(0b000001, 0), false);
/// assert_eq!(fits_into_bitwidth(0b000001, 1), true);
/// assert_eq!(fits_into_bitwidth(0b000001, 2), true);
/// assert_eq!(fits_into_bitwidth(0b000100, 2), false);
/// assert_eq!(fits_into_bitwidth(0b000100, 3), true);
/// ```
pub fn fits_into_bitwidth(val: TypeValue, bitwidth: TypeBitwidth) -> bool {
    (!bitmask_from_width(bitwidth)) & val == 0
}

/// Convert a mask to a vector of the positions of all bits that are set.
pub fn mask_to_bits(mask: TypeValue) -> Vec<TypeBitwidth> {
    let mut bits = vec![];

    for bitpos in 0..MAX_BITWIDTH {
        if mask & (0x1 << bitpos) != 0 {
            bits.push(bitpos);
        }
    }

    bits
}

/// Convert a bit mask to a list of inclusive ranges that cover all bits that
/// are set.
///
/// Example:
/// ```rust
/// # use reginald_codegen::bits::mask_to_bit_ranges;
/// assert_eq!(mask_to_bit_ranges(0b000000), vec![]);
/// assert_eq!(mask_to_bit_ranges(0b000001), vec![0..=0]);
/// assert_eq!(mask_to_bit_ranges(0b101110), vec![1..=3, 5..=5]);
/// ```
pub fn mask_to_bit_ranges(mask: TypeValue) -> Vec<RangeInclusive<TypeBitwidth>> {
    numbers_as_ranges(mask_to_bits(mask))
}

/// Convert a bit mask to a string that explains which bits are set using a list of ranges.
///
/// Example:
/// ```rust
/// # use reginald_codegen::bits::mask_to_bit_ranges_str;
/// assert_eq!(mask_to_bit_ranges_str(0b000001), String::from("0"));
/// assert_eq!(mask_to_bit_ranges_str(0b001110), String::from("3:1"));
/// assert_eq!(mask_to_bit_ranges_str(0b111010), String::from("5:3, 1"));
/// ```
pub fn mask_to_bit_ranges_str(mask: TypeValue) -> String {
    let ranges: Vec<String> = mask_to_bit_ranges(mask)
        .iter()
        .map(|range| {
            if range.start() == range.end() {
                format!("{}", range.start())
            } else {
                format!("{}:{}", range.end(), range.start())
            }
        })
        .rev()
        .collect();

    ranges.join(", ")
}

/// Number of bytes required to store an N-bit value.
///
/// Example:
/// ```rust
/// # use reginald_codegen::bits::bitwidth_to_width_bytes;
/// assert_eq!(bitwidth_to_width_bytes(7), 1);
/// assert_eq!(bitwidth_to_width_bytes(8), 1);
/// assert_eq!(bitwidth_to_width_bytes(9), 2);
/// ```
pub fn bitwidth_to_width_bytes(bitwidth: TypeBitwidth) -> TypeBitwidth {
    (bitwidth + 7) / 8
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
        assert_eq!(bitmask_is_contigous(0b0), true);
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
        assert_eq!(mask_to_bits(0b0), Vec::<TypeBitwidth>::new());
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
