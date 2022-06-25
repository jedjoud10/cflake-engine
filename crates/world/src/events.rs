use crate::World;
use ahash::AHashMap;
use std::{
    any::{Any, TypeId},
    cell::{RefCell, RefMut},
    marker::PhantomData,
    rc::Rc,
};

// Descriptors describe what parameters we will use when calling some specific events
pub trait Descriptor<'a>: Sized + 'static {
    type Params: 'a;
}

// Main event trait (this trait cannot be boxed, so we have to call the execute() method within the boxed impl)
pub trait Event<'a, F>: Descriptor<'a> {
    fn execute(func: &F, params: &mut Self::Params);
}

// Main event trait that will be boxed and stored within the world
pub trait BoxedEvent<Marker> {
    fn execute<'a>(&self, params: &mut <Marker as Descriptor<'a>>::Params)
    where
        Marker: Descriptor<'a>;
}

// These are specialized events that can be executed with any parameter type
struct SpecializedEvents<Marker: 'static>(Vec<(Box<dyn BoxedEvent<Marker>>, i32)>, i32);

impl<Marker: 'static> SpecializedEvents<Marker> {
    // Register a new specialized event with a specific priority index
    fn register_with(&mut self, event: impl BoxedEvent<Marker> + 'static, priority: i32) {
        self.0.push((Box::new(event), priority));
    }

    // Register a new specialized event with an automatic priority index
    fn register(&mut self, event: impl BoxedEvent<Marker> + 'static) {
        self.0.push((Box::new(event), self.1));
        self.1 += 1;
    }

    // Execute all the events that are stored within this set
    fn execute<'a>(&self, mut params: Marker::Params)
    where
        Marker: Descriptor<'a>,
    {
        for (event, _) in self.0.iter() {
            event.execute(&mut params);
        }
    }

    // Sort the specialized events based on their priority index
    fn sort(&mut self) {
        self.0.sort_by(|(_, a), (_, b)| i32::cmp(a, b));
    }
}

// This event map will contain multiple boxed SpecializedEvents<Marker>
type EventMap = AHashMap<TypeId, Box<dyn Any>>;

// Shared map for interior mutability. This also allows us to clone the main events anywhere we want
type SharedMap = Rc<RefCell<EventMap>>;

// These are the global events interface that we will be accessing. This allows us to register, sort, and execute specific events given their descriptor
#[derive(Default)]
pub struct Events(SharedMap);

impl Events {
    // This will get an interior reference to a specialized event set that uses the parameters from a specific descriptor
    fn fetch<'a, Marker: Descriptor<'a>>(&self) -> RefMut<SpecializedEvents<Marker>> {
        RefMut::map(self.0.borrow_mut(), |hashmap| {
            let boxed = hashmap
                .entry(TypeId::of::<Marker>())
                .or_insert_with(|| Box::new(SpecializedEvents::<Marker>(Vec::default(), 0)));
            let specialized = boxed.downcast_mut::<SpecializedEvents<Marker>>().unwrap();
            specialized
        })
    }

    // Register a new event using it's marker descriptor and it's priority index
    pub fn register_with<'a, Marker: Descriptor<'a>>(
        &self,
        event: impl BoxedEvent<Marker> + 'static,
        priority: i32,
    ) {
        self.fetch::<Marker>().register_with(event, priority);
    }

    // Register a new event using it's marker descriptor and an automatic priority index
    pub fn register<'a, Marker: Descriptor<'a>>(&self, event: impl BoxedEvent<Marker> + 'static) {
        self.fetch::<Marker>().register(event);
    }

    // Sort the events based on their priority for a specific marker descriptor type
    pub fn sort<'a, Marker: Descriptor<'a>>(&self) {
        self.fetch::<Marker>().sort();
    }

    // Execute all the events using a specific marker descriptor type
    pub fn execute<'a, Marker: Descriptor<'a>>(&self, params: Marker::Params) {
        self.fetch::<Marker>().execute(params);
    }
}
