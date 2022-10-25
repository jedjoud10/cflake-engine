use crate::{Archetype, Mask, QueryLayoutMut, Scene, StateRow};
use std::marker::PhantomData;

// This is a query that will be fetched from the main scene that we can use to get components out of entries with a specific layout
// Even though I define the 'it, 'b, and 's lfietimes, I don't use them in this query, I only use them in the query iterator
pub struct QueryMut<'a, 'b, 's, 'i, L: for<'it> QueryLayoutMut<'it>> {
    scene: &'a mut Scene,
    mask: Mask,
    _phantom1: PhantomData<&'b ()>,
    _phantom4: PhantomData<&'s ()>,
    _phantom2: PhantomData<&'i ()>,
    _phantom3: PhantomData<L>,
}

impl<'a, 'b, 's, 'i, L: for<'it> QueryLayoutMut<'it>> QueryMut<'a, 'b, 's, 'i, L> {
    // Create a new mut query from the scene
    pub fn new(scene: &'a mut Scene) -> Self {
        Self {
            scene,
            _phantom3: PhantomData,
            mask: L::reduce(|a, b| a | b).both(),
            _phantom1: PhantomData,
            _phantom2: PhantomData,
            _phantom4: PhantomData,
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
            for archetype in self.archetypes_mut() {
                let slices = unsafe { L::slices_from_mut_archetype_unchecked(archetype) };
                scope.for_each(slices, function.clone(), batch_size);
            }
        });
        
        // TODO: Handle state change
        /*
        
        let mask = self.mask;
        let states = archetype.states();
        let mut borrowed = states.borrow_mut();
        
        for state in borrowed.iter_mut() {
            StateRow::update(state, |_, _, mutated| *mutated = *mutated | mask);
        }
        */
    }

    // Get the mask that we will use to filter through the archetypes
    pub fn mask(&self) -> Mask {
        self.mask
    }

    // Get the number of entries that we will have to iterate through
    pub fn len(&self) -> usize {
        self.archetypes().map(|a| a.len()).sum()
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

impl<'a: 'b, 'b, 's: 'it, 'it, L: for<'i> QueryLayoutMut<'i> + 's> IntoIterator for QueryMut<'a, 'b, 's, 'it, L> {
    type Item = <L as QueryLayoutMut<'it>>::ItemTuple;
    type IntoIter = QueryMutIter<'b, 's, 'it, L>;

    fn into_iter(mut self) -> Self::IntoIter {
        let layout_mask = self.mask;
        let archetypes = self.scene
            .archetypes_mut()
            .iter_mut()
            .filter_map(move |(&mask, archetype)| mask.contains(layout_mask).then_some(archetype)).collect::<Vec<_>>();
        QueryMutIter {
            archetypes,
            current: None,
            slice: None,
            index: 0,
            mask: self.mask,
            _phantom3: PhantomData,
            _phantom4: PhantomData,
            _phantom5: PhantomData,
        }
    }
}

// This is a mutable query iterator that will iterate through all the query entries in arbitrary order
pub struct QueryMutIter<'b, 's: 'i, 'i, L: QueryLayoutMut<'i> + 's> {
    archetypes: Vec<&'b mut Archetype>,
    current: Option<&'b mut Archetype>,
    slice: Option<&'s mut L>,
    index: usize,
    mask: Mask,
    _phantom5: PhantomData<&'s ()>,
    _phantom3: PhantomData<&'i ()>,
    _phantom4: PhantomData<L>,
}

impl<'b, 's: 'i, 'i, L: QueryLayoutMut<'i>> Iterator for QueryMutIter<'b, 's, 'i, L> {
    type Item = L::ItemTuple;

    fn next(&mut self) -> Option<Self::Item> {
        let arch = self.archetypes.pop().unwrap();
        self.current = Some(arch);
        //self.slice = Some(unsafe { L::slices_from_mut_archetype_unchecked(self.current.as_mut().unwrap()) } );
        let slice = self.slice.as_mut().unwrap();

        // Fuck you.
        let slice = unsafe { &mut *(slice as *mut &mut L) };
        let items = unsafe { L::get_mut_unchecked(slice, 0) };
        Some(items)
    }
}