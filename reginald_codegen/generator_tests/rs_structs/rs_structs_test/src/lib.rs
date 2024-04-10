#![no_std]
#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(dead_code)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::derive_partial_eq_without_eq)]

pub mod out;

// Unused. Included to ensure they compile:
pub mod out_errormsgs;
pub mod out_notraits;

#[cfg(test)]
mod tests {

    #[test]
    fn test_basic_reg1() {
        use crate::out::*;

        // Basic packing:
        let reg = Reg1 {
            field0: true,
            field1: 0xA,
        };

        let expected: [u8; 1] = [
            (0x1 << 0)  // Field 0
            | 0xA << 2, // Field 1
        ];

        assert_eq!(expected, reg.to_le_bytes());
        assert_eq!(expected, reg.to_be_bytes());
        assert_eq!(u8::from_le_bytes(expected), reg.clone().into());

        // Basic unpacking:
        let mut packed = reg.to_le_bytes();
        packed[0] |= 0x3 << 5; // Futz with unused bits

        let reg_unpacked = Reg1::from_le_bytes(packed);
        assert_eq!(reg_unpacked.field0, true);
        assert_eq!(reg_unpacked.field1, 0xA);

        let reg_unpacked = Reg1::from_be_bytes(packed);
        assert_eq!(reg_unpacked.field0, true);
        assert_eq!(reg_unpacked.field1, 0xA);

        let packed_uint = u8::from_le_bytes(packed);

        let reg_unpacked: Reg1 = packed_uint.into();
        assert_eq!(reg_unpacked.field0, true);
        assert_eq!(reg_unpacked.field1, 0xA);

        // Try unpacking:
        let reg_unpacked = Reg1::try_from_le_bytes(packed).unwrap();
        assert_eq!(reg_unpacked.field0, true);
        assert_eq!(reg_unpacked.field1, 0xA);

        let reg_unpacked = Reg1::try_from_be_bytes(packed).unwrap();
        assert_eq!(reg_unpacked.field0, true);
        assert_eq!(reg_unpacked.field1, 0xA);
    }

    #[test]
    fn test_basic_reg2() {
        use crate::out::*;

        // Basic packing:
        let reg = Reg2 {
            field1: Stat::Hot,
            field2: Field2::En,
            field3: true,
            field4: 0xA,
        };

        let b0 = (0x1 << 4)              // Always write
            | ((Stat::Hot as u8) << 6)  // Field 1
            | ((Field2::En as u8) << 0) // Field 2
            | ((1 << 2)); // Field 3
        let b1 = (0xA) << 0;

        let expected_le: [u8; 2] = [b0, b1];
        let expected_be: [u8; 2] = [b1, b0];

        assert_eq!(expected_le, reg.to_le_bytes());
        assert_eq!(expected_be, reg.to_be_bytes());

        // Basic unpacking:
        let mut packed_le = reg.to_le_bytes();
        packed_le[0] |= 0x3 << 4; // mess with always-write
        packed_le[1] |= 0x7 << (13 - 8); // mess with unused bits

        let reg_unpacked = Reg2::try_from_le_bytes(packed_le).unwrap();
        assert_eq!(reg_unpacked.field1, Stat::Hot);
        assert_eq!(reg_unpacked.field2, Field2::En);
        assert_eq!(reg_unpacked.field3, true);
        assert_eq!(reg_unpacked.field4, 0xA);

        let mut packed_be = packed_le;
        packed_be.reverse();

        let reg_unpacked = Reg2::try_from_be_bytes(packed_be).unwrap();
        assert_eq!(reg_unpacked.field1, Stat::Hot);
        assert_eq!(reg_unpacked.field2, Field2::En);
        assert_eq!(reg_unpacked.field3, true);
        assert_eq!(reg_unpacked.field4, 0xA);

        let packed_uint = u16::from_le_bytes(packed_le);

        let reg_unpacked: Reg2 = packed_uint.try_into().unwrap();
        assert_eq!(reg_unpacked.field1, Stat::Hot);
        assert_eq!(reg_unpacked.field2, Field2::En);
        assert_eq!(reg_unpacked.field3, true);
        assert_eq!(reg_unpacked.field4, 0xA);
    }

    #[test]
    fn test_basic_reg3() {
        use crate::out::*;

        // Basic packing:
        let reg = Reg3 {
            field0: 0xCBF,
            field1: 0x81,
        };

        let expected_le: [u8; 8] = [0xBF, 0x0C, 0, 0, 0, 0, 0, 0x81];
        let expected_be: [u8; 8] = [0x81, 0, 0, 0, 0, 0, 0x0C, 0xBF];

        assert_eq!(expected_le, reg.to_le_bytes());
        assert_eq!(expected_be, reg.to_be_bytes());

        // Basic unpacking:
        let packed_le = reg.to_le_bytes();

        let reg_unpacked = Reg3::from_le_bytes(packed_le);
        assert_eq!(reg_unpacked.field0, 0xCBF);
        assert_eq!(reg_unpacked.field1, 0x81);

        let mut packed_be = packed_le;
        packed_be.reverse();

        let reg_unpacked = Reg3::from_be_bytes(packed_be);
        assert_eq!(reg_unpacked.field0, 0xCBF);
        assert_eq!(reg_unpacked.field1, 0x81);

        let packed_uint = u64::from_le_bytes(packed_le);

        let reg_unpacked: Reg3 = packed_uint.into();
        assert_eq!(reg_unpacked.field0, 0xCBF);
        assert_eq!(reg_unpacked.field1, 0x81);
    }

    #[test]
    fn register_validation() {
        use crate::out::*;

        // `STAT` enum in field 1 (bits 7-6) can only be 0x1-0x3.
        // `STAT` enum in field// `STAT` enum in fieldEN` enum in field 2 (bits 1-0) can only be 0x3

        Reg2::try_from_le_bytes([(0x1 << 6) | 0x0, 0]).unwrap_err();
        Reg2::try_from_le_bytes([(0x2 << 6) | 0x1, 0]).unwrap_err();
        Reg2::try_from_le_bytes([(0x3 << 6) | 0x2, 0]).unwrap_err();
        Reg2::try_from_le_bytes([(0x1 << 6) | 0x3, 0]).unwrap();
        Reg2::try_from_le_bytes([(0x0 << 6) | 0x3, 0]).unwrap_err();

        Reg2::try_from((0x1 << 6) | 0x0).unwrap_err();
        Reg2::try_from((0x2 << 6) | 0x1).unwrap_err();
        Reg2::try_from((0x3 << 6) | 0x2).unwrap_err();
        Reg2::try_from((0x1 << 6) | 0x3).unwrap();
        Reg2::try_from((0x0 << 6) | 0x3).unwrap_err();
    }

    #[test]
    fn register_validation_error_msg() {
        use crate::out_errormsgs::*;

        // `STAT` enum in field 1 (bits 7-6) can only be 0x1-0x3.
        // `EN` enum in field 2 (bits 1-0) can only be 0x3
        let err = Reg2::try_from_le_bytes([(0x1 << 6) | 0x0, 0]).unwrap_err();
        assert!(err.contains("Field2 unpack error"));

        let err = Reg2::try_from_le_bytes([(0x2 << 6) | 0x1, 0]).unwrap_err();
        assert!(err.contains("Field2 unpack error"));

        let err = Reg2::try_from_le_bytes([(0x3 << 6) | 0x2, 0]).unwrap_err();
        assert!(err.contains("Field2 unpack error"));

        Reg2::try_from_le_bytes([(0x1 << 6) | 0x3, 0]).unwrap();

        let err = Reg2::try_from_le_bytes([(0x0 << 6) | 0x3, 0]).unwrap_err();
        assert!(err.contains("Stat unpack error"));
    }

    #[test]
    fn enum_validation() {
        use crate::out::*;

        TryInto::<Stat>::try_into(0_u8).unwrap_err();
        TryInto::<Stat>::try_into(1_u8).unwrap();
        TryInto::<Stat>::try_into(2_u8).unwrap();
        TryInto::<Stat>::try_into(3_u8).unwrap();
        TryInto::<Stat>::try_into(4_u8).unwrap_err();

        TryInto::<Field2>::try_into(0_u8).unwrap_err();
        TryInto::<Field2>::try_into(1_u8).unwrap_err();
        TryInto::<Field2>::try_into(2_u8).unwrap_err();
        TryInto::<Field2>::try_into(3_u8).unwrap();
        TryInto::<Field2>::try_into(4_u8).unwrap_err();
    }

    #[test]
    fn enum_validation_error_msgs() {
        use crate::out_errormsgs::*;

        let err = TryInto::<Stat>::try_into(0_u8).unwrap_err();
        assert!(err.contains("Stat unpack error"));

        TryInto::<Stat>::try_into(1_u8).unwrap();
        TryInto::<Stat>::try_into(2_u8).unwrap();
        TryInto::<Stat>::try_into(3_u8).unwrap();

        let err = TryInto::<Stat>::try_into(4_u8).unwrap_err();
        assert!(err.contains("Stat unpack error"));

        let err = TryInto::<Field2>::try_into(0_u8).unwrap_err();
        assert!(err.contains("Field2 unpack error"));

        let err = TryInto::<Field2>::try_into(1_u8).unwrap_err();
        assert!(err.contains("Field2 unpack error"));

        let err = TryInto::<Field2>::try_into(2_u8).unwrap_err();
        assert!(err.contains("Field2 unpack error"));

        TryInto::<Field2>::try_into(3_u8).unwrap();

        let err = TryInto::<Field2>::try_into(4_u8).unwrap_err();
        assert!(err.contains("Field2 unpack error"));
    }

    #[test]
    fn test_shared_layout_basic() {
        use crate::out::*;

        // Packing:
        let reg = BasicSharedLayout {
            shared_field1: 0x4,
            shared_field2: SharedField2::IsOne,
        };

        let expected_le: [u8; 2] = [0x4 << 1, 1 << 2];
        let expected_be: [u8; 2] = [1 << 2, 0x4 << 1];

        assert_eq!(expected_le, reg.to_le_bytes());
        assert_eq!(expected_be, reg.to_be_bytes());

        // Basic unpacking:
        let packed_le = reg.to_le_bytes();

        let reg_unpacked = BasicSharedLayout::from_le_bytes(packed_le);
        assert_eq!(reg_unpacked.shared_field1, 0x4);
        assert_eq!(reg_unpacked.shared_field2, SharedField2::IsOne);

        let mut packed_be = packed_le;
        packed_be.reverse();

        let reg_unpacked = BasicSharedLayout::from_be_bytes(packed_be);
        assert_eq!(reg_unpacked.shared_field1, 0x4);
        assert_eq!(reg_unpacked.shared_field2, SharedField2::IsOne);

        let packed_uint = u16::from_le_bytes(packed_le);

        let reg_unpacked: BasicSharedLayout = packed_uint.into();
        assert_eq!(reg_unpacked.shared_field1, 0x4);
        assert_eq!(reg_unpacked.shared_field2, SharedField2::IsOne);
    }

    #[test]
    fn test_fixed_accross_byte() {
        use crate::out::*;

        // Packing:
        let reg = RegFixedAcrossBytes {};

        let expected_le: [u8; 2] = [0x1 << 6, 2];
        let expected_be: [u8; 2] = [2, 0x1 << 6];
        let expected_uint: u16 = u16::from_le_bytes(expected_le);

        assert_eq!(expected_le, reg.to_le_bytes());
        assert_eq!(expected_be, reg.to_be_bytes());
        assert_eq!(expected_uint, reg.into());
    }

    #[test]
    fn test_layout_fields() {
        use crate::out::*;

        // Basic packing:
        let reg = RegLayoutField {
            layout_field: LayoutField {
                f1: 1,
                f2: F2 { f22: 0xFF },
                f3: F3 {},
            },
        };

        let expected_le: [u8; 2] = [(0x1 << 0) | (0xFF << 2), 0x3];
        let expected_be: [u8; 2] = [0x3, (0x1 << 0) | (0xFF << 2)];

        assert_eq!(expected_le, reg.to_le_bytes());
        assert_eq!(expected_be, reg.to_be_bytes());

        // // Basic unpacking:
        let mut packed_le = reg.to_le_bytes();
        packed_le[0] |= 0xB8;

        let reg_unpacked = RegLayoutField::from_le_bytes(packed_le);
        assert_eq!(reg_unpacked.layout_field.f1, 1);
        assert_eq!(reg_unpacked.layout_field.f2.f22, 0xFF);

        let mut packed_be = packed_le;
        packed_be.reverse();

        let reg_unpacked = RegLayoutField::from_be_bytes(packed_be);
        assert_eq!(reg_unpacked.layout_field.f1, 1);
        assert_eq!(reg_unpacked.layout_field.f2.f22, 0xFF);

        let packed_uint = u16::from_le_bytes(packed_le);

        let reg_unpacked = RegLayoutField::from(packed_uint);
        assert_eq!(reg_unpacked.layout_field.f1, 1);
        assert_eq!(reg_unpacked.layout_field.f2.f22, 0xFF);
    }

    #[test]
    fn test_nested_only_fixed() {
        use crate::out::*;

        // Basic packing:
        let reg = RegNestedOnlyFixed {
            layout_field_1: LayoutField1 {},
        };

        let expected: [u8; 1] = [0xAB];
        assert_eq!(expected, reg.to_le_bytes());
        assert_eq!(expected, reg.to_be_bytes());
        assert_eq!(expected[0], reg.into());
    }

    #[test]
    fn test_split_field() {
        use crate::out::*;

        // Basic packing:
        let reg = RegSplitField {
            split_field_1: 0b010001,
            split_field_2: 0b100010,
        };

        let expected: [u8; 1] = [0b10011001];
        assert_eq!(expected, reg.to_le_bytes());
        assert_eq!(expected, reg.to_be_bytes());
        assert_eq!(expected[0], reg.into());

        // Basic unpacking:
        let reg_unpacked = RegSplitField::from_le_bytes(expected);
        assert_eq!(reg_unpacked.split_field_1, 0b010001);
        assert_eq!(reg_unpacked.split_field_2, 0b100010);
    }

    #[test]
    fn test_split_enum() {
        use crate::out::*;

        // Basic packing:
        let reg = RegSplitEnum {
            split_enum: SplitEnum::Se3,
        };

        let expected: [u8; 1] = [0b101];
        assert_eq!(expected, reg.to_le_bytes());
        assert_eq!(expected, reg.to_be_bytes());
        assert_eq!(expected[0], reg.into());

        // Basic unpacking:
        let packed = [0b101];
        let reg_unpacked = RegSplitEnum::from_le_bytes(packed);
        assert_eq!(reg_unpacked.split_enum, SplitEnum::Se3);

        let packed = [0b111];
        let reg_unpacked = RegSplitEnum::from_le_bytes(packed);
        assert_eq!(reg_unpacked.split_enum, SplitEnum::Se3);

        let packed = [0b1010];
        let reg_unpacked = RegSplitEnum::from_le_bytes(packed);
        assert_eq!(reg_unpacked.split_enum, SplitEnum::Se0);

        // Ensure that the split enum truncating conversion function covers all bits:
        for i in 0..255_u8 {
            SplitEnum::truncated_from(i);
        }
    }

    #[test]
    fn test_split_layout() {
        use crate::out::*;

        // Basic packing:
        let reg = RegSplitLayout {
            split_layout: SplitLayout { f1: 0x3, f2: 0x7 },
        };

        let expected: [u8; 1] = [0b11101100];
        assert_eq!(expected, reg.to_le_bytes());
        assert_eq!(expected, reg.to_be_bytes());
        assert_eq!(expected[0], reg.into());

        // Basic unpacking:
        let reg_unpacked = RegSplitLayout::from_le_bytes(expected);
        assert_eq!(reg_unpacked.split_layout.f1, 0x3);
        assert_eq!(reg_unpacked.split_layout.f2, 0x7);
    }
}
