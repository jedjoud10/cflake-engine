use std::sync::{Arc, Mutex};

use ahash::AHashMap;
use worker_threads::ThreadPool;

use crate::entity::EntityID;

use super::LinkedComponents;

// An enum that stores either a reference to a hashmap or an owned vector. We will use this to iterate through every LinkedComponents
pub enum ComponentQueryIterType {
    ArcHashMap(Arc<Mutex<AHashMap<EntityID, LinkedComponents>>>),
    HashMap(AHashMap<EntityID, LinkedComponents>),
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
    // Update a single linked component from this query using it's respective entity ID
    pub fn update<F: FnMut(&mut LinkedComponents)>(self, id: EntityID, mut function: F) {
        if let Some(_type) = self.linked_components {
            match _type {
                ComponentQueryIterType::ArcHashMap(arc) => {
                    let mut lock = arc.lock().unwrap();
                    let linked = lock.get_mut(&id);
                    if let Some(linked) = linked {
                        function(linked);
                    }
                }
                ComponentQueryIterType::HashMap(mut hashmap) => {
                    let linked = hashmap.get_mut(&id);
                    if let Some(linked) = linked {
                        function(linked);
                    }
                }
            }
        }
    }
    // Update all the components consecutively, on the main thread
    pub fn update_all<F: FnMut(&mut LinkedComponents)>(self, mut function: F) {
        // Run it normally
        if let Some(_type) = self.linked_components {
            match _type {
                ComponentQueryIterType::ArcHashMap(arc) => {
                    let mut lock = arc.lock().unwrap();
                    for (_, linked_components) in lock.iter_mut() {
                        function(linked_components);
                    }
                }
                ComponentQueryIterType::HashMap(hashmap) => {
                    for (_, mut linked_components) in hashmap {
                        function(&mut linked_components);
                    }
                }
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
                        let opt = function(linked_components);
                        if opt.is_none() {
                            break;
                        }
                    }
                }
                ComponentQueryIterType::HashMap(hashmap) => {
                    for (_, mut linked_components) in hashmap {
                        let opt = function(&mut linked_components);
                        if opt.is_none() {
                            break;
                        }
                    }
                }
            }
        }
    }
    // Update all the components consecutively, on the main thread, but while also mapping each element and returning a new vector that may or may not contain each element
    pub fn update_all_map_filter<U, F: FnMut(&mut LinkedComponents) -> Option<U>>(self, mut function: F) -> Option<Vec<U>> {
        let output_vec = if let Some(_type) = self.linked_components {
            let len = match &_type {
                ComponentQueryIterType::ArcHashMap(x) => x.lock().unwrap().len(),
                ComponentQueryIterType::HashMap(x) => x.len(),
            };
            // Create the output vector that will store the mapped values
            let mut output_vec = Vec::with_capacity(len);

            match _type {
                ComponentQueryIterType::ArcHashMap(arc) => {
                    let mut lock = arc.lock().unwrap();
                    for (_, linked_components) in lock.iter_mut() {
                        let output = function(linked_components);
                        if let Some(output) = output {
                            output_vec.push(output);
                        }
                    }
                }
                ComponentQueryIterType::HashMap(hashmap) => {
                    for (_, mut linked_components) in hashmap {
                        let output = function(&mut linked_components);
                        if let Some(output) = output {
                            output_vec.push(output);
                        }
                    }
                }
            }
            Some(output_vec)
        } else {
            None
        };
        output_vec
    }
    // Update all the components in parallel, on multiple worker threads
    pub fn update_all_threaded<F: Fn(&mut LinkedComponents) + Sync + Send>(self, function: F) {
        if let Some(_type) = self.linked_components {
            let thread_pool = self.thread_pool.lock().unwrap();
            match _type {
                ComponentQueryIterType::ArcHashMap(arc) => {
                    let mut lock = arc.lock().unwrap();
                    let vec = lock.iter_mut().map(|(_, x)| x as *mut LinkedComponents).collect::<Vec<_>>();
                    thread_pool.execute_vec_ptr(vec, function);
                }
                ComponentQueryIterType::HashMap(mut hashmap) => {
                    let vec = hashmap.iter_mut().map(|(_, x)| x as *mut LinkedComponents).collect::<Vec<_>>();
                    thread_pool.execute_vec_ptr(vec, function);
                }
            }
        }
    }
}
