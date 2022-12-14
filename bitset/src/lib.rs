#![no_std]
pub use bitset_derive::*;

use core::{
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
    fn count_ones(self) -> usize;
}

macro_rules! impl_bits {
    ($( $t:ty  ),*) => { $(
        impl Bits for $t {
            const ZERO: Self = 0;
            const COUNT: usize = Self::BITS as usize;
            #[inline(always)] fn bit(index: usize) -> Self { (1 as Self).checked_shl(index as u32).unwrap_or(0)  }
            #[inline(always)] fn trailing_zeros(self) -> usize { self.trailing_zeros() as usize }
            #[inline(always)] fn count_ones(self) -> usize { self.count_ones() as usize }
        }
    )* }
}

impl_bits!(u8, u16, u32, u64, u128);

pub trait BitFlag: Sized + Copy {
    type Storage: Bits;

    fn bits(self) -> Self::Storage;
    fn from_index(index: usize) -> Option<Self>;
}

pub struct BitSet<F: BitFlag + ?Sized>(F::Storage);

// Derive uses type parameter bounds rather than the associated type
// So we have to implement these ourself to avoid unnecessary bounds
// https://github.com/rust-lang/rust/issues/26925

impl<F: BitFlag> Clone for BitSet<F> {
    fn clone(&self) -> Self {
        BitSet(self.0)
    }
}

impl<F: BitFlag> Copy for BitSet<F> {}

impl<F: BitFlag> PartialEq for BitSet<F> {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl<F: BitFlag> Eq for BitSet<F> {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GetSingleError {
    Empty,
    TooMany,
}

impl<F: BitFlag> BitSet<F> {
    #[inline]
    pub fn new() -> Self {
        Self(F::Storage::ZERO)
    }

    #[inline]
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
    pub fn is_subset(self, superset: Self) -> bool {
        self & superset == self
    }

    #[inline]
    pub fn count(&self) -> usize {
        self.0.count_ones()
    }

    #[inline]
    pub fn bits(&self) -> &F::Storage {
        &self.0
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
            Err(GetSingleError::Empty)
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

    #[inline]
    pub fn single(flag: F) -> Self {
        BitSet(flag.bits())
    }
}

impl<F: BitFlag + Debug> Debug for BitSet<F> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("BitSet<{}>", core::any::type_name::<F>()))?;
        f.debug_set().entries(*self).finish()
    }
}

impl<F: BitFlag> From<F> for BitSet<F> {
    #[inline]
    fn from(flag: F) -> Self {
        BitSet::single(flag)
    }
}

impl<F: BitFlag> BitAnd for BitSet<F> {
    type Output = Self;

    #[inline]
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl<F: BitFlag> BitAnd<F> for BitSet<F> {
    type Output = bool;

    #[inline]
    fn bitand(self, rhs: F) -> Self::Output {
        self.get(rhs)
    }
}

impl<F: BitFlag, I: Into<BitSet<F>>> BitAndAssign<I> for BitSet<F> {
    #[inline]
    fn bitand_assign(&mut self, rhs: I) {
        self.0 = self.0 & rhs.into().0
    }
}

impl<F: BitFlag, I: Into<BitSet<F>>> BitOr<I> for BitSet<F> {
    type Output = Self;

    #[inline]
    fn bitor(self, rhs: I) -> Self::Output {
        Self(self.0 | rhs.into().0)
    }
}

impl<F: BitFlag, I: Into<BitSet<F>>> BitOrAssign<I> for BitSet<F> {
    #[inline]
    fn bitor_assign(&mut self, rhs: I) {
        self.0 |= rhs.into().0
    }
}

impl<F: BitFlag, I: Into<BitSet<F>>> BitXor<I> for BitSet<F> {
    type Output = Self;
    #[inline]
    fn bitxor(self, rhs: I) -> Self::Output {
        Self(self.0 ^ rhs.into().0)
    }
}

impl<F: BitFlag, I: Into<BitSet<F>>> BitXorAssign<I> for BitSet<F> {
    #[inline]
    fn bitxor_assign(&mut self, rhs: I) {
        self.0 ^= rhs.into().0
    }
}

impl<F: BitFlag, I: Into<BitSet<F>>> Sub<I> for BitSet<F> {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: I) -> Self::Output {
        Self(self.0 & !rhs.into().0)
    }
}

impl<F: BitFlag, I: Into<BitSet<F>>> SubAssign<I> for BitSet<F> {
    #[inline]
    fn sub_assign(&mut self, rhs: I) {
        self.0 &= !rhs.into().0
    }
}
impl<F: BitFlag> Not for BitSet<F> {
    type Output = Self;

    #[inline]
    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}

impl<F: BitFlag> Iterator for BitSet<F> {
    type Item = F;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.pop()
    }
}

impl<F: BitFlag> Extend<F> for BitSet<F> {
    #[inline]
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
    #[inline]
    fn from_iter<T: IntoIterator<Item = F>>(iter: T) -> Self {
        let mut set = BitSet::new();
        set.extend(iter);
        set
    }
}
