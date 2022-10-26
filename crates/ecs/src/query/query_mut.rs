use smallvec::SmallVec;

use crate::{Archetype, Mask, QueryLayoutMut, Scene, StateRow, QueryFilter};
use std::marker::PhantomData;

// This is a query that will be fetched from the main scene that we can use to get components out of entries with a specific layout
// Even though I define the 'it, 'b, and 's lfietimes, I don't use them in this query, I only use them in the query iterator
pub struct QueryMut<'a: 'b, 'b, 'i, L: for<'it> QueryLayoutMut<'it>> {
    archetypes: Vec<&'a mut Archetype>,
    mask: Mask,
    enabled: Option<SmallVec<[u128; 2]>>,

    // Implement hop map if needed


    _phantom1: PhantomData<&'b ()>,
    _phantom2: PhantomData<&'i ()>,
    _phantom3: PhantomData<L>,
}

impl<'a: 'b, 'b, 'i, L: for<'it> QueryLayoutMut<'it>> QueryMut<'a, 'b, 'i, L> {
    // Get the archetypes and layout mask. Used internally only
    fn archetypes_mut(scene: &mut Scene) -> (Mask, Vec<&mut Archetype>) {
        let mask = L::reduce(|a, b| a | b).both();
        let archetypes = scene
            .archetypes_mut()
            .iter_mut()
            .filter_map(move |(&archetype_mask, archetype)| archetype_mask.contains(mask).then_some(archetype))
            .collect::<Vec<_>>();
        (mask, archetypes)
    }

    // Create a new mut query from the scene
    pub fn new(scene: &'a mut Scene) -> Self {
        let (mask, archetypes) = Self::archetypes_mut(scene);

        Self {
            archetypes,
            enabled: None,
            _phantom3: PhantomData,
            mask,
            _phantom1: PhantomData,
            _phantom2: PhantomData,
        }
    }

    // Create a new mut query from the scene, but make it have a specific entry enable/disable masks
    pub fn new_with_filter<F: QueryFilter>(scene: &'a mut Scene, filter: F) -> Self {
        let (mask, archetypes) = Self::archetypes_mut(scene);

        let cached = F::prepare();
        let mask = mask;
        archetypes.iter().map(|archetype| {
            let states = archetype.states();
            let states = states.borrow();
            let iter = states.iter().cloned().map(|state| F::eval(&cached, state, mask));
            //iter
            todo!()
        }).flatten();

        Self {
            archetypes,
            mask,
            enabled: None,
            _phantom3: PhantomData,
            _phantom1: PhantomData,
            _phantom2: PhantomData,
        }
    }

    // Iterate through the query entries and execute a function for each one of them in another thread
    pub fn for_each(
        mut self,
        threadpool: &mut world::ThreadPool,
        function: impl Fn(<L as world::SliceTuple<'_>>::ItemTuple) + Send + Sync + Clone,
        batch_size: usize,
    ) where
        L: for<'it> world::SliceTuple<'it>,
    {
        threadpool.scope(|scope| {
            for archetype in self.archetypes.iter_mut() {
                // Send the archetype slices to multiple threads to be able to compute them
                let slices = unsafe { L::slices_from_mut_archetype_unchecked(archetype) };
                scope.for_each(slices, function.clone(), batch_size);

                // We don't have to worry about doing this since the entry disabled/enabled mask is already computed when the query was created
                let mask = self.mask;
                let states = archetype.states();
                let mut borrowed = states.borrow_mut();
                
                // Update the mutable state masks
                for state in borrowed.iter_mut() {
                    StateRow::update(state, |_, _, mutated| *mutated = *mutated | mask);
                }
            }
        });
    }

    // Get the mask that we will use to filter through the archetypes
    pub fn mask(&self) -> Mask {
        self.mask
    }

    // Get the number of entries that we will have to iterate through
    pub fn len(&self) -> usize {
        self.archetypes.iter().map(|a| a.len()).sum()
    }
}

impl<'a: 'b, 'b, 'it, L: for<'i> QueryLayoutMut<'i>> IntoIterator for QueryMut<'a, 'b, 'it, L> {
    type Item = <L as QueryLayoutMut<'it>>::ItemTuple;
    type IntoIter = QueryMutIter<'b, 'it, L>;

    fn into_iter(mut self) -> Self::IntoIter {
        QueryMutIter {
            archetypes: self.archetypes,
            current: None,
            slice: None,
            index: 0,
            length: None,
            mask: self.mask,
            _phantom1: PhantomData,
            _phantom2: PhantomData,
        }
    }
}

// This is a mutable query iterator that will iterate through all the query entries in arbitrary order
pub struct QueryMutIter<'b, 'i, L: QueryLayoutMut<'i>> {
    archetypes: Vec<&'b mut Archetype>,
    current: Option<&'b mut Archetype>,
    length: Option<usize>,
    slice: Option<L>,
    index: usize,
    mask: Mask,
    _phantom1: PhantomData<&'i ()>,
    _phantom2: PhantomData<L>,
}

impl<'b, 's: 'i, 'i, L: QueryLayoutMut<'i>> Iterator for QueryMutIter<'b, 'i, L> {
    type Item = L::ItemTuple;

    fn next(&mut self) -> Option<Self::Item> {
        // Hop onto the next archetype if we are done iterating through the current one
        if (self.index + 1) > self.length.unwrap_or_default() {
            let next = self.archetypes.pop()?;
            let layout = unsafe { L::slices_from_mut_archetype_unchecked(next) };
            self.length = Some(next.len());
            self.current = Some(next);
            self.slice = Some(layout);
            self.index = 0;
        }

        // I have to do this since iterators cannot return data that they are referencing, but in this case, it is safe to do so
        self.current.as_mut()?;
        let slice = self.slice.as_mut().unwrap();
        let slice = unsafe { &mut *(slice as *mut _) };
        let items = unsafe { L::get_mut_unchecked(slice, 0) };
        self.index += 1;

        // Update the mask for the current entity
        let states = self.current.as_mut().unwrap().states();
        let mut vec = states.borrow_mut();
        vec[self.index].update(|_, _, update| *update = *update | self.mask);

        Some(items)
    }
}