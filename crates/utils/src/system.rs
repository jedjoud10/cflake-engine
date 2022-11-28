use crate::ThreadPool;
use world::{user, System, World};

// Add the threadpool resource to the world
pub fn system(system: &mut System) {
    system
        .insert_init(|world: &mut World| {
            world.insert(ThreadPool::default())
        })
        .before(user);
}
