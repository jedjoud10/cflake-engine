use std::{rc::Rc, cell::{RefCell, RefMut}, any::{TypeId, Any}, marker::PhantomData};

use ahash::AHashMap;

use crate::World;

struct Init;
struct Update;

pub trait Descriptor: 'static {
    type Params: 'static + ArgsTuple;
}

impl Descriptor for Init {
    type Params = EvWrite<World>;
}

impl Descriptor for Update {
    type Params = EvWrite<World>;
}

pub trait Event<Params> {
    fn call<'a>(&self, params: &mut <Params as ArgsTupleConverter<'a>>::Inner) where Params: ArgsTupleConverter<'a>;
}

impl<F> Event<EvWrite<World>> for F where F: Fn(&mut World) + 'static {
    fn call<'a>(&self, params: &mut <EvWrite<World> as ArgsTupleConverter<'a>>::Inner) where EvWrite<World>: ArgsTupleConverter<'a> {
        todo!()
    }
}

pub trait ArgsTuple {
}
pub trait ArgsTupleConverter<'a> {
    type Inner: 'a;
}
trait Handler {
}
pub trait HandlerConverter<'a> {
    type Inner: 'a;
}
struct EvWrite<T>(PhantomData<*mut T>);
struct EvRead<T>(PhantomData<*const T>);
impl<T: 'static> Handler for EvWrite<T> {}
impl<T: 'static> Handler for EvRead<T> {}
impl<T: Handler> ArgsTuple for T {}
impl<A: Handler, B: Handler> ArgsTuple for (A, B) {}
impl<'a, T: HandlerConverter<'a>> ArgsTupleConverter<'a> for T {
    type Inner = T::Inner;
}
impl<'a, A: HandlerConverter<'a>, B: HandlerConverter<'a>> ArgsTupleConverter<'a> for (A, B) {
    type Inner = (A::Inner, B::Inner);
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
    fn call<'a>(&mut self, mut params: P::Inner) where P: ArgsTupleConverter<'a> {
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
    // This will get an interior reference to a specialized event set that uses the parameters from a specific descriptor
    fn fetch<D: Descriptor>(&self) -> RefMut<SpecializedEvents<D::Params>> {
        RefMut::map(self.0.borrow_mut(), |hashmap| {
            let boxed = hashmap.entry(TypeId::of::<D>()).or_insert_with(|| Box::new(SpecializedEvents::<D::Params>(Vec::default(), 0)));
            let specialized = boxed.downcast_mut::<SpecializedEvents<D::Params>>().unwrap();
            specialized            
        })
    }

    // Register a new event using it's marker descriptor and it's priority index
    pub fn register_with<D: Descriptor>(&self, event: impl Event<D::Params> + 'static, priority: i32) {
        self.fetch::<D>().register_with(event, priority);
    }

    // Register a new event using it's marker descriptor and an automatic priority index
    pub fn register<D: Descriptor>(&self, event: impl Event<D::Params> + 'static) {
        self.fetch::<D>().register(event);
    }

    // Execute all the events using a specific marker descriptor type
    // This will return the number of events that were successfully executed
    pub fn execute<'a, D: Descriptor>(&self, params: <D::Params as ArgsTupleConverter<'a>>::Inner) -> Option<usize> where D::Params: ArgsTupleConverter<'a> {
        self.fetch::<D>().call(params);
        None
    }
}