use crate::{Archetype, Mask, QueryLayoutMut, Scene};
use std::marker::PhantomData;

pub struct QueryMut<'a, L: for<'i> QueryLayoutMut<'i>> {
    scene: &'a mut Scene,
    _phantom3: PhantomData<L>,
    mask: Mask,
}

impl<'a, L: for<'i> QueryLayoutMut<'i>> QueryMut<'a, L> {
    // Create a new mut query from the scene
    pub fn new(scene: &'a mut Scene) -> Self {
        Self {
            scene,
            _phantom3: PhantomData,
            mask: L::reduce(|a, b| a | b).both(),
        }
    }

    // Iterate through the query entries and execute a function for each one of them in another thread
    pub fn for_each(
        mut self,
        threadpool: &mut world::ThreadPool,
        function: impl Fn(<L as world::SliceTuple<'_>>::ItemTuple) + Send + Sync + Copy,
        batch_size: usize,
    ) where
        L: for<'i> world::SliceTuple<'i>,
    {
        threadpool.scope(|scope| {
            for archetype in self.archetypes_mut() {
                let slices = unsafe { L::slices_from_mut_archetype_unchecked(archetype) };
                scope.for_each(slices, function, batch_size);
            }
        });
    }

    // Get the mask that we will use to filter through the archetypes
    pub fn mask(&self) -> Mask {
        self.mask
    }

    // Get an iterator over all the immutable archetypes that we will use
    pub fn archetypes(&self) -> impl Iterator<Item = &Archetype> {
        let layout_mask = self.mask;
        self.scene
            .archetypes()
            .iter()
            .filter_map(move |(&mask, archetype)| mask.contains(layout_mask).then_some(archetype))
    }

    // Get an iterator over all the mutable archetypes that we will use
    pub fn archetypes_mut(&mut self) -> impl Iterator<Item = &mut Archetype> {
        let layout_mask = self.mask;
        self.scene
            .archetypes_mut()
            .iter_mut()
            .filter_map(move |(&mask, archetype)| mask.contains(layout_mask).then_some(archetype))
    }
}
