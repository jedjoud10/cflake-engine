use crate::{Mask, QueryLayoutMut, QueryLayoutRef};
use std::ops::{BitAnd, BitOr, BitXor};

// Layout access that contain the shared access mask and unique access mask
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LayoutAccess {
    // Used for searching for valid archetypes
    pub(super) arch_search: Mask,

    // Used for query validation
    pub(super) validation_shared: Mask,
    pub(super) validation_unique: Mask,
}

impl LayoutAccess {
    // Get the archetype search mask
    pub fn search(&self) -> Mask {
        self.arch_search
    }

    // Get the shared validation mask
    pub fn shared(&self) -> Mask {
        self.validation_shared
    }

    // Get the unique validation mask
    pub fn unique(&self) -> Mask {
        self.validation_unique
    }

    // Get both validation masks (bitwise or)
    pub fn both_validation_masks(&self) -> Mask {
        self.validation_shared | self.validation_unique
    }

    // Create a layout access mask from a layout ref
    pub fn from_layout_ref<L: QueryLayoutRef>() -> Self {
        L::reduce(|a, b| a | b)
    }

    // Create a layout access mask from a query layout mut
    pub fn from_layout_mut<L: QueryLayoutMut>() -> Self {
        L::reduce(|a, b| a | b)
    }
}

impl BitOr for LayoutAccess {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self {
            arch_search: self.arch_search | rhs.arch_search,
            validation_shared: self.validation_shared | rhs.validation_shared,
            validation_unique: self.validation_unique | rhs.validation_unique,
        }
    }
}

impl BitAnd for LayoutAccess {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self {
            arch_search: self.arch_search & rhs.arch_search,
            validation_shared: self.validation_shared & rhs.validation_shared,
            validation_unique: self.validation_unique & rhs.validation_unique,
        }
    }
}

impl BitXor for LayoutAccess {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Self {
            arch_search: self.arch_search ^ rhs.arch_search,
            validation_shared: self.validation_shared ^ rhs.validation_shared,
            validation_unique: self.validation_unique ^ rhs.validation_unique,
        }
    }
}
