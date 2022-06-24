use std::{rc::Rc, cell::RefCell, any::{TypeId, Any}};

use ahash::AHashMap;

use crate::World;

struct Init;
struct Update;

pub trait Descriptor<'a> {
    type Params: 'a;
}

impl<'a> Descriptor<'a> for Init {
    type Params = &'a mut World;
}

impl<'a> Descriptor<'a> for Update {
    type Params = &'a mut World;
}

pub trait Event<Params> {
    fn call(&self, params: &mut Params);
}

impl<F> Event<&mut World> for F where F: Fn(&mut World) + 'static {
    fn call(&self, params: &mut &mut World) {
        self(params)
    }
}

// These are specialized events that can be executed with any parameter type
struct SpecializedEvents<Params>(Vec<(Box<dyn Event<Params>>, i32)>, i32);

impl<P> SpecializedEvents<P> {
    // Register a new specialized event with a specific priority index
    fn register_with(&mut self, event: impl Event<P> + 'static, priority: i32) {
        self.0.push((Box::new(event), priority));
    }

    // Register a new specialized event with an automatic priority index
    fn register(&mut self, event: impl Event<P> + 'static) {
        self.0.push((Box::new(event), self.1));
        self.1 += 1;
    }

    // Call each boxed event with the appropriate given parameters
    fn call(&mut self, mut params: P) {
        for (event, _) in self.0.iter_mut() {
            event.call(&mut params);
        }
    }

    // Sort the specialized events based on their priority index
    fn sort(&mut self) {
        self.0.sort_by(|(_, a), (_, b)| i32::cmp(a, b));
    }
}

// This event map will contain multiple boxed SpecializedEvents<P>
type EventMap = AHashMap<TypeId, Box<dyn Any>>;

// Shared map for interior mutability. This also allows us to clone the main events anywhere we want
type SharedMap = Rc<RefCell<EventMap>>;

// These are the global events interface that we will be accessing. This allows us to register, sort, and execute specific events given their descriptor
#[derive(Default)]
pub struct Events(SharedMap);

impl Events {
    // Register a new event using it's marker descriptor and it's priority index
    pub fn register_with<'a, D: Descriptor<'a>>(&self, event: impl Event<D::Params> + 'static, priority: i32) {

    }

    // Register a new event using it's marker descriptor and an automatic priority index
    pub fn register<'a, D: Descriptor<'a>>(&self, event: impl Event<D::Params> + 'static) {
        
    }
}