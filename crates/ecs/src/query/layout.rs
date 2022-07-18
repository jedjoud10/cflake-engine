// Mutable query layouts that might contain mutable references
// This must take a mutable reference to the current archetype 
pub unsafe trait MutQueryLayout<'a>: 'a + Sized {
    type Cache: 'a;
    fn prepare(archetype: &mut Archetype) -> Self::Cache;
    fn read(cache: &mut Self::Cache, i: usize) -> Self;
}

// Immutable query layouts that will never contain any mutable referneces
// This simply takes an immutable reference to the archetype
pub unsafe trait RefQueryLayout<'a>: 'a + Sized {
    type Cache: 'a;
    fn prepare(archetype: &Archetype) -> Self::Cache;
    fn read(cache: &Self::Cache, i: usize) -> Self;
}

// An archetype item is something that is stored independently within each archetype
// This could be entity ID's or event component table slices
pub trait ArchetypeItem {
    fn get(archetype: &Archetype) -> Option<&Self>;
    fn get_mut(archetype: &mut Archetype) -> Option<&mut Self>;
}