use std::{sync::Mutex, collections::HashMap};

// Global
use lazy_static::lazy_static;
lazy_static! {
    static ref CALLBACKM: Mutex<CallbackManager> = Mutex::new(CallbackManager::default());
}

// Global functions
pub fn add_callback<F>(callback: F) 
where 
    F: Fn() + 'static + Send
{
    let mut x = CALLBACKM.lock().unwrap();
    x.add_callback(callback);
}
pub fn update() {
    let mut x = CALLBACKM.lock().unwrap();
    x.update();
}

// Self explanatory
#[derive(Default)]
pub struct CallbackManager {
    pub callbacks: HashMap<(std::thread::ThreadId, usize), Box<dyn Fn() + Send>>, // Callbacks that are going to be run eventually
    pub callbacks_to_run: Vec<usize> // Callbacks that need to run the next time we run the update loop
}

impl CallbackManager {
    // Add a callback
    pub fn add_callback<F>(&mut self, callback: F) 
    where 
        F: Fn() + 'static + Send
    {
        let boxed = Box::new(callback);
        // Get the callback's ID and the thread's ID
        let thread_id = std::thread::current().id();
        let callback_id = self.callbacks.len();
        self.callbacks.insert((thread_id, callback_id), boxed);
    }
    // Update the callback manager on a specific thread so we can run that specific's thread callback
    pub fn update(&mut self) {
        let thread_id = std::thread::current().id();
        // Get the valid callbacks for this thread 
        let mut valid_callbacks = self.callbacks_to_run.clone().into_iter().filter_map(|x| {
            let callback = self.callbacks.remove(&(thread_id, x));
            callback
        }).collect::<Vec<Box<dyn Fn() + Send>>>();
        // Run the callbacks now on the calling thread
        for callback in valid_callbacks.iter_mut() {
            callback();
        }
    }
}