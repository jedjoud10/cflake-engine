use super::System;

/// Takes in multiple SystemScheduling structs and creates a tree of all the systems that must execute
/// This function will try to maximize parallelism, and make as many execute in parallel as possible
/// This will also try to handle sorting the events based on their dependencies and inter-dependencies
/// TODO: Actually make it work with parallel execution
fn schedule<E>(systems: &[&dyn System<E>]) {

}