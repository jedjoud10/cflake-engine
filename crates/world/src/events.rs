use std::{any::{TypeId, Any}, rc::Rc, cell::{RefCell, RefMut}};
use ahash::AHashMap;
use glutin::event::WindowEvent;
use crate::World;

pub trait Event<'a, M: MarkerDescriptor<'a>>: 'static {
    fn call(&self, data: M::Inner);
}

pub trait MarkerDescriptor<'a>: 'static {
    type Inner: 'a;
}

struct Init;
struct Update;


impl<'a> MarkerDescriptor<'a> for Init {
    type Inner = &'a mut World;
}

impl<'a> MarkerDescriptor<'a> for Update {
    type Inner = &'a mut World;
}


impl<'a, F> Event<'a, Init> for F where F: Fn(&mut World) + 'static {
    fn call(&self, data: <Init as MarkerDescriptor>::Inner) {
        self(data);
    }
}

impl<'a, F> Event<'a, Update> for F where F: Fn(&mut World) + 'static {
    fn call(&self, data: <Update as MarkerDescriptor>::Inner) {
        self(data);
    }
}

// This trait will be implemented for vectors that contain multiple closures of a specific type
// This should allow for less heap allocation in total, since most closures will just be function pointers (fn())
trait EventStorage {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn sort(&mut self);
}

// Implementations of event storage that's kinda wrong but it works
impl<T: Sized + 'static> EventStorage for Vec<(T, i32)> {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn sort(&mut self) {
        self.sort_by(|(_, a), (_, b)| i32::cmp(a, b));
    }
}

// Normal event map and the shared map (interior mutability)
// These event maps contain the events in this hierarchy
// TypeId -> Marker Storager -> Vector Storage -> Closures 
type EventMap = AHashMap<TypeId, (Box<dyn EventStorage>, i32)>;
type SharedEventMap = Rc<RefCell<EventMap>>;

// This is the main event handler that will be cloned (shared)
// We can clone this safely without paining ourselves since the inner data can be simply just cloned and extracted when needed
// This stores each closure as it's own boxed type, so if we have a bunch of closures that implement the same 
#[derive(Default)]
pub struct Events(SharedEventMap);

impl Events {
    // Register a new event with an automatic priority index
    pub fn register<'a, M: MarkerDescriptor<'a>>(&self, event: impl Event<'a, M>) {
    }

    // Register a new event with a given priority index
    pub fn register_with<'a, M: MarkerDescriptor<'a>>(&self, event: impl Event<'a, M>, priority: i32) {
    }

    // Sort the events that are created using a specific marker type
    pub fn sort<'a, M: MarkerDescriptor<'a>>(&self) {
    }

    // Execute all the boxed events of a specific type 
    pub fn execute<'a, M: MarkerDescriptor<'a>>(&self, data: M::Inner) {

    } 
}