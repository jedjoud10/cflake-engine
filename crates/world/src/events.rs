use glutin::{event::{WindowEvent, DeviceEvent}, window::Window};
use crate::World;

// These event markers are only used to make things a bit prettier when inserting / sorting events
// This trait will be implemented for Init, Update, glutin::WindowEvent, glutin::DeviceEvent 
pub trait Marker {
    // Dyn Fn or FnOnce
    type F: ?Sized;

    // Insert the boxed slot into the event manager
    fn insert(slot: Slot<Self::F>, events: &mut Events);
}

// Descriptors are also implemented for markers, but they have a different purpose
// Descriptors simply specify what parameters will be needed to execute the inner event, and how to execute em
pub trait Descriptor<'a>: Marker {
    type Params: 'a;

    // Execute all the boxed events/functions using the proper given parameters
    fn call_all(events: &mut Events, params: Self::Params); 
}

// Events also contain a priority index that we will use to sort them
// Each slot contains a single event with it's priority index
// F will be the trait object, like dyn FnOnce or dyn Fn
pub struct Slot<F: ?Sized> {
    boxed: Box<F>,
    priority: i32,
}

// Convert any function closure or function event into a proper slot
// This will be implemented for the function closures and it will box them into their corresponding slot type
pub trait IntoSlot<F: ?Sized> {
    fn into_slot(self, priority: i32) -> Slot<F>;
}

// Events are anything that can be called, like function pointers, or closures
// This main struct will help us define, read, and fetch events to be able to execute them
// Events are only defined internally, and the user cannot create implementations of their own events
pub struct Events {
    pub(crate) init: Vec<Slot<dyn FnOnce(&mut World)>>,
    pub(crate) update: Vec<Slot<dyn Fn(&mut World)>>,
    pub(crate) window: Vec<Slot<dyn Fn(&mut World, &WindowEvent)>>,
    pub(crate) device: Vec<Slot<dyn Fn(&mut World, &DeviceEvent)>>,
}

impl Events {
    // Register a new event with an automatic priority index
    pub fn register<'a, M: Marker + Descriptor<'a>>(&mut self, event: impl IntoSlot<M::F>) {
        M::insert(event.into_slot(0), self)
    }

    // Register a new event with a specific priority index
    pub fn register_with<'a, M: Marker + Descriptor<'a>>(&mut self, event: impl IntoSlot<M::F>, priority: i32) {
        M::insert(event.into_slot(priority), self)
    }

    // This will sort all the events of all types
    pub fn sort(&mut self) {
        fn sort<T: ?Sized>(vec: &mut Vec<Slot<T>>) {
            vec.sort_by(|a: &Slot<T>, b: &Slot<T>| i32::cmp(&a.priority, &b.priority))
        }

        // Sort all of the vectors
        sort(&mut self.init);
        sort(&mut self.update);
        sort(&mut self.window);
        sort(&mut self.device)
    }
    
    // This will execute all the events of a specific type
    pub fn execute<'a, M: Marker + Descriptor<'a>>(&mut self, params: M::Params) {
        M::call_all(self, params);
    }
}


// Init marker event
pub struct Init(());

impl<F> IntoSlot<dyn FnOnce(&mut World)> for F where F: FnOnce(&mut World) + 'static {
    fn into_slot(self, priority: i32) -> Slot<dyn FnOnce(&mut World)> {
        Slot { boxed: Box::new(self), priority }
    }
}

impl Marker for Init {
    type F = dyn FnOnce(&mut World);

    fn insert(slot: Slot<Self::F>, events: &mut Events) {
        events.init.push(slot);
    }
}

impl<'a> Descriptor<'a> for Init {
    type Params = &'a mut World;

    fn call_all(events: &mut Events, params: Self::Params) {
        let vec = std::mem::take(&mut events.init);

        for Slot { boxed, .. } in vec {
            boxed(params);
        }
    }
}

// Update marker event
pub struct Update(());

impl<F> IntoSlot<dyn Fn(&mut World)> for F where F: Fn(&mut World) + 'static {
    fn into_slot(self, priority: i32) -> Slot<dyn Fn(&mut World)> {
        Slot { boxed: Box::new(self), priority }
    }
}

impl Marker for Update {
    type F = dyn Fn(&mut World);

    fn insert(slot: Slot<Self::F>, events: &mut Events) {
        events.update.push(slot);
    }
}

impl<'a> Descriptor<'a> for Update {
    type Params = &'a mut World;

    fn call_all(events: &mut Events, params: Self::Params) {
        for Slot { boxed, .. } in events.update.iter() {
            boxed(params)
        }
    }
}

// Window marker event
impl<F> IntoSlot<dyn Fn(&mut World, &WindowEvent)> for F where F: Fn(&mut World, &WindowEvent) + 'static {
    fn into_slot(self, priority: i32) -> Slot<dyn Fn(&mut World, &WindowEvent)> {
        Slot { boxed: Box::new(self), priority }
    }
}

impl<'a> Marker for WindowEvent<'a> {
    type F = dyn Fn(&mut World, &WindowEvent);

    fn insert(slot: Slot<Self::F>, events: &mut Events) {
        events.window.push(slot);
    }
}

impl<'a, 'b> Descriptor<'a> for WindowEvent<'b> where 'b: 'a {
    type Params = (&'a mut World, &'a WindowEvent<'b>);

    fn call_all(events: &mut Events, params: Self::Params) {
        let world = params.0;
        let window = params.1;

        for Slot { boxed, .. } in events.window.iter() {
            boxed(world, window);
        }
    }
}

// Device marker event
impl<F> IntoSlot<dyn Fn(&mut World, &DeviceEvent)> for F where F: Fn(&mut World, &DeviceEvent) + 'static {
    fn into_slot(self, priority: i32) -> Slot<dyn Fn(&mut World, &DeviceEvent)> {
        Slot { boxed: Box::new(self), priority }
    }
}

impl Marker for DeviceEvent {
    type F = dyn Fn(&mut World, &DeviceEvent);

    fn insert(slot: Slot<Self::F>, events: &mut Events) {
        events.device.push(slot);
    }
}

impl<'a> Descriptor<'a> for DeviceEvent {
    type Params = (&'a mut World, &'a DeviceEvent);

    fn call_all(events: &mut Events, params: Self::Params) {
        let world = params.0;
        let event = params.1;
        
        for Slot { boxed, .. } in events.device.iter() {
            boxed(world, event)
        }
    }
}