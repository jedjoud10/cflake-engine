use std::{collections::{HashMap, HashSet}, mem::{ManuallyDrop, size_of}, ffi::c_void};

use rendering::{object::ObjectID, basics::shader::Shader};
use crate::{Root, InstancedBatch};

// A buffer containing the instanced model that we will use for rendering
// We will always at the end of the frame as well
pub struct RenderingBuffer {
    // Instanced batches used for rendering
    pub batches: HashMap<ObjectID<Shader>, InstancedBatch>,
}

impl RenderingBuffer {
    // Create a new rendering buffer with a certain capacity to hold some default elements 
    pub fn new(capacity: usize) -> Self {
        Self {
            batches: HashMap::with_capacity(capacity),
        }        
    }
    // Draw all the elements that are part of the root
    // We must run this one the render thread
    pub fn draw(&mut self, root: &mut Root) {
        // Get the elements that we have added and add them
        let added = std::mem::take(&mut root.added);
        for added_id in added {
            let element = root.element(added_id).unwrap();
            // Add the element to the respective batch
            let batch = self.batches.entry(&element.shader).or_insert(InstancedBatch::new());
            // Add the element's data to our dynamic arrays
            self.push(element.depth as f32 / root.max_depth as f32);
            // Calculate the screen position and min and max uvs
            self.screen_uvs_buf.push(element.)
        } 
    }
}