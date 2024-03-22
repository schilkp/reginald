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
