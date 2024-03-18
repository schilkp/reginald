#![no_std]
#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::derive_partial_eq_without_eq)]

pub mod out;

// Unused. Included to ensure they compile:
pub mod out_errormsgs;
pub mod out_noregblockmods;
pub mod out_notraits;

#[cfg(test)]
mod tests {

    #[test]
    fn convert_8b() {
        use crate::out::reg1::*;
        use crate::out::{FromBytes, ToBytes, TryFromBytes};

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

        // Basic unpacking:
        let mut packed = reg.to_le_bytes();
        packed[0] |= 0x3 << 5; // Futz with unused bits

        let reg_unpacked = Reg1::from_le_bytes(packed);
        assert_eq!(reg_unpacked.field0, true);
        assert_eq!(reg_unpacked.field1, 0xA);

        let reg_unpacked = Reg1::from_be_bytes(packed);
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
    fn convert_16b() {
        use crate::out::reg2::*;
        use crate::out::{Stat, ToBytes, TryFromBytes};

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
    }

    #[test]
    fn convert_64b() {
        use crate::out::reg3::*;
        use crate::out::{FromBytes, ToBytes};

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
    }

    #[test]
    fn register_validation() {
        use crate::out::reg2::*;
        use crate::out::TryFromBytes;

        // `STAT` enum in field 1 (bits 7-6) can only be 0x1-0x3.
        // `STAT` enum in field// `STAT` enum in fieldEN` enum in field 2 (bits 1-0) can only be 0x3
        Reg2::try_from_le_bytes([(0x1 << 6) | 0x0, 0]).unwrap_err();
        Reg2::try_from_le_bytes([(0x2 << 6) | 0x1, 0]).unwrap_err();
        Reg2::try_from_le_bytes([(0x3 << 6) | 0x2, 0]).unwrap_err();
        Reg2::try_from_le_bytes([(0x1 << 6) | 0x3, 0]).unwrap();
        Reg2::try_from_le_bytes([(0x0 << 6) | 0x3, 0]).unwrap_err();
    }

    #[test]
    fn register_validation_error_msg() {
        use crate::out_errormsgs::reg2::*;
        use crate::out_errormsgs::TryFromBytes;

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

        TryInto::<reg2::Field2>::try_into(0_u8).unwrap_err();
        TryInto::<reg2::Field2>::try_into(1_u8).unwrap_err();
        TryInto::<reg2::Field2>::try_into(2_u8).unwrap_err();
        TryInto::<reg2::Field2>::try_into(3_u8).unwrap();
        TryInto::<reg2::Field2>::try_into(4_u8).unwrap_err();
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

        let err = TryInto::<reg2::Field2>::try_into(0_u8).unwrap_err();
        assert!(err.contains("Field2 unpack error"));

        let err = TryInto::<reg2::Field2>::try_into(1_u8).unwrap_err();
        assert!(err.contains("Field2 unpack error"));

        let err = TryInto::<reg2::Field2>::try_into(2_u8).unwrap_err();
        assert!(err.contains("Field2 unpack error"));

        TryInto::<reg2::Field2>::try_into(3_u8).unwrap();

        let err = TryInto::<reg2::Field2>::try_into(4_u8).unwrap_err();
        assert!(err.contains("Field2 unpack error"));
    }
}
