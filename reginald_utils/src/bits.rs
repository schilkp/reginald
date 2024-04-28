#![allow(clippy::suspicious_op_assign_impl)]
use std::{
    fmt::Display,
    ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, RangeInclusive, Shl, ShlAssign, Shr, ShrAssign},
};

use crate::{numbers_as_ranges, ranges_to_str, RangeStyle};

#[cfg(feature = "clap")]
use clap::ValueEnum;

const DIGIT_SIZE: usize = 32;

#[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord)]
#[cfg_attr(feature = "clap", derive(ValueEnum))]
pub enum Endianess {
    Little,
    Big,
}

impl Display for Endianess {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Endianess::Little => write!(f, "little-endian"),
            Endianess::Big => write!(f, "big-endian"),
        }
    }
}

impl Endianess {
    pub fn short(&self) -> &'static str {
        match self {
            Endianess::Little => "le",
            Endianess::Big => "be",
        }
    }
}

#[derive(Clone, PartialEq, Default, Hash, Eq)]
pub struct Bits {
    digits: Vec<u32>,
}

impl Bits {
    /// Create new `Bits` that is equal to zero.
    ///
    /// Example:
    /// ```rust
    /// # use reginald_utils::Bits;
    /// assert_eq!(Bits::new(), Bits::from_uint(0b0));
    /// ```
    pub fn new() -> Self {
        Self {
            digits: Vec::with_capacity(8),
        }
    }

    /// Create new `Bits` with bit at given position set.
    ///
    /// Example:
    /// ```rust
    /// # use reginald_utils::Bits;
    /// assert_eq!(Bits::from_bitpos(0), Bits::from_uint(0b001));
    /// assert_eq!(Bits::from_bitpos(1), Bits::from_uint(0b010));
    /// assert_eq!(Bits::from_bitpos(2), Bits::from_uint(0b100));
    /// ```
    pub fn from_bitpos(p: usize) -> Self {
        let mut r = Self::new();
        r.set_bit(p);
        r
    }

    /// Create new `Bits` with all bits in range set.
    ///
    /// Example:
    /// ```rust
    /// # use reginald_utils::Bits;
    /// assert_eq!(Bits::from_range(0..=3), Bits::from_uint(0b1111));
    /// assert_eq!(Bits::from_range(2..=3), Bits::from_uint(0b1100));
    /// ```
    pub fn from_range(r: RangeInclusive<usize>) -> Self {
        let mut b = Self::new();
        b.set_range(r);
        b
    }

    /// Create new `Bits` with value equal to the given literal.
    ///
    /// Example:
    /// ```rust
    /// # use reginald_utils::Bits;
    /// assert_eq!(Bits::from_uint(0b0100), Bits::from_bitpos(2));
    /// assert_eq!(Bits::from_uint(0b1110), Bits::from_range(1..=3));
    /// ```
    pub fn from_uint(m: u128) -> Self {
        let mut r = Self::new();
        assert_eq!(32, DIGIT_SIZE); // Below needs to be updated if digit size changes.
        for digit in 0..4 {
            r.digits.push(((m >> (digit * 32)) & 0xFFFFFFFF) as u32);
        }
        r.trim();
        r
    }

    /// Determines the position of the least significant set bit.
    ///
    /// Example:
    /// ```rust
    /// # use reginald_utils::Bits;
    /// assert_eq!(Bits::from_uint(0b00000).lsb_pos(), 0);
    /// assert_eq!(Bits::from_uint(0b00001).lsb_pos(), 0);
    /// assert_eq!(Bits::from_uint(0b00110).lsb_pos(), 1);
    /// assert_eq!(Bits::from_uint(0b00111).lsb_pos(), 0);
    /// assert_eq!(Bits::from_uint(0b01110).lsb_pos(), 1);
    /// assert_eq!(Bits::from_uint(0b11100).lsb_pos(), 2);
    /// ```
    pub fn lsb_pos(&self) -> usize {
        for bit in 0..self.bit_capacity() {
            if self.get_bit(bit) {
                return bit;
            }
        }

        0
    }

    /// Determines the position of the most significant set bit.
    ///
    /// Example:
    /// ```rust
    /// # use reginald_utils::Bits;
    /// assert_eq!(Bits::from_uint(0b00000).msb_pos(), 0);
    /// assert_eq!(Bits::from_uint(0b00001).msb_pos(), 0);
    /// assert_eq!(Bits::from_uint(0b00110).msb_pos(), 2);
    /// assert_eq!(Bits::from_uint(0b00111).msb_pos(), 2);
    /// assert_eq!(Bits::from_uint(0b01110).msb_pos(), 3);
    /// assert_eq!(Bits::from_uint(0b11101).msb_pos(), 4);
    /// ```
    pub fn msb_pos(&self) -> usize {
        for bit in (0..self.bit_capacity()).rev() {
            if self.get_bit(bit) {
                return bit;
            }
        }

        0
    }

    /// Sets the bit at the given position to one.
    ///
    /// Example:
    /// ```rust
    /// # use reginald_utils::Bits;
    /// let mut b = Bits::new();
    /// b.set_bit(2);
    /// assert_eq!(b, Bits::from_bitpos(2));
    /// ```
    pub fn set_bit(&mut self, p: usize) {
        let digit = p / DIGIT_SIZE;
        let bit_offset = p % DIGIT_SIZE;

        while self.digits.len() < digit + 1 {
            self.digits.push(0x0)
        }
        self.digits[digit] |= 1 << bit_offset;
    }

    /// Sets the bit at the given position to zero.
    ///
    /// Example:
    /// ```rust
    /// # use reginald_utils::Bits;
    /// let mut b = Bits::from_uint(0b1111);
    /// b.clear_bit(2);
    /// assert_eq!(b, Bits::from_uint(0b1011));
    /// ```
    pub fn clear_bit(&mut self, p: usize) {
        let digit = p / DIGIT_SIZE;
        let bit_offset = p % DIGIT_SIZE;

        if self.digits.len() > digit {
            let new_digit = self.digits[digit] & !(0x1 << bit_offset);
            self.digits[digit] = new_digit;
            if digit == self.digits.len() - 1 && new_digit == 0 {
                self.trim()
            }
        }
    }

    /// Sets the bit at the given position to the given value.
    ///
    /// Example:
    /// ```rust
    /// # use reginald_utils::Bits;
    /// let mut b = Bits::new();
    /// b.write_bit(1, true);
    /// assert_eq!(b, Bits::from_uint(0b0010));
    /// b.write_bit(1, false);
    /// assert_eq!(b, Bits::from_uint(0b0000));
    /// ```
    pub fn write_bit(&mut self, p: usize, val: bool) {
        if val {
            self.set_bit(p);
        } else {
            self.clear_bit(p);
        }
    }

    /// Sets all bits in the given range to one.
    ///
    /// Example:
    /// ```rust
    /// # use reginald_utils::Bits;
    /// let mut b = Bits::new();
    /// b.set_range(0..=3);
    /// assert_eq!(b, Bits::from_uint(0b1111));
    /// ```
    pub fn set_range(&mut self, r: RangeInclusive<usize>) {
        for pos in r.clone() {
            self.set_bit(pos);
        }
    }

    /// Retrieve the bit at the given position.
    ///
    /// Example:
    /// ```rust
    /// # use reginald_utils::Bits;
    /// let mut b = Bits::from_uint(0b10);
    /// assert_eq!(false, b.get_bit(0));
    /// assert_eq!(true, b.get_bit(1));
    /// ```
    pub fn get_bit(&self, p: usize) -> bool {
        let digit = p / DIGIT_SIZE;
        let bit_offset = p % DIGIT_SIZE;

        if digit < self.digits.len() {
            (self.digits[digit] & (0x1 << bit_offset)) != 0
        } else {
            false
        }
    }

    /// Check if all bits are set to zero.
    ///
    /// Example:
    /// ```rust
    /// # use reginald_utils::Bits;
    /// let mut b = Bits::from_uint(0b00);
    /// assert_eq!(true, b.is_zero());
    /// let mut b = Bits::from_uint(0b10);
    /// assert_eq!(false, b.is_zero());
    /// ```
    pub fn is_zero(&self) -> bool {
        for b in &self.digits {
            if *b != 0 {
                return false;
            }
        }

        true
    }

    /// Check if any bits are set to one.
    ///
    /// Example:
    /// ```rust
    /// # use reginald_utils::Bits;
    /// let mut b = Bits::from_uint(0b10);
    /// assert_eq!(true, b.is_nonzero());
    /// let mut b = Bits::from_uint(0b00);
    /// assert_eq!(false, b.is_nonzero());
    /// ```
    pub fn is_nonzero(&self) -> bool {
        !self.is_zero()
    }

    /// Check if bits overlap with other bits.
    /// Two bits are said to be overlapping if they both contain a one at any
    /// position.
    ///
    /// Example:
    /// ```rust
    /// # use reginald_utils::Bits;
    /// assert_eq!(true,  Bits::from_uint(0b1110).overlaps_with(&Bits::from_uint(0b0011)));
    /// assert_eq!(false, Bits::from_uint(0b1110).overlaps_with(&Bits::from_uint(0b0001)));
    /// ```
    pub fn overlaps_with(&self, other: &Bits) -> bool {
        (self & other).is_nonzero()
    }

    /// Set all bits that are set in the given mask to zero.
    ///
    /// Example:
    /// ```rust
    /// # use reginald_utils::Bits;
    /// let mut b = Bits::from_uint(0b1111_1111);
    /// b.clear_mask_assign(&Bits::from_uint(0b1100_1100));
    /// assert_eq!(b, Bits::from_uint(0b0011_0011));
    /// ```
    pub fn clear_mask_assign(&mut self, rhs: &Self) {
        let len_digits = usize::max(self.digits.len(), rhs.digits.len());
        for pos in 0..len_digits {
            match (self.digits.get_mut(pos), rhs.digits.get(pos)) {
                (Some(lhs), Some(rhs)) => *lhs &= !rhs, // Simple masking
                (None, Some(_)) => (),                  // relevant digit is already zero.
                (Some(_), None) => (),                  // bitand with all ones noop.
                (None, None) => unreachable!(),         // Guarded by max calculation above.
            }
        }
        self.trim();
    }

    /// Create new `Bits` equal to the current value but with all bits that are set
    ///  in the given mask to zero.
    ///
    /// Example:
    /// ```rust
    /// # use reginald_utils::Bits;
    /// let b = Bits::from_uint(0b1111_1111);
    /// let b = b.clear_mask(&Bits::from_uint(0b1100_1100));
    /// assert_eq!(b, Bits::from_uint(0b0011_0011));
    /// ```
    pub fn clear_mask(&self, rhs: &Self) -> Bits {
        let mut r = self.clone();
        r.clear_mask_assign(rhs);
        r
    }

    /// Shifts `Bits` to the right, until the least significant bit that is set is at position 0.
    ///
    /// Example:
    /// ```rust
    /// # use reginald_utils::Bits;
    /// assert_eq!(Bits::from_uint(0b00000).unpositioned(), Bits::from_uint(0b000));
    /// assert_eq!(Bits::from_uint(0b00001).unpositioned(), Bits::from_uint(0b001));
    /// assert_eq!(Bits::from_uint(0b00110).unpositioned(), Bits::from_uint(0b011));
    /// assert_eq!(Bits::from_uint(0b10100).unpositioned(), Bits::from_uint(0b101));
    /// ```
    pub fn unpositioned(&self) -> Bits {
        self >> self.lsb_pos()
    }

    /// Determine the number of bits required to store this value.
    ///
    /// Example:
    /// ```rust
    /// # use reginald_utils::Bits;
    /// assert_eq!(Bits::from_uint(0b00000).bitwidth(), 0);
    /// assert_eq!(Bits::from_uint(0b00001).bitwidth(), 1);
    /// assert_eq!(Bits::from_uint(0b00100).bitwidth(), 3);
    /// assert_eq!(Bits::from_uint(0b00111).bitwidth(), 3);
    /// ```
    pub fn bitwidth(&self) -> usize {
        if self.is_zero() {
            0
        } else {
            self.msb_pos() + 1
        }
    }

    /// Determines the width of the field that these `Bits` describe, when
    /// interpreted as a positioned mask.
    ///
    /// Example:
    /// ```rust
    /// # use reginald_utils::Bits;
    /// assert_eq!(Bits::from_uint(0b00000).positioned_bitwidth(), 0);
    /// assert_eq!(Bits::from_uint(0b00001).positioned_bitwidth(), 1);
    /// assert_eq!(Bits::from_uint(0b00110).positioned_bitwidth(), 2);
    /// assert_eq!(Bits::from_uint(0b10100).positioned_bitwidth(), 3);
    /// ```
    pub fn positioned_bitwidth(&self) -> usize {
        if self.is_zero() {
            0
        } else {
            self.msb_pos() - self.lsb_pos() + 1
        }
    }

    /// Determine the number of bytes required to store this value.
    ///
    /// Example:
    /// ```rust
    /// # use reginald_utils::Bits;
    /// assert_eq!(Bits::from_uint(0b0 << 0).width_bytes(), 0);
    /// assert_eq!(Bits::from_uint(0b1 << 7).width_bytes(), 1);
    /// assert_eq!(Bits::from_uint(0b1 << 8).width_bytes(), 2);
    /// assert_eq!(Bits::from_uint(0x1FF).width_bytes(), 2);
    /// ```
    pub fn width_bytes(&self) -> usize {
        (self.bitwidth() + 7) / 8
    }

    /// Determines the width of the field in bytes that these `Bits` describe,
    /// when interpreted as a positioned mask.
    ///
    /// Example:
    /// ```rust
    /// # use reginald_utils::Bits;
    /// assert_eq!(Bits::from_uint(0b0000_0000_0000_0000).positioned_width_bytes(), 0);
    /// assert_eq!(Bits::from_uint(0b0000_0000_0000_0001).positioned_width_bytes(), 1);
    /// assert_eq!(Bits::from_uint(0b0000_0001_0000_0001).positioned_width_bytes(), 2);
    /// assert_eq!(Bits::from_uint(0b0000_1111_1111_0000).positioned_width_bytes(), 1);
    /// ```
    pub fn positioned_width_bytes(&self) -> usize {
        (self.positioned_bitwidth() + 7) / 8
    }

    /// Conver to list of all bit positions that contain a one.
    ///
    /// Example:
    /// ```rust
    /// # use reginald_utils::Bits;
    /// assert_eq!(Bits::from_uint(0b00000).to_bit_positions(), vec![]);
    /// assert_eq!(Bits::from_uint(0b00001).to_bit_positions(), vec![0]);
    /// assert_eq!(Bits::from_uint(0b00110).to_bit_positions(), vec![1, 2]);
    /// assert_eq!(Bits::from_uint(0b10100).to_bit_positions(), vec![2, 4]);
    /// ```
    pub fn to_bit_positions(&self) -> Vec<usize> {
        let mut bits = vec![];

        for bitpos in 0..=self.msb_pos() {
            if self.get_bit(bitpos) {
                bits.push(bitpos);
            }
        }

        bits
    }

    /// Convert to a list of inclusive ranges that efficiently cover all bits that are set.
    ///
    /// Example:
    /// ```rust
    /// # use reginald_utils::Bits;
    /// assert_eq!(Bits::from_uint(0b00000).to_bit_ranges(), vec![]);
    /// assert_eq!(Bits::from_uint(0b00001).to_bit_ranges(), vec![0..=0]);
    /// assert_eq!(Bits::from_uint(0b00110).to_bit_ranges(), vec![1..=2]);
    /// assert_eq!(Bits::from_uint(0b10100).to_bit_ranges(), vec![2..=2, 4..=4]);
    /// ```
    pub fn to_bit_ranges(&self) -> Vec<RangeInclusive<usize>> {
        numbers_as_ranges(self.to_bit_positions())
    }

    /// Convert to a list of inclusive ranges that efficiently cover all bits that are set.
    ///
    /// Example:
    /// ```rust
    /// # use reginald_utils::Bits;
    /// # use reginald_utils::RangeStyle;
    /// assert_eq!(Bits::from_uint(0b10110).to_bit_ranges_str(RangeStyle::Rust),          "4, 1..3");
    /// assert_eq!(Bits::from_uint(0b10110).to_bit_ranges_str(RangeStyle::RustInclusive), "4, 1..=2");
    /// assert_eq!(Bits::from_uint(0b10110).to_bit_ranges_str(RangeStyle::Verilog),       "4, 2:1");
    /// ```
    pub fn to_bit_ranges_str(&self, style: RangeStyle) -> String {
        let mut b = self.to_bit_ranges();
        b.reverse();
        ranges_to_str(&b, style)
    }

    /// Get the byte at a given position (interpreted with the specified endianess).
    ///
    /// Example:
    /// ```rust
    /// # use reginald_utils::Bits;
    /// # use reginald_utils::Endianess;
    /// assert_eq!(Bits::from_uint(0x00ABCDEF).get_byte(Endianess::Little, 0, 4), 0xEF);
    /// assert_eq!(Bits::from_uint(0x00ABCDEF).get_byte(Endianess::Little, 1, 4), 0xCD);
    /// assert_eq!(Bits::from_uint(0x00ABCDEF).get_byte(Endianess::Little, 2, 4), 0xAB);
    /// assert_eq!(Bits::from_uint(0x00ABCDEF).get_byte(Endianess::Little, 4, 4), 0x00);
    /// assert_eq!(Bits::from_uint(0x00ABCDEF).get_byte(Endianess::Big, 0, 3), 0xAB);
    /// assert_eq!(Bits::from_uint(0x00ABCDEF).get_byte(Endianess::Big, 1, 3), 0xCD);
    /// assert_eq!(Bits::from_uint(0x00ABCDEF).get_byte(Endianess::Big, 2, 3), 0xEF);
    /// ```
    ///
    /// Be aware that requesting a byte at a big-endian position greater or equal to byte_width
    /// will panic:
    /// ```should_panic
    /// # use reginald_utils::Bits;
    /// # use reginald_utils::Endianess;
    /// assert_eq!(Bits::from_uint(0x00ABCDEF).width_bytes(), 0xEF);
    /// Bits::from_uint(0x00ABCDEF).get_byte(Endianess::Big, 4, 3); // panics
    /// ```
    pub fn get_byte(&self, endian: Endianess, byte_pos: usize, width_bytes: usize) -> u8 {
        let le_byte_pos = match endian {
            Endianess::Little => byte_pos,
            Endianess::Big => width_bytes - byte_pos - 1,
        };

        let digit = (le_byte_pos * 8) / DIGIT_SIZE;
        let bit_offset = (le_byte_pos * 8) % DIGIT_SIZE;

        if digit < self.digits.len() {
            ((self.digits[digit] >> bit_offset) & 0xff) as u8
        } else {
            0
        }
    }

    pub fn get_le_byte(&self, byte_pos: usize) -> u8 {
        self.get_byte(Endianess::Little, byte_pos, self.width_bytes())
    }

    pub fn get_be_byte(&self, byte_pos: usize, width_bytes: usize) -> u8 {
        self.get_byte(Endianess::Big, byte_pos, width_bytes)
    }

    fn trim(&mut self) {
        while let Some(last) = self.digits.last() {
            if *last == 0 {
                self.digits.pop();
            } else {
                break;
            }
        }
    }

    fn bit_capacity(&self) -> usize {
        self.digits.len() * DIGIT_SIZE
    }
}

impl std::fmt::Debug for Bits {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let bits_str = format!("[{}]", self.to_bit_ranges_str(RangeStyle::RustInclusive));
        f.debug_struct("Bits").field("digits", &bits_str).finish()
    }
}

macro_rules! forward_op_to_opassign {
    ($trait_op:ident, $method_op:ident,  $trait_opassign:ident, $method_opassign:ident) => {
        impl $trait_opassign<Bits> for Bits {
            fn $method_opassign(&mut self, rhs: Bits) {
                self.$method_opassign(&rhs);
            }
        }

        impl $trait_op<&Bits> for &Bits {
            type Output = Bits;

            fn $method_op(self, rhs: &Bits) -> Self::Output {
                let mut r = self.clone();
                r.$method_opassign(rhs);
                r
            }
        }

        impl $trait_op<Bits> for &Bits {
            type Output = Bits;

            fn $method_op(self, rhs: Bits) -> Self::Output {
                let mut r = self.clone();
                r.$method_opassign(&rhs);
                r
            }
        }

        impl $trait_op<Bits> for Bits {
            type Output = Bits;

            fn $method_op(self, rhs: Bits) -> Self::Output {
                let mut r = self.clone();
                r.$method_opassign(&rhs);
                r
            }
        }
    };
}

impl BitAndAssign<&Bits> for Bits {
    fn bitand_assign(&mut self, rhs: &Bits) {
        let len_digits = usize::max(self.digits.len(), rhs.digits.len());
        for pos in 0..len_digits {
            match (self.digits.get_mut(pos), rhs.digits.get(pos)) {
                (Some(lhs), Some(rhs)) => *lhs = *lhs & rhs, // Actual bitand
                (None, Some(_)) => (),                       // relevant digit is already zero.
                (Some(lsh), None) => *lsh = 0,               // bitand with zero is always zero.
                (None, None) => unreachable!(),              // Guarded by max calculation above.
            }
        }
        self.trim();
    }
}

forward_op_to_opassign!(BitAnd, bitand, BitAndAssign, bitand_assign);

impl BitOrAssign<&Bits> for Bits {
    fn bitor_assign(&mut self, rhs: &Bits) {
        let len_digit = usize::max(self.digits.len(), rhs.digits.len());
        for pos in 0..len_digit {
            match (self.digits.get_mut(pos), rhs.digits.get(pos)) {
                (Some(lhs), Some(rhs)) => *lhs = *lhs | rhs, // Actual bitor
                (None, Some(rhs)) => self.digits.push(*rhs), // Added new digit through bitor
                (Some(_), None) => (),                       // bitor with zero noop
                (None, None) => unreachable!(),              // Guarded by max calculation above.
            }
        }
        self.trim();
    }
}

forward_op_to_opassign!(BitOr, bitor, BitOrAssign, bitor_assign);

impl ShlAssign<usize> for Bits {
    fn shl_assign(&mut self, rhs: usize) {
        for bit in (0..self.bit_capacity()).rev() {
            self.write_bit(bit + rhs, self.get_bit(bit));
        }
        for bit in 0..rhs {
            self.clear_bit(bit)
        }
    }
}

impl Shl<usize> for &Bits {
    type Output = Bits;

    fn shl(self, rhs: usize) -> Self::Output {
        let mut r = self.clone();
        r.shl_assign(rhs);
        r
    }
}

impl Shl<usize> for Bits {
    type Output = Bits;

    fn shl(self, rhs: usize) -> Self::Output {
        let mut r = self.clone();
        r.shl_assign(rhs);
        r
    }
}

impl ShrAssign<usize> for Bits {
    fn shr_assign(&mut self, rhs: usize) {
        for bit in 0..self.bit_capacity() {
            self.write_bit(bit, self.get_bit(bit + rhs));
        }
    }
}

impl Shr<usize> for &Bits {
    type Output = Bits;

    fn shr(self, rhs: usize) -> Self::Output {
        let mut r = self.clone();
        r.shr_assign(rhs);
        r
    }
}

impl Shr<usize> for Bits {
    type Output = Bits;

    fn shr(self, rhs: usize) -> Self::Output {
        let mut r = self.clone();
        r.shr_assign(rhs);
        r
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_bitpos() {
        let test_bit_width = DIGIT_SIZE * 2 + 4;

        for set_pos in 0..test_bit_width {
            let b = Bits::from_bitpos(set_pos);
            for check_pos in 0..test_bit_width {
                if set_pos == check_pos {
                    assert!(b.get_bit(check_pos));
                } else {
                    assert!(!b.get_bit(check_pos));
                }
            }
        }
    }

    #[test]
    fn from_bitrange() {
        let test_bit_width = DIGIT_SIZE * 2 + 4;
        let ranges = [4..=8, 1..=1, 0..=0, 0..=test_bit_width - 1, 32..=test_bit_width - 1];

        for test_range in ranges {
            let b = Bits::from_range(test_range.clone());
            for check_pos in 0..test_bit_width {
                if test_range.contains(&check_pos) {
                    assert!(b.get_bit(check_pos));
                } else {
                    assert!(!b.get_bit(check_pos));
                }
            }
        }
    }

    #[test]
    fn from_uint() {
        let test_bit_width = DIGIT_SIZE * 2 + 4;

        let b = Bits::from_uint(0b0);
        for check_pos in 0..test_bit_width {
            assert!(!b.get_bit(check_pos));
        }

        let b = Bits::from_uint(0b1);
        assert!(b.get_bit(0));
        for check_pos in 1..test_bit_width {
            assert!(!b.get_bit(check_pos));
        }

        let b = Bits::from_uint(0b1 << (DIGIT_SIZE - 1));
        for check_pos in 1..test_bit_width {
            assert_eq!(check_pos == DIGIT_SIZE - 1, b.get_bit(check_pos));
        }

        let b = Bits::from_uint(0b1 << DIGIT_SIZE);
        for check_pos in 1..test_bit_width {
            assert_eq!(check_pos == DIGIT_SIZE, b.get_bit(check_pos));
        }
    }

    #[test]
    fn lsb_pos() {
        assert_eq!(Bits::from_uint(0x0).lsb_pos(), 0);
        assert_eq!(Bits::from_uint(0x1).lsb_pos(), 0);
        assert_eq!(Bits::from_uint(0x2).lsb_pos(), 1);
        assert_eq!(Bits::from_uint(0x3).lsb_pos(), 0);
        assert_eq!(Bits::from_uint(0x10).lsb_pos(), 4);
        assert_eq!(Bits::from_uint(0x1F).lsb_pos(), 0);
        assert_eq!(Bits::from_uint(0x20).lsb_pos(), 5);
    }

    #[test]
    fn msb_pos() {
        assert_eq!(Bits::from_uint(0x0).msb_pos(), 0);
        assert_eq!(Bits::from_uint(0x1).msb_pos(), 0);
        assert_eq!(Bits::from_uint(0x2).msb_pos(), 1);
        assert_eq!(Bits::from_uint(0x3).msb_pos(), 1);
        assert_eq!(Bits::from_uint(0x10).msb_pos(), 4);
        assert_eq!(Bits::from_uint(0x1F).msb_pos(), 4);
        assert_eq!(Bits::from_uint(0x20).msb_pos(), 5);
        assert_eq!(Bits::from_range(0..=5).msb_pos(), 5);
        assert_eq!(Bits::from_range(0..=500).msb_pos(), 500);
    }

    #[test]
    fn bitand() {
        assert_eq!(Bits::from_uint(0xDEADBEEF) & Bits::from_uint(0xABFF0123), Bits::from_uint(0xDEADBEEF & 0xABFF0123));
        assert_eq!(Bits::from_bitpos(1000) & Bits::from_uint(0x1), Bits::from_uint(0));
        assert_eq!(Bits::from_uint(0x1) & Bits::from_bitpos(1000), Bits::from_uint(0));
        assert_eq!(Bits::from_bitpos(1000) & Bits::from_range(999..=1001), Bits::from_bitpos(1000));
    }

    #[test]
    fn bitor() {
        assert_eq!(Bits::from_uint(0xDEADBEEF) | Bits::from_uint(0xABFF0123), Bits::from_uint(0xDEADBEEF | 0xABFF0123));
        assert_eq!(Bits::from_uint(0x0) | Bits::from_bitpos(1000), Bits::from_bitpos(1000));
        assert_eq!(Bits::from_bitpos(1000) | Bits::from_uint(0), Bits::from_bitpos(1000));
        assert_eq!(Bits::from_bitpos(1000) & Bits::from_range(999..=1001), Bits::from_bitpos(1000));
    }

    #[test]
    fn shl() {
        assert_eq!(Bits::from_uint(0x123) << 0, Bits::from_uint(0x123));
        assert_eq!(Bits::from_uint(0x123) << 1, Bits::from_uint(0x123 << 1));
        assert_eq!(Bits::from_uint(0x123) << 4, Bits::from_uint(0x123 << 4));
        assert_eq!(Bits::from_uint(0x123) << 32, Bits::from_uint(0x123 << 32));
        assert_eq!(Bits::from_uint(0x123) << 80, Bits::from_uint(0x123 << 80));
    }

    #[test]
    fn shr() {
        assert_eq!(Bits::from_uint(0x123) >> 0, Bits::from_uint(0x123));
        assert_eq!(Bits::from_uint(0x123) >> 4, Bits::from_uint(0x123 >> 4));
        assert_eq!(Bits::from_uint(0x123) >> 32, Bits::from_uint(0x123 >> 32));
        assert_eq!(Bits::from_uint(0x123 << 28) >> 0, Bits::from_uint(0x123 << 28));
        assert_eq!(Bits::from_uint(0x123 << 28) >> 4, Bits::from_uint((0x123 << 28) >> 4));
        assert_eq!(Bits::from_uint(0x123 << 28) >> 32, Bits::from_uint((0x123 << 28) >> 32));
    }

    #[test]
    fn grab_bytes() {
        // Length 1:
        assert_eq!(Bits::from_uint(0xAF).get_byte(Endianess::Little, 0, 1), 0xAF);
        assert_eq!(Bits::from_uint(0xAF).get_byte(Endianess::Big, 0, 1), 0xAF);

        // Length 2:
        assert_eq!(Bits::from_uint(0xBEEF).get_byte(Endianess::Little, 0, 2), 0xEF);
        assert_eq!(Bits::from_uint(0xBEEF).get_byte(Endianess::Little, 1, 2), 0xBE);
        assert_eq!(Bits::from_uint(0xBEEF).get_byte(Endianess::Big, 0, 2), 0xBE);
        assert_eq!(Bits::from_uint(0xBEEF).get_byte(Endianess::Big, 1, 2), 0xEF);

        // Length 3:
        assert_eq!(Bits::from_uint(0xDEADBE).get_byte(Endianess::Little, 0, 3), 0xBE);
        assert_eq!(Bits::from_uint(0xDEADBE).get_byte(Endianess::Little, 1, 3), 0xAD);
        assert_eq!(Bits::from_uint(0xDEADBE).get_byte(Endianess::Little, 2, 3), 0xDE);
        assert_eq!(Bits::from_uint(0xDEADBE).get_byte(Endianess::Big, 0, 3), 0xDE);
        assert_eq!(Bits::from_uint(0xDEADBE).get_byte(Endianess::Big, 1, 3), 0xAD);
        assert_eq!(Bits::from_uint(0xDEADBE).get_byte(Endianess::Big, 2, 3), 0xBE);

        // Length 4:
        assert_eq!(Bits::from_uint(0xDEADBEEF).get_byte(Endianess::Little, 0, 4), 0xEF);
        assert_eq!(Bits::from_uint(0xDEADBEEF).get_byte(Endianess::Little, 1, 4), 0xBE);
        assert_eq!(Bits::from_uint(0xDEADBEEF).get_byte(Endianess::Little, 2, 4), 0xAD);
        assert_eq!(Bits::from_uint(0xDEADBEEF).get_byte(Endianess::Little, 3, 4), 0xDE);
        assert_eq!(Bits::from_uint(0xDEADBEEF).get_byte(Endianess::Big, 0, 4), 0xDE);
        assert_eq!(Bits::from_uint(0xDEADBEEF).get_byte(Endianess::Big, 1, 4), 0xAD);
        assert_eq!(Bits::from_uint(0xDEADBEEF).get_byte(Endianess::Big, 2, 4), 0xBE);
        assert_eq!(Bits::from_uint(0xDEADBEEF).get_byte(Endianess::Big, 3, 4), 0xEF);
    }
}
