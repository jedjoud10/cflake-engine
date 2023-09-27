use crate::{mask::{Mask, MaskHashMap}, archetype::{Archetype, StateFlags}, vec::UntypedVec, registry::{Component, mask}};

/// An owned layout trait will be implemented for owned tuples that contain a set of components.
/// This will also handle the synchronization between the states/component columns whenever we add bundles.
pub trait Bundle: 'static {
    /// Get a combined  mask by running a lambda on each mask.
    fn reduce(lambda: impl FnMut(Mask, Mask) -> Mask) -> Mask
    where
        Self: Sized;

    /// Checks if this bundle is valid.
    fn is_valid() -> bool
    where
        Self: Sized,
    {
        let mut count = 1;
        let mask = Self::reduce(|a, b| {
            count += 1;
            a | b
        });
        mask.count_ones() == count as u32
    }

    /// Push multiple elements into an archetype, returns how many we added.
    /// Returns None if the bundle mask does not match with the archetype mask.
    /// If moved is true, don't set the newly added components as "added" or "modified".
    fn extend_from_iter<'a>(
        archetype: &'a mut Archetype,
        moved: bool,
        iter: impl IntoIterator<Item = Self>,
    ) -> Option<usize>
    where
        Self: Sized;

    /// Get the default untyped component vectors for this bundle.
    fn default_vectors() -> MaskHashMap<Box<dyn UntypedVec>>
    where
        Self: Sized;
}

/// Bundles can be boxed and cloned (this is why this is not defined as Sized) so we can use them as a base for our prefabs.
pub trait PrefabBundle: 'static + Bundle {
    /// Clone the prefab bundle and add it to the archetype.
    fn prefabify<'a>(&self, archetype: &'a mut Archetype) -> Option<()>;
}

impl<B: Clone + Bundle> PrefabBundle for B {
    fn prefabify<'a>(&self, archetype: &'a mut Archetype) -> Option<()> {
        B::extend_from_iter(archetype, false, [self.clone()]).map(|_| ())
    }
}

// Implement the bundle for single component
impl<T: Component> Bundle for T {
    fn reduce(lambda: impl FnMut(Mask, Mask) -> Mask) -> Mask {
        std::iter::once(mask::<T>())
            .into_iter()
            .reduce(lambda)
            .unwrap()
    }

    fn is_valid() -> bool {
        true
    }

    fn extend_from_iter<'a>(
        archetype: &'a mut Archetype,
        moved: bool,
        iter: impl IntoIterator<Item = Self>,
    ) -> Option<usize> {
        let (components, delta_frame_states, delta_tick_states) = archetype.column_mut::<T>()?;

        let mut additional = 0;
        for bundle in iter {
            components.push(bundle);
            additional += 1;
        }

        delta_frame_states.extend_with_flags(
            additional,
            StateFlags {
                added: !moved,
                modified: !moved,
            },
        );

        delta_tick_states.extend_with_flags(
            additional,
            StateFlags {
                added: !moved,
                modified: !moved,
            },
        );

        Some(additional)
    }

    fn default_vectors() -> MaskHashMap<Box<dyn UntypedVec>> {
        let boxed: Box<dyn UntypedVec> = Box::new(Vec::<T>::new());
        let mask = mask::<T>();
        MaskHashMap::from_iter(std::iter::once((mask, boxed)))
    }
}
