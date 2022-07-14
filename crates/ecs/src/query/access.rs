use crate::{registry, Archetype, Component, Entity, Mask};
use std::{
    ops::{BitAnd, BitOr},
    ptr::NonNull,
};

// Layout access that contain the normal mask and writing mask
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct LayoutAccess(Mask, Mask);

impl LayoutAccess {
    // Create a new layout access
    pub const fn new(reading: Mask, writing: Mask) -> Self {
        Self(reading, writing)
    }

    // No layout access at all
    pub const fn none() -> Self {
        Self(Mask::zero(), Mask::zero())
    }

    // Get the normal mask
    pub fn reading(&self) -> Mask {
        self.0
    }

    // Get the writing mask
    pub fn writing(&self) -> Mask {
        self.1
    }

    // Reading AND writing masks combined
    pub fn both(&self) -> Mask {
        self.reading() | self.writing()
    }

    // Given a mask,
}

impl BitOr for LayoutAccess {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0, self.1 | rhs.1)
    }
}

impl BitAnd for LayoutAccess {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0, self.1 & rhs.1)
    }
}
