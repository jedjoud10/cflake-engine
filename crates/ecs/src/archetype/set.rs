use std::collections::{HashMap, hash_map::Entry};

use crate::{MaskHasher, Archetype, Mask, UniqueComponentStoragesHashMap};

// The archetype set (HashMap and Vec)
#[derive(Default)]
pub struct ArchetypeSet {
    // Masks -> Archetype Index
    indices: HashMap<Mask, usize, MaskHasher>,

    // Archetypes
    archetypes: Vec<Archetype>
}

impl ArchetypeSet {
    // Insert a new archetype if it does not exist yet, and return a mutable reference to it
    pub(crate) fn insert_default(&mut self, mask: Mask, uniques: &UniqueComponentStoragesHashMap) -> &mut Archetype {
        // Check if an archetype with this mask already exists        
        if let Entry::Occupied(index) = self.indices.entry(mask) {
            self.archetypes.get_mut(*index.get()).unwrap()
        } else {
            // Insert a new archetype with the proper mask
            let index = self.archetypes.len();
            self.archetypes.push(Archetype::new(mask, uniques));
            self.indices.insert(mask, index);
            self.archetypes.get_mut(index).unwrap()
        }
    }
    // Get two archetypes at the same time
    pub fn get_two_mut(&mut self, m1: Mask, m2: Mask) -> Option<(&mut Archetype, &mut Archetype)> {
        // The archetypes are not disjoint
        if m1 == m2 { return None; }

        // Get the indices
        let mut i1 = *self.indices.get(&m1)?;
        let mut i2 = *self.indices.get(&m2)?;

        // Swap if needed
        if i2 < i1 { std::mem::swap(&mut i1, &mut i2) }

        // Now get the two archetypes
        let (first, second) = self.archetypes.split_at_mut(i2);
        Some((first.get_mut(i1).unwrap(), second.get_mut(0).unwrap()))
    }
    // Get an archetype using it's mask immutably and mutably
    pub fn get(&self, mask: &Mask) -> Option<&Archetype> {
        self.archetypes.get(*self.indices.get(mask)?)
    }
    pub fn get_mut(&mut self, mask: &Mask) -> Option<&mut Archetype> {
        self.archetypes.get_mut(*self.indices.get(mask)?)
    }
    // Iterate through each archetype
    pub fn iter(&self) -> impl Iterator<Item = &Archetype> {
        self.archetypes.iter()
    }
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Archetype> {
        self.archetypes.iter_mut()
    }
}