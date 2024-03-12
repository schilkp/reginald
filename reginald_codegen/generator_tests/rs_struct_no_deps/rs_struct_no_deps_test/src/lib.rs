#![no_std]

pub mod out;

// Unused. Included to ensure they compile:
pub mod out_errormsgs;
pub mod out_noregblockmods;

#[cfg(test)]
mod tests {

    #[test]
    fn convert_8b() {
        use crate::out::reg2::*;

        // Basic packing:
        let reg = Reg2 { field0: true };
        let packed: u8 = reg.into();
        let expected: u8 = 0x1; // Field 0
        assert_eq!(packed, expected);

        // Basic unpacking:
        let packed = packed | (0xF << 1); // Futz with unused bits

        let reg: Reg2 = packed.try_into().unwrap();
        assert_eq!(reg.field0, true);
    }

    #[test]
    fn convert_16b() {
        use crate::out::reg1::*;
        use crate::out::*;
        // Basic packing:
        let reg = Reg1 {
            field1: Stat::Hot,
            field2: reg1::Field2::En,
            field3: true,
            field4: 0xA,
        };
        let packed: u16 = reg.into();
        let expected: u16 = (0x1 << 4)   // Always write
                           | (0x3 << 6)  // Field 1 == "HOT"
                           | (0x3 << 0)  // Field 2 == "EN"
                           | (0x1 << 2)  // Field 2 == "EN"
                           | (0xA << 8); // Field 4 == 0xA
        assert_eq!(packed, expected);

        // Basic unpacking:
        let packed = packed | ((0x7 << 14) | (0x3 << 4)); // Futz with unused/alwayswrite bits.

        let reg: Reg1 = packed.try_into().unwrap();
        assert_eq!(reg.field1, Stat::Hot);
        assert_eq!(reg.field2, Field2::En);
        assert_eq!(reg.field3, true);
        assert_eq!(reg.field4, 0xA);
    }

    #[test]
    fn convert_64b() {
        use crate::out::bigreg::*;

        // Basic packing:
        let reg = Bigreg {
            field0: 0xFFF,
            field1: 0x1F,
        };
        let packed: u64 = reg.into();
        let expected: u64 = 0x1F00000000000FFF;
        assert_eq!(packed, expected);

        // Basic unpacking:
        let packed = packed | (0xF << 20);

        let reg: Bigreg = packed.into();
        assert_eq!(reg.field0, 0xFFF);
        assert_eq!(reg.field1, 0x1F);
    }

    #[test]
    fn register_validation() {
        use crate::out::reg1::*;

        // 'STAT' enum in field 1 (bits 7-6) can only be 0x1-0x3.
        // 'EN' enum in field 2 (bits 1-0) can only be 0x3
        TryInto::<Reg1>::try_into((0x1 << 6) | 0x0).unwrap_err();
        TryInto::<Reg1>::try_into((0x2 << 6) | 0x1).unwrap_err();
        TryInto::<Reg1>::try_into((0x3 << 6) | 0x2).unwrap_err();
        TryInto::<Reg1>::try_into((0x1 << 6) | 0x3).unwrap();
        TryInto::<Reg1>::try_into((0x0 << 6) | 0x3).unwrap_err();
    }

    #[test]
    fn register_validation_error_msg() {
        use crate::out_errormsgs::reg1::*;

        // 'STAT' enum in field 1 (bits 7-6) can only be 0x1-0x3.
        // 'EN' enum in field 2 (bits 1-0) can only be 0x3
        let err = TryInto::<Reg1>::try_into((0x1 << 6) | 0x0).unwrap_err();
        assert!(err.contains("Field2 unpack error"));
        let err = TryInto::<Reg1>::try_into((0x2 << 6) | 0x1).unwrap_err();
        assert!(err.contains("Field2 unpack error"));
        let err = TryInto::<Reg1>::try_into((0x3 << 6) | 0x2).unwrap_err();
        assert!(err.contains("Field2 unpack error"));

        TryInto::<Reg1>::try_into((0x1 << 6) | 0x3).unwrap();
        let err = TryInto::<Reg1>::try_into((0x0 << 6) | 0x3).unwrap_err();
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

        TryInto::<reg1::Field2>::try_into(0_u8).unwrap_err();
        TryInto::<reg1::Field2>::try_into(1_u8).unwrap_err();
        TryInto::<reg1::Field2>::try_into(2_u8).unwrap_err();
        TryInto::<reg1::Field2>::try_into(3_u8).unwrap();
        TryInto::<reg1::Field2>::try_into(4_u8).unwrap_err();
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

        let err = TryInto::<reg1::Field2>::try_into(0_u8).unwrap_err();
        assert!(err.contains("Field2 unpack error"));
        let err = TryInto::<reg1::Field2>::try_into(1_u8).unwrap_err();
        assert!(err.contains("Field2 unpack error"));
        let err = TryInto::<reg1::Field2>::try_into(2_u8).unwrap_err();
        assert!(err.contains("Field2 unpack error"));
        TryInto::<reg1::Field2>::try_into(3_u8).unwrap();
        let err = TryInto::<reg1::Field2>::try_into(4_u8).unwrap_err();
        assert!(err.contains("Field2 unpack error"));
    }
}
