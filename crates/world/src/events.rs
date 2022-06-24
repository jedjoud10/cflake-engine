use std::{rc::Rc, cell::RefCell, any::{TypeId, Any}, marker::PhantomData};

use ahash::AHashMap;

use crate::World;

struct Init;
struct Update;

pub trait Descriptor<'a>: 'static {
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

trait ArgsTuple {
}
trait Handler<'a> {
    type Inner: 'a;
}
struct EvWrite<T>(PhantomData<*mut T>);
struct EvRead<T>(PhantomData<*const T>);
impl<'a, T: 'static> Handler<'a> for EvWrite<T> {
    type Inner = &'a mut T;
}
impl<'a, T: 'static> Handler<'a> for EvRead<T> {
    type Inner = &'a T;
}
impl<'a, T: Handler<'a>> ArgsTuple for T {

}
impl<'a, A: Handler<'a>, B: Handler<'a>> ArgsTuple for (A, B) {

}
impl<'a, A: Handler<'a>, B: Handler<'a>, C: Handler<'a>> ArgsTuple for (A, B, C) {

}

// These are specialized events that can be executed with any parameter type
struct SpecializedEvents<Params: 'static + ArgsTuple>(Vec<(Box<dyn Event<Params>>, i32)>, i32);

impl<P: 'static + ArgsTuple> SpecializedEvents<P> {
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
        let bruh = SpecializedEvents::<D::Params>(Vec::default(), 0);
        let boxed: Box<dyn Any> = Box::new(bruh);
    }

    // Execute all the events using a specific marker descriptor type
    // This will return the number of events that were successfully executed
    pub fn execute<'a, D: Descriptor<'a>>(&self, params: D::Params) -> Option<usize> {
        let mut hashmap = self.0.borrow_mut();
        let boxed = hashmap.get_mut(&TypeId::of::<D>())?;
        None
    }
}