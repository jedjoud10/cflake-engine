use world::World;

// This is an event set that contains multiple events that will get executed at some point later in time
// These events are sorted by their "priority", since they are executed sequentially
pub struct Events<E: Event> {
    // The main vector that contains all the event function pointers and their current priority
    vec: Vec<(E, i32)>,

    // The next priority index that we will use when running the register() method
    next: i32,
}

impl<E: Event> Default for Events<E> {
    fn default() -> Self {
        Self { vec: Default::default(), next: Default::default() }
    }
}

// Main marker trait so we can't add random types into the Event set
pub trait Event: Clone + Sized + 'static {
}

impl<'a, T: Callable<'a> + Clone + Sized + 'static> Event for T {}

// Main callable event trait
// This is the trait that we must implement for our function pointers
pub trait Callable<'a> {
    type Data: 'a;
    fn execute(self, data: &mut Self::Data);
}

// Generalized world event
impl<'a> Callable<'a> for fn(&mut World) {
    type Data = World;
    fn execute(self, data: &mut Self::Data) {
        (self)(data);
    }
}

impl<E: Event> Events<E> {  
    // Register a new event into the current thread local event handler
    // This will place the event as the last element in the queue 
    pub fn register(&mut self, event: E) {
        self.register_with(event, self.next);
        self.next += 1;
    }

    // This is like the register() method, but it places the event with a specific priority compared to other events
    // This will be useful if some events must be executed before others
    pub fn register_with(&mut self, event: E, priority: i32) {
        self.vec.push((event, priority));
    }

    // This will sort all the events based on their priority
    pub fn sort(&mut self) {
        self.vec.sort_by(|(_, a), (_, b)| i32::cmp(a, b));
    } 

    // Execute all the events using it's proper input data
    pub fn execute<'a>(&self, data: &mut E::Data) where E: Callable<'a> {
        for (event, _) in self.vec.iter().cloned() {
            event.execute(data);
        }
    }
}