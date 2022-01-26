use std::sync::{Arc, Mutex};

use ahash::AHashMap;
use worker_threads::ThreadPool;

use crate::entity::EntityID;

use super::LinkedComponents;

pub enum ComponentQueryIterType {
    ArcHashMap(Arc<Mutex<AHashMap<EntityID, LinkedComponents>>>),
    Vec(Vec<LinkedComponents>),
}

// A struct full of LinkedComponents that we send off to update in parallel
// This will use the components data given by the world to run all the component updates in PARALLEL
// The components get mutated in parallel, though the system is NOT stored on another thread
pub struct ComponentQuery {
    // The actual components
    pub(crate) linked_components: Option<ComponentQueryIterType>,
    // Thread pool because I am insane
    pub(crate) thread_pool: Arc<Mutex<ThreadPool<LinkedComponents>>>,
}

impl ComponentQuery {
    // Update all the components consecutively, on the main thread
    pub fn update_all<F: FnMut(&mut LinkedComponents)>(self, mut function: F) {
        // Run it normally
        if let Some(_type) = self.linked_components {
            match _type {
                ComponentQueryIterType::ArcHashMap(arc) => {
                    let mut lock = arc.lock().unwrap();
                    for (_, linked_components) in lock.iter_mut() {
                        function(&mut linked_components);
                    }
                },
                ComponentQueryIterType::Vec(mut vec) => for mut linked_components in vec {
                    function(&mut linked_components);
                },
            }
        }
    }
    // Update all the components consecutively, on the main thread, but we can break out from the inner loop whenever we pass it an Option::None at the end
    pub fn update_all_breakable<F: FnMut(&mut LinkedComponents) -> Option<()>>(self, mut function: F) {
        // Run it normally
        if let Some(_type) = self.linked_components {
            match _type {
                ComponentQueryIterType::ArcHashMap(arc) => {
                    let mut lock = arc.lock().unwrap();
                    for (_, linked_components) in lock.iter_mut() {
                        let opt = function(&mut linked_components);
                        if opt.is_none() { break; }
                    }
                },
                ComponentQueryIterType::Vec(mut vec) => for mut linked_components in vec {
                    let opt = function(&mut linked_components);
                    if opt.is_none() { break; }
                },
            }
        }
    }
    // Update all the components consecutively, on the main thread, but while also mapping each element and returning a new vector that may or may not contain each element
    pub fn update_all_map_filter<U, F: FnMut(&mut LinkedComponents) -> Option<U>>(self, mut function: F) -> Option<Vec<U>> {
        let output_vec = if let Some(_type) = self.linked_components {
            let len = match _type {
                ComponentQueryIterType::ArcHashMap(x) => x.lock().unwrap().len(),
                ComponentQueryIterType::Vec(x) => x.len(),
            };
            // Create the output vector that will store the mapped values
            let mut output_vec = Vec::with_capacity(self.linked_components.as_ref().map(|x| len).unwrap_or_default());

            match _type {
                ComponentQueryIterType::ArcHashMap(arc) => {
                    let mut lock = arc.lock().unwrap();
                    for (_, linked_components) in lock.iter_mut() {
                        let output = function(&mut linked_components);
                        if let Some(output) = output { output_vec.push(output); }
                    }
                },
                ComponentQueryIterType::Vec(mut vec) => for mut linked_components in vec {
                    for linked_components in vec.iter_mut() {
                        let output = function(&mut linked_components);
                        if let Some(output) = output { output_vec.push(output); }
                    }
                },
            }   
            Some(output_vec)
        } else { None };
        output_vec
    }
    // Update all the components in parallel, on multiple worker threads
    pub fn update_all_threaded<F: Fn(&mut LinkedComponents) + 'static + Sync + Send>(self, function: F) {
        if let Some(lock) = self.linked_components {
            let mut vec = lock.lock().unwrap();
            let thread_pool = self.thread_pool.lock().unwrap();
            thread_pool.execute(&mut vec, function);
        }
    }
}
