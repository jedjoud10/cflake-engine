use std::{collections::{HashMap, HashSet}, mem::{ManuallyDrop, size_of}, ffi::c_void};

use rendering::{object::ObjectID, basics::shader::Shader, pipeline::Pipeline};
use crate::{Root, InstancedBatch, ElementID};

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
    pub fn draw(&mut self, pipeline: &mut Pipeline, root: &mut Root, window_size: veclib::Vector2<u16>) {
        let window_size: veclib::Vector2<f32> = window_size.into();
        // Get the elements that we have added and add them
        let added = std::mem::take(&mut root.added);
        let max_depth = root.calculate_max_depth();
        for added_id in added {
            let element = root.get_element(added_id).unwrap();
            // Add the element to the respective batch
            let batch = self.batches.entry(element.shader).or_insert(InstancedBatch::new());
            // Add the per instance parameters now
            // We will all the default values for these, since we're going to be updating them in a later step anyways
            batch.texture_uvs_buf.push(Default::default());
            batch.screen_uvs_buf.push(Default::default());
        } 
        // We gotta update the depth and screen_uvs values for every element, even if we didn't mutate it
        let all = root.elements.iter();
        for (id, element) in all {
            // Always update the depth and uvs
            let id = ElementID(id);
            let batch = self.batches.get_mut(&element.shader).unwrap();
            let index = batch.instances.get(&id).unwrap();
            // These values must always be updated since we don't know when they will change
            batch.depth_buf.update(*index, |x| *x = element.depth as f32 / max_depth as f32);            
            batch.screen_uvs_buf.update(*index, |x| *x = {
                // Calculate the screen UVs (min, max) using the center position and size
                let center = element.center;
                let size = element.size;
                // Calculate min and max now
                let min: veclib::Vector2<f32> = veclib::vec2(center.x - size.x / 2, center.y - size.y / 2).into();
                let max: veclib::Vector2<f32> = veclib::vec2(center.x + size.x / 2, center.y + size.y / 2).into();
                // Convert from pixel coordinates to UV coordinates
                let min = min / window_size;
                let max = max / window_size;
                veclib::vec4(min.x, min.y, max.x, max.y)
            })
        }

        // Update some values if we mutated the values
        let mutated = std::mem::take(&mut root.mutated);
        for mutated_id in mutated {
            // Always update the depth
            let element = root.get_element(mutated_id).unwrap();
            let batch = self.batches.get_mut(&element.shader).unwrap();
            let index = *batch.instances.get(&mutated_id).unwrap();
            // Update some values if needed
            batch.texture_uvs_buf.update(index, |x| *x = element.texture_uvs);
            batch.colors_buf.update(index, |x| *x = element.color);
        }

        // Remove 
        let removed = std::mem::take(&mut root.removed);
        for (removed_id, shader) in removed {
            // Remove the elements from the buffers
            let batch = self.batches.get_mut(&shader).unwrap();
            let index = *batch.instances.get(&removed_id).unwrap();
            // Now we can remove the value from the buffers
            batch.depth_buf.swap_remove(index);
            batch.screen_uvs_buf.swap_remove(index);
            batch.texture_uvs_buf.swap_remove(index);
            batch.colors_buf.swap_remove(index);
        }
    
        // Now we can actually draw the elements as multiple instanced batches
        for (shader, batch) in self.batches.iter() {
            // Get the shader, create some uniforms, then draw
            let shader = pipeline.get
        }
    }
}