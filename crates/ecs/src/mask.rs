use std::{
    collections::{HashMap, HashSet},
    fmt::{Debug, Display},
    hash::BuildHasherDefault,
    ops::{BitAnd, BitOr, BitXor, Not, Shl, Shr},
};

use nohash_hasher::{IsEnabled, NoHashHasher};

// A mask is a simple 64 bit integer that tells us what components are enabled / disabled from within an entity
// The ECS registry system uses masks to annotate each different type that might be a component, so in total
// In total, there is only 64 different components that can be implemented using this ECS implementation
#[derive(Clone, Copy, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub struct Mask(u64);
impl IsEnabled for Mask {}

impl Mask {
    // Create a mask that has it's bitfield set to one
    pub fn one() -> Mask {
        Mask(0b1)
    }

    // Create a mask that has it's bitfield set to zero
    pub fn zero() -> Mask {
        Mask(0b0)
    }

    // Create a mask that has all of it's bits set
    pub fn all() -> Mask {
        Mask(u64::MAX)
    }

    // Get the offset of this mask, assuming that it is a unit mask
    pub fn offset(&self) -> usize {
        self.0.trailing_zeros() as usize
    }

    // Check if a mask is empty
    pub fn is_zero(&self) -> bool {
        *self == Self::zero()
    }

    // Set a single bit to either true or false
    pub fn set(&mut self, offset: usize, enabled: bool) {
        *self = if enabled {
            // Or
            *self | (Mask::one() << offset)
        } else {
            // Negate and And
            *self & !(Mask::one() << offset)
        }
    }

    // Get a specific bit using an offset
    pub fn get(&self, offset: usize) -> bool {
        (self.0 >> offset) & 1 == 1
    }

    // Check if all the bits from Other are present within Self
    // other: 0100
    // self:  1111
    // true
    pub fn contains(&self, other: Self) -> bool {
        *self & other == other
    }
}

// Convert to raw bitfield
impl Into<u64> for Mask {
    fn into(self) -> u64 {
        self.0
    }
}

// Convert from raw bitfield
impl From<u64> for Mask {
    fn from(bits: u64) -> Self {
        Self(bits)
    }
}

// NoHash hasher that works with Mask
type MaskHasher = BuildHasherDefault<NoHashHasher<Mask>>;
pub(crate) type MaskMap<E> = HashMap<Mask, E, MaskHasher>;
pub(crate) type MaskSet = HashSet<Mask, MaskHasher>;

impl BitAnd for Mask {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Mask(self.0 & rhs.0)
    }
}

impl BitOr for Mask {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Mask(self.0 | rhs.0)
    }
}

impl BitXor for Mask {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Mask(self.0 ^ rhs.0)
    }
}

impl Not for Mask {
    type Output = Self;

    fn not(self) -> Self::Output {
        Mask(!self.0)
    }
}

impl Shl<usize> for Mask {
    type Output = Self;

    fn shl(self, rhs: usize) -> Self::Output {
        Mask(self.0 << rhs)
    }
}

impl Shr<usize> for Mask {
    type Output = Self;

    fn shr(self, rhs: usize) -> Self::Output {
        Mask(self.0 >> rhs)
    }
}

impl Display for Mask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "m{:b}", self.0)
    }
}

impl Debug for Mask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "m{:b}", self.0)
    }
}
