use std::{any::{TypeId, Any}, rc::Rc, cell::{RefCell, RefMut}};
use ahash::AHashMap;
use glutin::event::WindowEvent;
use crate::World;

pub trait Callable<'a, Marker: MarkerDescriptor<'a>> {
    fn call(&self, data: Marker::Inner);
}


pub trait MarkerDescriptor<'a> {
    type Inner: 'a;
}

struct Init;
struct Update;


impl<'a> MarkerDescriptor<'a> for Init {
    type Inner = ();
}

impl<'a> MarkerDescriptor<'a> for Update {
    type Inner = ();
}


impl<'a, F> Callable<'a, Init> for F where F: FnOnce(&mut World) {
    fn call(&self, data: <Init as MarkerDescriptor>::Inner) {
        todo!()
    }
}

impl<'a, F> Callable<'a, Update> for F where F: Fn(&mut World) {
    fn call(&self, data: <Init as MarkerDescriptor>::Inner) {
        todo!()
    }
}

// Type aliases
pub(crate) type EventMap = AHashMap<TypeId, (Vec<(Box<dyn Any>, i32)>, i32)>;
pub(crate) type SharedEventMap = Rc<RefCell<EventMap>>;

// This is the main event handler that will be cloned (shared)
// We can clone this safely without paining ourselves since the inner data can be simply just cloned and extracted when needed
#[derive(Default)]
pub struct Events(pub(crate) SharedEventMap);

impl Events {
    // Register a new event with an automatic priority index
    pub fn register<'a, M: MarkerDescriptor<'a>>(&self, event: impl Callable<'a, M>) {
        let mut hashmap = self.0.borrow_mut();

        // Get the vector and fetch the old next index
        let (vec, next) = hashmap.entry(TypeId::of::<M>()).or_default();
        
        // Box the function event and insert it into the vector
        let boxed: Box<dyn Any> = Box::new(event);
        vec.push((boxed, *next));
        
        // Increment the automatic priority index
        *next += 1;
    }

    // Register a new event with a given priority index
    pub fn register_with<'a, M: MarkerDescriptor<'a>>(&self, event: impl Callable<'a, M>, priority: i32) {
        let mut hashmap = self.0.borrow_mut();

        // Get the vector or insert it if missing
        let (vec, _) = hashmap.entry(TypeId::of::<M>()).or_default();

        // Box the functio event and insert it into the vector
        let boxed: Box<dyn Any> = Box::new(event);
        vec.push((boxed, priority));
    }

    // Sort the events that are created using a specific marker type
    pub fn sort<'a, M: MarkerDescriptor<'a>>(&self) {
        let mut hashmap = self.0.borrow_mut();

        // Get the vector or insert it if missing
        let (vec, next) = hashmap.entry(TypeId::of::<M>()).or_default();

        // Sort the vector using the priorities
        vec.sort_by(|(_, a), (_, b)| i32::cmp(a, b));
    }

    // Execute all the events of a specific marker type, using it's given data
    pub fn execute<'a, M: MarkerDescriptor<'a>>(&self, data: &mut M::Inner) {
        let mut hashmap = self.0.borrow_mut();

        // Get the vector or insert it if missing (kinda useless in this case but eh)
        let (vec, _) = hashmap.entry(TypeId::of::<M>()).or_default();

        // Iterate through the vector and execute the events
        for (event, _) in vec.iter_mut() {
            event.
        }
    }
}