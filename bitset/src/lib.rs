pub use bitset_derive::*;

use std::{
    fmt::Debug,
    ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not, Sub, SubAssign},
};

pub trait Bits:
    Sized
    + Clone
    + Copy
    + PartialEq
    + Eq
    + BitOr<Output = Self>
    + BitOrAssign
    + BitAnd<Output = Self>
    + BitAndAssign
    + BitXor<Output = Self>
    + BitXorAssign
    + Not<Output = Self>
{
    const ZERO: Self;
    const COUNT: usize;

    fn bit(index: usize) -> Self;
    fn trailing_zeros(self) -> usize;
}

macro_rules! impl_bits {
    ($( $t:ty  ),*) => { $(
        impl Bits for $t {
            const ZERO: Self = 0;
            const COUNT: usize = Self::BITS as usize;
            #[inline(always)] fn bit(index: usize) -> Self { (1 as Self).checked_shl(index as u32).unwrap_or(0)  }
            #[inline(always)] fn trailing_zeros(self) -> usize { self.trailing_zeros() as usize }
        }
    )* }
}

impl_bits!(u8, u16, u32, u64, u128);

pub trait BitFlag: Sized + Copy {
    type Storage: Bits;

    fn bits(self) -> Self::Storage;
    fn from_index(index: usize) -> Option<Self>;
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct BitSet<F: BitFlag + ?Sized>(F::Storage);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GetSingleError {
    Empty,
    TooMany,
    Invalid,
}

impl<F: BitFlag> BitSet<F> {
    #[inline]
    pub fn new() -> Self {
        Self(F::Storage::ZERO)
    }

    pub fn clear(&mut self) {
        self.0 = F::Storage::ZERO;
    }

    #[inline]
    pub fn union(self, rhs: Self) -> Self {
        self | rhs
    }

    #[inline]
    pub fn intersect(self, rhs: Self) -> Self {
        self & rhs
    }

    #[inline]
    pub fn bits(&self) -> F::Storage {
        self.0
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0 == F::Storage::ZERO
    }

    #[inline]
    pub fn invert(&mut self) {
        self.0 = !self.0
    }

    #[inline]
    pub fn get_single(self) -> Result<F, GetSingleError> {
        let mut temp = self.clone();
        if let Some(flag) = temp.pop() {
            if temp.is_empty() {
                Ok(flag)
            } else {
                Err(GetSingleError::TooMany)
            }
        } else {
            if self.is_empty() {
                Err(GetSingleError::Empty)
            } else {
                Err(GetSingleError::Invalid)
            }
        }
    }

    #[inline]
    pub fn pop(&mut self) -> Option<F> {
        let next = self.0.trailing_zeros();
        self.0 &= !F::Storage::bit(next);
        F::from_index(next)
    }

    #[inline]
    pub fn get(&self, flag: F) -> bool {
        flag.bits() & self.0 != F::Storage::ZERO
    }

    #[inline]
    pub fn set(&mut self, flag: F) -> bool {
        let result = self.get(flag);
        *self |= flag;
        result
    }
}

impl<F: BitFlag + Debug> Debug for BitSet<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(std::any::type_name::<Self>())?;
        f.debug_set().entries(*self).finish()
    }
}

impl<F: BitFlag> From<F> for BitSet<F> {
    #[inline]
    fn from(flag: F) -> Self {
        BitSet(flag.bits())
    }
}

impl<F: BitFlag> BitAnd for BitSet<F> {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}
impl<F: BitFlag> BitAnd<F> for BitSet<F> {
    type Output = bool;

    fn bitand(self, rhs: F) -> Self::Output {
        self.get(rhs)
    }
}

impl<F: BitFlag, I: Into<BitSet<F>>> BitAndAssign<I> for BitSet<F> {
    fn bitand_assign(&mut self, rhs: I) {
        self.0 = self.0 & rhs.into().0
    }
}

impl<F: BitFlag, I: Into<BitSet<F>>> BitOr<I> for BitSet<F> {
    type Output = Self;

    fn bitor(self, rhs: I) -> Self::Output {
        Self(self.0 | rhs.into().0)
    }
}

impl<F: BitFlag, I: Into<BitSet<F>>> BitOrAssign<I> for BitSet<F> {
    fn bitor_assign(&mut self, rhs: I) {
        self.0 |= rhs.into().0
    }
}

impl<F: BitFlag, I: Into<BitSet<F>>> BitXor<I> for BitSet<F> {
    type Output = Self;
    fn bitxor(self, rhs: I) -> Self::Output {
        Self(self.0 ^ rhs.into().0)
    }
}

impl<F: BitFlag, I: Into<BitSet<F>>> BitXorAssign<I> for BitSet<F> {
    fn bitxor_assign(&mut self, rhs: I) {
        self.0 ^= rhs.into().0
    }
}

impl<F: BitFlag, I: Into<BitSet<F>>> Sub<I> for BitSet<F> {
    type Output = Self;
    fn sub(self, rhs: I) -> Self::Output {
        Self(self.0 & !rhs.into().0)
    }
}

impl<F: BitFlag, I: Into<BitSet<F>>> SubAssign<I> for BitSet<F> {
    fn sub_assign(&mut self, rhs: I) {
        self.0 &= !rhs.into().0
    }
}
impl<F: BitFlag> Not for BitSet<F> {
    type Output = Self;
    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}

impl<F: BitFlag> Iterator for BitSet<F> {
    type Item = F;

    fn next(&mut self) -> Option<Self::Item> {
        self.pop()
    }
}

impl<F: BitFlag> Extend<F> for BitSet<F> {
    fn extend<T>(&mut self, iter: T)
    where
        T: IntoIterator<Item = F>,
    {
        for flag in iter {
            *self |= flag;
        }
    }
}

impl<F: BitFlag> FromIterator<F> for BitSet<F> {
    fn from_iter<T: IntoIterator<Item = F>>(iter: T) -> Self {
        let mut set = BitSet::new();
        set.extend(iter);
        set
    }
}
