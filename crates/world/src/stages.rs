/*
use std::marker::PhantomData;

// Stages are a way for us to sort specific events to be able to sort them by their "priority"
// Ex. We can have a "rendering" stage for the Update event that tells it that the specific event must execute before the "ui" stage
pub struct Stage(Name, i32);

// Stage registry that will keep track of each stage and sort them
// This stage registry will be unique for EACH event type
pub struct StageRegistry<T> {
    stages: Vec<(Stage, i32)>,
    _phantom: PhantomData<T>,
}

// This is the name and key for each stage
type Name = &'static str;

impl Stage {
    // This creates a stage with no specific priority index. This event can be executed whenever
    pub fn new(name: Name) -> Self { Self(name, 0) }

    // Offset of i32::MIN. This stage will be one of the first to execute
    pub fn earliest(name: Name) -> Self { Self(name, i32::MIN) }

    // Offset of i32::MAX. This stage will be one of the last to execute
    pub fn latest(name: Name) -> Self { Self(name, i32::MAX) }
}

// We need to have a stage trait since we will deal with modifiers
trait Offset {
    // Get the offset index of the current staged struct (given any stage registry) using only it's name
    fn offset<T>(name: Name, registry: &StageRegistry<T>) -> i32;
}

struct Before<S: Offset>(Name, S);
struct After<S: Offset>(Name, S);

impl Offset for Stage {
    fn offset<T>(name: Name, registry: &StageRegistry<T>) -> i32 {
        todo!()
    }
}

impl<S: Offset> Offset for Before<S> {
    fn offset<T>(name: Name, registry: &StageRegistry<T>) -> i32 {
        todo!()
    }
}

impl<S: Offset> Offset for After<S> {
}


impl Stage {
    // This hints that the current stage must be executed before the given "before" stage
    pub fn before(self, before: &Name) -> Before<Self> {
        Before(before, self)
    }

    // This hints that the current stage must be execute after the given "after" stage
    pub fn after(self, after: &Name) -> After<Self> {
        After(after, self)
    }
}
*/