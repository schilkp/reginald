#![warn(clippy::pedantic)]
pub mod out;

#[cfg(test)]
mod tests {
    use crate::out::{bigreg::*, reg1::*, reg2::*};

    use super::out::*;

    #[test]
    fn convert_8b() {
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
        // 'STAT' enum in field 1 (bits 7-6) can only be 0x1-0x3.
        // 'EN' enum in field 2 (bits 1-0) can only be 0x3
        TryInto::<Reg1>::try_into((0x1 << 6) | 0x0).unwrap_err();
        TryInto::<Reg1>::try_into((0x2 << 6) | 0x1).unwrap_err();
        TryInto::<Reg1>::try_into((0x3 << 6) | 0x2).unwrap_err();
        TryInto::<Reg1>::try_into((0x1 << 6) | 0x3).unwrap();
        TryInto::<Reg1>::try_into((0x0 << 6) | 0x3).unwrap_err();
    }

    #[test]
    fn enum_validation() {
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
}
