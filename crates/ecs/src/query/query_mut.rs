use std::marker::PhantomData;
use crate::{Scene, QueryLayoutMut, Mask, Archetype};

pub struct QueryMut<'a, 's, L: for<'i> QueryLayoutMut<'s, 'i>> {
    scene: &'a mut Scene,
    _phantom: PhantomData<&'s ()>,
    _phantom3: PhantomData<L>,
    mask: Mask,
}

impl<'a, 's, L: for<'i> QueryLayoutMut<'s, 'i>> QueryMut<'a, 's, L> {
    // Create a new ref query from the scene
    pub fn new(scene: &Scene) -> Self {
        todo!()
    }

    // Iterate through the query entries and execute a function for each one of them in another thread
    pub fn for_each(mut self, threadpool: &mut world::ThreadPool, function: impl Fn(L) + Send + Sync, batch_size: usize) {
        for archetype in self.archetypes_mut() {
            let mut slices = unsafe { L::slices_from_mut_archetype_unchecked(archetype) };
            //let tuple = unsafe { L::get_mut_unchecked(&mut slices, 0) };

            /*
            for i in 0..archetype.len() {
                //let borrow = &mut slices;
                let tuple = unsafe { L::get_mut_unchecked(borrow, 0) };
                //function(tuple);
            }
            */
        }
    }

    // Get the mask that we will use to filter through the archetypes
    pub fn mask(&self) -> Mask {
        self.mask
    }
    
    // Get an iterator over all the immutable archetypes that we will use
    pub fn archetypes(&self) -> impl Iterator<Item = &Archetype> {
        let layout_mask = self.mask;
        self.scene.archetypes().iter().filter_map(move |(&mask, archetype)| {
            mask.contains(layout_mask).then_some(archetype)
        })
    }

    // Get an iterator over all the mutable archetypes that we will use
    pub fn archetypes_mut(&mut self) -> impl Iterator<Item = &mut Archetype> {
        let layout_mask = self.mask;
        self.scene.archetypes_mut().iter_mut().filter_map(move |(&mask, archetype)| {
            mask.contains(layout_mask).then_some(archetype)
        })
    }
}