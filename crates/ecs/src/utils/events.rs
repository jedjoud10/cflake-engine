use std::{
    cell::{Ref, RefCell},
    fmt::{Debug, Display},
    rc::Rc,
    time::{Duration, Instant},
};

use crate::EventExecutionOrder;

// A single event
pub type Event<World> = fn(&mut World);

// A single event timing
#[derive(Clone)]
pub struct ProfiledEventTiming {
    ordering: i32,
    elapsed: Duration,
}

impl Display for ProfiledEventTiming {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Event '{}' took {}micros to execute", self.ordering, self.elapsed.as_micros())
    }
}

impl Debug for ProfiledEventTiming {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self, f)
    }
}

// Inner system set that will be duped using a Rc<RefCell<>>
struct InnerSystemSet<World> {
    events: RefCell<Vec<(i32, Event<World>)>>,
    profiled: RefCell<(Vec<ProfiledEventTiming>, Instant)>,
}

// Multiple events that will be stored in the world
pub struct SystemSet<World> {
    inner: Rc<InnerSystemSet<World>>,
}

impl<World> SystemSet<World> {
    // Insert an event into the system set
    pub fn insert(&mut self, evn: fn(&mut World)) {
        let idx = EventExecutionOrder::fetch_add();
        self.inner.events.borrow_mut().push((idx, evn));
    }
    // Insert an event that executes at a specific order index
    pub fn insert_with(&mut self, evn: fn(&mut World), order: i32) {
        self.inner.events.borrow_mut().push((order, evn));
    }
    // Sort the events based on their execution order index
    pub fn sort(&mut self) {
        let mut borrowed = self.inner.events.borrow_mut();
        borrowed.shrink_to_fit();
        borrowed.sort_by(|(a, _), (b, _)| i32::cmp(a, b));
    }
    // Run all the events, in order
    pub fn execute(&self, world: &mut World) {
        // Borrowing the refcells and setting vector size
        let events = self.inner.events.borrow();
        let mut profiled = self.inner.profiled.borrow_mut();
        profiled.0.resize(
            events.len(),
            ProfiledEventTiming {
                ordering: 0,
                elapsed: Duration::ZERO,
            },
        );
        drop(profiled);

        // Profile the time it took to execute each event
        let mut profiled = Vec::with_capacity(events.len());
        for &(ordering, f) in &*events {
            // Start profiling the event time
            let now = Instant::now();

            // Executing the event
            f(world);

            // Saving how much time it took to execute
            profiled.push(ProfiledEventTiming { ordering, elapsed: now.elapsed() });
        }

        // Check if we need to update the timings
        if self.inner.profiled.borrow().1.elapsed().as_secs() >= 1 {
            self.inner.profiled.replace((profiled, Instant::now()));
        }
    }
    // Get the event timings from last frame
    pub fn get_timings(&self) -> Ref<(Vec<ProfiledEventTiming>, Instant)> {
        self.inner.profiled.borrow()
    }
}

impl<World> Default for SystemSet<World> {
    fn default() -> Self {
        Self {
            inner: Rc::new(InnerSystemSet {
                events: Default::default(),
                profiled: RefCell::new((Default::default(), Instant::now())),
            }),
        }
    }
}

impl<World> Clone for SystemSet<World> {
    fn clone(&self) -> Self {
        Self { inner: self.inner.clone() }
    }
}
