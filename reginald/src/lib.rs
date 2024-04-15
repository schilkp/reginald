use core::{convert::Infallible, usize};

// Struct to bytes converstion:
pub trait ToBytes<const N: usize>: Sized {
    #[inline(always)]
    fn to_le_bytes(&self) -> [u8; N] {
        let mut val = self.to_be_bytes();
        val.reverse();
        val
    }

    #[inline(always)]
    fn to_be_bytes(&self) -> [u8; N] {
        let mut val = self.to_le_bytes();
        val.reverse();
        val
    }
}

// Bytes to struct conversion (fallible):
pub trait TryFromBytes<const N: usize>: Sized {
    type Error;

    #[inline(always)]
    fn try_from_le_bytes(val: &[u8; N]) -> Result<Self, Self::Error> {
        let mut val = *val;
        val.reverse();
        Self::try_from_be_bytes(&val)
    }

    #[inline(always)]
    fn try_from_be_bytes(val: &[u8; N]) -> Result<Self, Self::Error> {
        let mut val = *val;
        val.reverse();
        Self::try_from_le_bytes(&val)
    }
}

// Bytes to struct conversion (infallible):
pub trait FromBytes<const N: usize>: Sized {
    #[inline(always)]
    fn from_le_bytes(val: &[u8; N]) -> Self {
        let mut val = *val;
        val.reverse();
        Self::from_be_bytes(&val)
    }

    #[inline(always)]
    fn from_be_bytes(val: &[u8; N]) -> Self {
        let mut val = *val;
        val.reverse();
        Self::from_le_bytes(&val)
    }
}

// Implement fallible conversion for infallible conversion:
impl<const N: usize, T> TryFromBytes<N> for T
where
    T: FromBytes<N>,
{
    type Error = Infallible;

    #[inline(always)]
    fn try_from_le_bytes(val: &[u8; N]) -> Result<Self, Self::Error> {
        Ok(Self::from_le_bytes(val))
    }

    #[inline(always)]
    fn try_from_be_bytes(val: &[u8; N]) -> Result<Self, Self::Error> {
        Ok(Self::from_be_bytes(val))
    }
}

// Bytes to struct conversion (infallible, but possibly lossy):
pub trait FromBytesLossy<const N: usize>: Sized {
    #[inline(always)]
    fn from_le_bytes_lossy(val: &[u8; N]) -> Self {
        let mut val = *val;
        val.reverse();
        Self::from_be_bytes_lossy(&val)
    }

    #[inline(always)]
    fn from_be_bytes_lossy(val: &[u8; N]) -> Self {
        let mut val = *val;
        val.reverse();
        Self::from_le_bytes_lossy(&val)
    }
}

// Implement possibly conversion for infallible conversion:
impl<const N: usize, T> FromBytesLossy<N> for T
where
    T: FromBytes<N>,
{
    #[inline(always)]
    fn from_le_bytes_lossy(val: &[u8; N]) -> Self {
        Self::from_le_bytes(val)
    }

    #[inline(always)]
    fn from_be_bytes_lossy(val: &[u8; N]) -> Self {
        Self::from_be_bytes(val)
    }
}

// Register properties:
pub enum ResetVal<const N: usize> {
    BigEndian([u8; N]),
    LittleEndian([u8; N]),
}

pub trait Register<const N: usize, A> {
    const ADDRESS: A;
    const RESET_VAL: Option<ResetVal<N>>;

    fn reset_val_le() -> Option<[u8; N]> {
        match Self::RESET_VAL {
            None => None,
            Some(ResetVal::LittleEndian(val_le)) => Some(val_le),
            Some(ResetVal::BigEndian(val)) => {
                let mut val = val;
                val.reverse();
                Some(val)
            }
        }
    }

    fn reset_val_be() -> Option<[u8; N]> {
        match Self::RESET_VAL {
            None => None,
            Some(ResetVal::LittleEndian(val)) => {
                let mut val = val;
                val.reverse();
                Some(val)
            }
            Some(ResetVal::BigEndian(val_be)) => Some(val_be),
        }
    }
}
