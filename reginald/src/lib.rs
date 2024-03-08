use std::usize;

pub trait Pack<const N: usize> {
    fn pack_le(&self) -> [u8; N];
    fn pack_be(&self) -> [u8; N] {
        let mut val = self.pack_le();
        val.reverse();
        val
    }
}

pub trait TryUnpack<const N: usize>: Sized {
    fn try_unpack_le(val: &[u8; N]) -> Option<Self>;
    fn try_unpack_be(val: &[u8; N]) -> Option<Self> {
        // TODO: Is this smart? Pointless copy?
        let mut val = *val;
        val.reverse();
        Self::try_unpack_le(&val)
    }
}

pub trait Unpack<const N: usize>: Sized {
    fn unpack_le(val: &[u8; N]) -> Self;
    fn unpack_be(val: &[u8; N]) -> Self;
}

impl<const N: usize, T> TryUnpack<N> for T
where
    T: Unpack<N>,
{
    fn try_unpack_le(val: &[u8; N]) -> Option<Self> {
        Some(Self::unpack_le(val))
    }
    fn try_unpack_be(val: &[u8; N]) -> Option<Self> {
        Some(Self::unpack_be(val))
    }
}
