use std::fmt::Display;

#[cfg(feature = "cli")]
use clap::ValueEnum;

use crate::regmap::{TypeBitwidth, TypeValue};

// FIXME Remove this and port to reginald_utils impls with bits.

#[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord)]
#[cfg_attr(feature = "cli", derive(ValueEnum))]
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

pub fn grab_byte(endian: Endianess, val: TypeValue, byte_pos: TypeBitwidth, width_bytes: TypeBitwidth) -> u8 {
    let le_byte_pos = match endian {
        Endianess::Little => byte_pos,
        Endianess::Big => width_bytes - byte_pos - 1,
    };
    ((val >> (le_byte_pos * 8)) & 0xff) as u8
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ShiftDirection {
    Left,
    Right,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Transform {
    pub shift: Option<(ShiftDirection, TypeBitwidth)>,
    pub mask: u8,
}

/// Determine the transform required to put a field's value into a given byte of
/// a packed byte array.
///
/// Given a register of width `packed_width_bytes`, and a field that exists of
/// the shape `field_mask` at postion 'field_pos' that is 'field_byte_width' bytes
/// wide:
/// This function determines if the byte at position `packed_byte_pos` contains
/// any part of the given field, and if so determines the required transform to
/// extract that part of the field and put it into the correct position. The byte
/// position is interpreted with respect to the endianess given in 'endian'.
///
/// If this function returns none, the field does not have bits in the packed byte.
/// If this function returns some transform, the bits of the field as they exist
/// in the given byte can be obtained by first shifting the field's value by
/// the given direction and then masking with the given mask.
pub fn field_to_packed_byte_transform(
    endian: Endianess,
    unpos_field_mask: TypeValue,
    field_pos: TypeBitwidth,
    packed_byte: TypeBitwidth,
    packed_width_bytes: TypeBitwidth,
) -> Option<Transform> {
    // Mask to be applied to the field value once shifted into place:
    let mask = grab_byte(endian, unpos_field_mask << field_pos, packed_byte, packed_width_bytes);
    if mask == 0 {
        return None;
    }

    // Convert byte position to little-endian equivalent:
    let le_byte_pos = match endian {
        Endianess::Little => packed_byte,
        Endianess::Big => packed_width_bytes - packed_byte - 1,
    };

    let byte_lsb_pos = 8 * le_byte_pos;

    match TypeBitwidth::cmp(&field_pos, &byte_lsb_pos) {
        std::cmp::Ordering::Equal => Some(Transform { shift: None, mask }),
        std::cmp::Ordering::Greater => Some(Transform {
            shift: Some((ShiftDirection::Left, field_pos - byte_lsb_pos)),
            mask,
        }),
        std::cmp::Ordering::Less => Some(Transform {
            shift: Some((ShiftDirection::Right, byte_lsb_pos - field_pos)),
            mask,
        }),
    }
}

/// Determine the transform required to put the data from a given byte in some field
/// into a given byte of a packed byte array.
///
/// Given a register of width `packed_width_bytes`, and a field that exists of
/// the shape `field_mask` at postion 'field_pos' that is 'field_byte_width' bytes
/// wide:
/// This function determines if the byte at position `packed_byte_pos` contains
/// any part of the given byte of the given field, and if so determines the
/// required transform to extract that part of the field byte and put it into the
/// correct position. The byte position is interpreted with respect to the endianess
/// given in 'endian'.
///
/// If this function returns none, the field's byte does not have bits in the packed byte.
/// If this function returns some transform, the bits of the field as they exist
/// in the given byte can be obtained by first shifting the field's value by
/// the given direction and then masking with the given mask.
pub fn field_byte_to_packed_byte_transform(
    endian: Endianess,
    unpos_field_mask: TypeValue,
    field_pos: TypeBitwidth,
    field_byte: TypeBitwidth,
    field_byte_width: TypeBitwidth,
    packed_byte: TypeBitwidth,
    packed_width_bytes: TypeBitwidth,
) -> Option<Transform> {
    // Mask of the field's byte of interest:
    let field_mask = grab_byte(endian, unpos_field_mask, field_byte, field_byte_width);

    // Calculate the actual bit position of the byte in the field:
    let field_pos = match endian {
        Endianess::Little => field_pos + field_byte * 8,
        Endianess::Big => field_pos + (field_byte_width - field_byte - 1) * 8,
    };

    field_to_packed_byte_transform(endian, field_mask.into(), field_pos, packed_byte, packed_width_bytes)
}

/// Determine the transform required to extract all bits of field's value present
/// in a given byte of a packed byte array.
///
/// Given a register of width `packed_width_bytes`, and a field that exists of
/// the shape `field_mask` at postion 'field_pos' that is 'field_byte_width' bytes
/// wide:
/// This function determines if the byte at position `packed_byte_pos` contains
/// any part of the given field, and if so determines the required transform to
/// extract that part of the byte and put it into the correct position in the field.
/// The byte position is interpreted with respect to the endianess given in 'endian'.
///
/// If this function returns none, the field does not have bits in the packed byte.
/// If this function returns some transform, the bits of the field as they exist
/// in the given byte can be obtained by first masking the bytes's value by
/// the given mask, and then shiftingt by the given shift.
pub fn packed_byte_to_field_transform(
    endian: Endianess,
    unpos_field_mask: TypeValue,
    field_pos: TypeBitwidth,
    packed_byte_pos: TypeBitwidth,
    packed_width_bytes: TypeBitwidth,
) -> Option<Transform> {
    let t = field_to_packed_byte_transform(endian, unpos_field_mask, field_pos, packed_byte_pos, packed_width_bytes)?;

    let shift = t.shift.map(|(dir, amt)| match dir {
        ShiftDirection::Left => (ShiftDirection::Right, amt),
        ShiftDirection::Right => (ShiftDirection::Left, amt),
    });

    Some(Transform { shift, mask: t.mask })
}

/// Determine the transform required to extract all bits of given byte of a given
/// field's value present in a given byte of a packed byte array.
///
/// Given a register of width `packed_width_bytes`, and a field that exists of
/// the shape `field_mask` at postion 'field_pos' that is 'field_byte_width' bytes
/// wide:
/// This function determines if the byte at position `packed_byte_pos` contains
/// any part of the given byte of the given field, and if so determines the required
/// transform to extract that part of the byte and put it into the correct position
/// in the field's byte The byte position is interpreted with respect to the
/// endianess given in 'endian'.
///
/// If this function returns none, the given byte of the field does not have
/// bits in the packed byte. If this function returns some transform, the bits
/// of the given field byte as they exist in the given byte can be obtained by
/// first masking the bytes's value by the given mask, and then shiftingt by
/// the given shift.
pub fn packed_byte_to_field_byte_transform(
    endian: Endianess,
    unpos_field_mask: TypeValue,
    field_pos: TypeBitwidth,
    field_byte: TypeBitwidth,
    field_byte_width: TypeBitwidth,
    packed_byte: TypeBitwidth,
    packed_width_bytes: TypeBitwidth,
) -> Option<Transform> {
    // Mask of the field's byte of interest:
    let field_mask = grab_byte(endian, unpos_field_mask, field_byte, field_byte_width);

    // Calculate the actual bit position of the byte in the field:
    let field_pos = match endian {
        Endianess::Little => field_pos + field_byte * 8,
        Endianess::Big => field_pos + (field_byte_width - field_byte - 1) * 8,
    };

    packed_byte_to_field_transform(endian, field_mask.into(), field_pos, packed_byte, packed_width_bytes)
}

#[cfg(test)]
mod tests {

    use crate::bits::{lsb_pos, unpositioned_mask};

    use super::*;

    #[test]
    fn test_grab_bytes() {
        // Length 1:
        assert_eq!(grab_byte(Endianess::Little, 0xAF, 0, 1), 0xAF);
        assert_eq!(grab_byte(Endianess::Big, 0xAF, 0, 1), 0xAF);

        // Length 2:
        assert_eq!(grab_byte(Endianess::Little, 0xBEEF, 0, 2), 0xEF);
        assert_eq!(grab_byte(Endianess::Little, 0xBEEF, 1, 2), 0xBE);
        assert_eq!(grab_byte(Endianess::Big, 0xBEEF, 0, 2), 0xBE);
        assert_eq!(grab_byte(Endianess::Big, 0xBEEF, 1, 2), 0xEF);

        // Length 3:
        assert_eq!(grab_byte(Endianess::Little, 0xDEADBE, 0, 3), 0xBE);
        assert_eq!(grab_byte(Endianess::Little, 0xDEADBE, 1, 3), 0xAD);
        assert_eq!(grab_byte(Endianess::Little, 0xDEADBE, 2, 3), 0xDE);
        assert_eq!(grab_byte(Endianess::Big, 0xDEADBE, 0, 3), 0xDE);
        assert_eq!(grab_byte(Endianess::Big, 0xDEADBE, 1, 3), 0xAD);
        assert_eq!(grab_byte(Endianess::Big, 0xDEADBE, 2, 3), 0xBE);

        // Length 4:
        assert_eq!(grab_byte(Endianess::Little, 0xDEADBEEF, 0, 4), 0xEF);
        assert_eq!(grab_byte(Endianess::Little, 0xDEADBEEF, 1, 4), 0xBE);
        assert_eq!(grab_byte(Endianess::Little, 0xDEADBEEF, 2, 4), 0xAD);
        assert_eq!(grab_byte(Endianess::Little, 0xDEADBEEF, 3, 4), 0xDE);
        assert_eq!(grab_byte(Endianess::Big, 0xDEADBEEF, 0, 4), 0xDE);
        assert_eq!(grab_byte(Endianess::Big, 0xDEADBEEF, 1, 4), 0xAD);
        assert_eq!(grab_byte(Endianess::Big, 0xDEADBEEF, 2, 4), 0xBE);
        assert_eq!(grab_byte(Endianess::Big, 0xDEADBEEF, 3, 4), 0xEF);
    }

    fn check_field_to_packed_byte_transform(
        field_mask: TypeValue,
        width_bytes: TypeBitwidth,
        expected_le: Vec<Option<Transform>>,
    ) {
        use pretty_assertions::assert_eq;

        let unpos_mask = unpositioned_mask(field_mask);
        let field_pos = lsb_pos(field_mask);

        let mut expect = expected_le;
        let is_le: Vec<Option<Transform>> = (0..width_bytes)
            .map(|x| field_to_packed_byte_transform(Endianess::Little, unpos_mask, field_pos, x, width_bytes))
            .collect();
        assert_eq!(expect, is_le);

        expect.reverse();
        let is_be: Vec<Option<Transform>> = (0..width_bytes)
            .map(|x| field_to_packed_byte_transform(Endianess::Big, unpos_mask, field_pos, x, width_bytes))
            .collect();
        assert_eq!(expect, is_be);
    }

    #[test]
    fn test_field_to_packed_byte_transform() {
        // Length 1 (Aligned)
        let expect_le = vec![Some(Transform {
            shift: None,
            mask: 0x0F,
        })];
        check_field_to_packed_byte_transform(0x0F, 1, expect_le);

        // Length 1 (Misaligned)
        let expect_le = vec![Some(Transform {
            shift: Some((ShiftDirection::Left, 3)),
            mask: 0x18,
        })];
        check_field_to_packed_byte_transform(0x18, 1, expect_le);

        // Length 2 (Aligned)
        let expect_le = vec![
            None,
            Some(Transform {
                shift: None,
                mask: 0xAF,
            }),
        ];
        check_field_to_packed_byte_transform(0xAF00, 2, expect_le);

        // Length 2 (Misaligned)
        let expect_le = vec![
            Some(Transform {
                shift: Some((ShiftDirection::Left, 4)),
                mask: 0xF0,
            }),
            Some(Transform {
                shift: Some((ShiftDirection::Right, 4)),
                mask: 0x0A,
            }),
        ];
        check_field_to_packed_byte_transform(0x0AF0, 2, expect_le);

        // Length 2 (Misaligned)
        let expect_le = vec![
            Some(Transform {
                shift: Some((ShiftDirection::Left, 7)),
                mask: 0x80,
            }),
            Some(Transform {
                shift: Some((ShiftDirection::Right, 1)),
                mask: 0x0A,
            }),
        ];
        check_field_to_packed_byte_transform(0x0A80, 2, expect_le);

        let expect_le = vec![
            Some(Transform {
                shift: Some((ShiftDirection::Left, 7)),
                mask: 0x80,
            }),
            Some(Transform {
                shift: Some((ShiftDirection::Right, 1)),
                mask: 0xFF,
            }),
            Some(Transform {
                shift: Some((ShiftDirection::Right, 9)),
                mask: 0x3,
            }),
            None,
        ];
        check_field_to_packed_byte_transform(0x0003FF80, 4, expect_le);
    }

    #[test]
    fn test_field_byte_to_packed_byte_transform() {
        // positioned field: 0b0011_1111_1111_0000
        let field_mask_unpos = 0x3ff;
        let field_pos = 4;
        let field_byte_width = 2;
        let packed_byte_width = 2;

        let expect = Some(Transform {
            shift: Some((ShiftDirection::Left, 4)),
            mask: 0xF0,
        });
        let is = field_byte_to_packed_byte_transform(
            Endianess::Little,
            field_mask_unpos,
            field_pos,
            0,
            field_byte_width,
            0,
            packed_byte_width,
        );
        assert_eq!(is, expect);

        let expect = None;
        let is = field_byte_to_packed_byte_transform(
            Endianess::Little,
            field_mask_unpos,
            field_pos,
            1,
            field_byte_width,
            0,
            packed_byte_width,
        );
        assert_eq!(is, expect);

        let expect = Some(Transform {
            shift: Some((ShiftDirection::Right, 4)),
            mask: 0x0F,
        });
        let is = field_byte_to_packed_byte_transform(
            Endianess::Little,
            field_mask_unpos,
            field_pos,
            0,
            field_byte_width,
            1,
            packed_byte_width,
        );
        assert_eq!(is, expect);

        let expect = Some(Transform {
            shift: Some((ShiftDirection::Left, 4)),
            mask: 0x30,
        });
        let is = field_byte_to_packed_byte_transform(
            Endianess::Little,
            field_mask_unpos,
            field_pos,
            1,
            field_byte_width,
            1,
            packed_byte_width,
        );
        assert_eq!(is, expect);
    }

    fn check_packed_byte_to_field_transform(
        field_mask: TypeValue,
        width_bytes: TypeBitwidth,
        expected_le: Vec<Option<Transform>>,
    ) {
        use pretty_assertions::assert_eq;

        let unpos_mask = unpositioned_mask(field_mask);
        let field_pos = lsb_pos(field_mask);

        let mut expect = expected_le;
        let is_le: Vec<Option<Transform>> = (0..width_bytes)
            .map(|x| packed_byte_to_field_transform(Endianess::Little, unpos_mask, field_pos, x, width_bytes))
            .collect();
        assert_eq!(expect, is_le);

        expect.reverse();
        let is_be: Vec<Option<Transform>> = (0..width_bytes)
            .map(|x| packed_byte_to_field_transform(Endianess::Big, unpos_mask, field_pos, x, width_bytes))
            .collect();
        assert_eq!(expect, is_be);
    }

    #[test]
    fn test_packed_byte_to_field_transform() {
        // Length 1 (Aligned)
        let expect_le = vec![Some(Transform {
            shift: None,
            mask: 0x0F,
        })];
        check_packed_byte_to_field_transform(0x0F, 1, expect_le);

        // Length 1 (Misaligned)
        let expect_le = vec![Some(Transform {
            shift: Some((ShiftDirection::Right, 3)),
            mask: 0x18,
        })];
        check_packed_byte_to_field_transform(0x18, 1, expect_le);

        // Length 2 (Aligned)
        let expect_le = vec![
            None,
            Some(Transform {
                shift: None,
                mask: 0xAF,
            }),
        ];
        check_packed_byte_to_field_transform(0xAF00, 2, expect_le);

        // Length 2 (Misaligned)
        let expect_le = vec![
            Some(Transform {
                shift: Some((ShiftDirection::Right, 4)),
                mask: 0xF0,
            }),
            Some(Transform {
                shift: Some((ShiftDirection::Left, 4)),
                mask: 0x0A,
            }),
        ];
        check_packed_byte_to_field_transform(0x0AF0, 2, expect_le);

        // Length 2 (Misaligned)
        let expect_le = vec![
            Some(Transform {
                shift: Some((ShiftDirection::Right, 7)),
                mask: 0x80,
            }),
            Some(Transform {
                shift: Some((ShiftDirection::Left, 1)),
                mask: 0x0A,
            }),
        ];
        check_packed_byte_to_field_transform(0x0A80, 2, expect_le);

        // Length 4 (Misaligned)
        let expect_le = vec![
            Some(Transform {
                shift: Some((ShiftDirection::Right, 7)),
                mask: 0x80,
            }),
            Some(Transform {
                shift: Some((ShiftDirection::Left, 1)),
                mask: 0xFF,
            }),
            Some(Transform {
                shift: Some((ShiftDirection::Left, 9)),
                mask: 0x3,
            }),
            None,
        ];
        check_packed_byte_to_field_transform(0x0003FF80, 4, expect_le);
    }

    #[test]
    fn test_packed_byte_to_field_byte_transform() {
        // positioned field: 0b0011_1111_1111_0000
        let field_mask_unpos = 0x3ff;
        let field_pos = 4;
        let field_byte_width = 2;
        let packed_byte_width = 2;

        let expect = Some(Transform {
            shift: Some((ShiftDirection::Right, 4)),
            mask: 0xF0,
        });
        let is = packed_byte_to_field_byte_transform(
            Endianess::Little,
            field_mask_unpos,
            field_pos,
            0,
            field_byte_width,
            0,
            packed_byte_width,
        );
        assert_eq!(is, expect);

        let expect = None;
        let is = packed_byte_to_field_byte_transform(
            Endianess::Little,
            field_mask_unpos,
            field_pos,
            1,
            field_byte_width,
            0,
            packed_byte_width,
        );
        assert_eq!(is, expect);

        let expect = Some(Transform {
            shift: Some((ShiftDirection::Left, 4)),
            mask: 0x0F,
        });
        let is = packed_byte_to_field_byte_transform(
            Endianess::Little,
            field_mask_unpos,
            field_pos,
            0,
            field_byte_width,
            1,
            packed_byte_width,
        );
        assert_eq!(is, expect);

        let expect = Some(Transform {
            shift: Some((ShiftDirection::Right, 4)),
            mask: 0x30,
        });
        let is = packed_byte_to_field_byte_transform(
            Endianess::Little,
            field_mask_unpos,
            field_pos,
            1,
            field_byte_width,
            1,
            packed_byte_width,
        );
        assert_eq!(is, expect);
    }
}
