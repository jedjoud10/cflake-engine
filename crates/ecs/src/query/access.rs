use crate::{registry, Archetype, Component, Entity, Mask};
use std::{
    ops::{BitAnd, BitOr},
    ptr::NonNull,
};

// Layout access that contain the normal mask and writing mask
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct LayoutAccess(pub(super) Mask, pub(super) Mask);

impl LayoutAccess {
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
