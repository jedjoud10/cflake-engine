use std::collections::HashMap;

use crate::cacher;

// An instance manager that keeps track of the number of unique instances in the world
#[derive(Default)]
pub struct InstanceManager {
    // The original name of the cached object and it's instance count
    pub instance_count: HashMap<String, u16>, 
}

impl InstanceManager {
    // Add a new instance to the world and return it's new object name
    pub fn add_instance(&mut self, object_name: &str) -> String {
        // Check if the value exists already, and increment it by one
        let x: &mut u16 = self.instance_count.entry(object_name.to_string()).or_insert(1);
        *x += 1;    
        let new_name = format!("{}_instance_#{}", object_name, &x);
        return new_name
    }
    // Remove an instance from the world
    pub fn remove_instance(&mut self, object_name: &str) {
        // Make sure the number can never be negative
        if self.instance_count.contains_key(object_name) {
            // Check the count
            if *self.instance_count.get(object_name).unwrap() == 1_u16 {
                // Remove the value
                self.instance_count.remove(object_name).unwrap();
            } else {
                // Decrement
                let x = self.instance_count.entry(object_name.to_string()).or_insert(1);
                *x -= 1;
            }
        }
    }
}

// An instance trait, can be added into objects that have their own cache manager and instance cloner
pub trait Instance {
    // Set the current name for this object
    fn set_name(&mut self, string: String);
    // Get the current name for this object
    fn get_name(&self) -> String; 
    // Create an instance of the current object and store it in it's cache manager
    fn insantiate_cm<'a>(&self, cache_manager: &'a mut cacher::CacheManager<Self>, instance_manager: &mut InstanceManager) -> &'a Self where Self: Sized + Clone {
        let mut instance = self.clone();
        // Get the instance name
        let name = instance_manager.add_instance(&self.get_name());
        // Get a new name for this insance
        instance.set_name(name.clone());
        let cached_object = cache_manager.cache_object(instance, &name);
        return cache_manager.id_get_object(cached_object).unwrap();
    }
    // Create an instance of the current object without storing in the cache manager
    fn insantiate(&self, instance_manager: &mut InstanceManager) -> Self where Self: Sized + Clone {
        let mut instance = self.clone();
        // Get the instance name
        let name = instance_manager.add_instance(&self.get_name());
        instance.set_name(name);
        return instance;
    }
}