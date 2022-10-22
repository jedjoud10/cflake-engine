use crate::Mask;
use std::ops::{BitAnd, BitOr, BitXor};

// Layout access that contain the shared access mask and unique access mask
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LayoutAccess {
    shared: Mask,
    unique: Mask,
}

impl LayoutAccess {
    // Create a new layout access
    pub fn new(shared: Mask, unique: Mask) -> Self {
        Self { shared, unique }
    }

    // No layout access at all
    pub fn none() -> Self {
        Self {
            shared: Mask::zero(),
            unique: Mask::zero(),
        }
    }

    // Get the shared access mask
    pub fn shared(&self) -> Mask {
        self.shared
    }

    // Get the unique access mask
    pub fn unique(&self) -> Mask {
        self.unique
    }

    // Or the shared and unique masks
    pub fn both(&self) -> Mask {
        self.shared | self.unique
    }
}

impl BitOr for LayoutAccess {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self {
            shared: self.shared | rhs.shared,
            unique: self.unique | rhs.unique,
        }
    }
}

impl BitAnd for LayoutAccess {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self {
            shared: self.shared & rhs.shared,
            unique: self.unique & rhs.unique,
        }
    }
}

impl BitXor for LayoutAccess {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Self {
            shared: self.shared ^ rhs.shared,
            unique: self.unique ^ rhs.unique,
        }
    }
}