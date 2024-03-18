#![no_std]
use core::{convert::Infallible, usize};

pub trait ToBytes<const N: usize> {
    fn to_le_bytes(&self) -> [u8; N];
    fn to_be_bytes(&self) -> [u8; N] {
        let mut val = self.to_le_bytes();
        val.reverse();
        val
    }
}

pub trait TryFromBytes<const N: usize>: Sized {
    type Error;

    fn try_from_le_bytes(val: [u8; N]) -> Result<Self, Self::Error>;
    fn try_from_be_bytes(val: [u8; N]) -> Result<Self, Self::Error> {
        let mut val = val;
        val.reverse();
        Self::try_from_le_bytes(val)
    }
}

pub trait FromBytes<const N: usize>: Sized {
    fn from_le_bytes(val: [u8; N]) -> Self;
    fn from_be_bytes(val: [u8; N]) -> Self {
        let mut val = val;
        val.reverse();
        Self::from_le_bytes(val)
    }
}

impl<const N: usize, T> TryFromBytes<N> for T
where
    T: FromBytes<N>,
{
    type Error = Infallible;
    fn try_from_le_bytes(val: [u8; N]) -> Result<Self, Self::Error> {
        Ok(Self::from_le_bytes(val))
    }
    fn try_from_be_bytes(val: [u8; N]) -> Result<Self, Self::Error> {
        Ok(Self::from_be_bytes(val))
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
pub enum Stat {
    Cool = 0x1,
    Hot = 0x3,
    NotCool = 0x2,
}

impl TryFrom<u8> for Stat {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x1 => Ok(Self::Cool),
            0x3 => Ok(Self::Hot),
            0x2 => Ok(Self::NotCool),
            _ => Err(()),
        }
    }
}

/// REG1 register
pub mod reg1 {

    /// REG1 register address
    pub const REG1_ADDRESS: u8 = 0x0;

    /// REG1 little-endian reset value
    pub const REG1_RESET_LE: [u8; 2] = [0x0, 0x0];

    /// REG1 big-endian reset value
    pub const REG1_RESET_BE: [u8; 2] = [0x0, 0x0];

    /// REG1 little-endian always write value
    pub const REG1_ALWAYSWRITE_VALUE_LE: [u8; 2] = [0x10, 0x0];

    /// REG1 big-endian always write value
    pub const REG1_ALWAYSWRITE_VALUE_BE: [u8; 2] = [0x0, 0x10];

    #[derive(Clone, Copy, Debug, PartialEq)]
    #[repr(u8)]
    pub enum Field2 {
        En = 0x3,
    }

    impl TryFrom<u8> for Field2 {
        type Error = ();

        fn try_from(value: u8) -> Result<Self, Self::Error> {
            match value {
                0x3 => Ok(Self::En),
                _ => Err(()),
            }
        }
    }

    /// REG1 register
    #[derive(Debug)]
    pub struct Reg1 {
        pub field1: super::Stat,
        pub field2: Field2,
        pub field3: bool,
        pub field4: u8,
    }

    impl super::ToBytes<2> for Reg1 {
        fn to_le_bytes(&self) -> [u8; 2] {
            [REG1_ALWAYSWRITE_VALUE_LE[0], REG1_ALWAYSWRITE_VALUE_LE[1]]
            // let mut val: [u8; 2] = [0; 2];
            // for i in 0..2 {
            //     val[i] |= REG1_ALWAYSWRITE_VALUE_LE[i];
            // }
            // val[0] |= (((self.field1 as u8) << 6) & 0xC0) as u8;
            // val[0] |= ((self.field2 as u8) & 0x3) as u8;
            // val[0] |= (((self.field3 as u8) << 2) & 0x4) as u8;
            // val[1] |= (self.field4 & 0x1F) as u8;
            // val
        }
    }
}
