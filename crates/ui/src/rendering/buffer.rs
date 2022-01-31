use std::collections::{HashMap, HashSet};

use multimap::MultiMap;
use ordered_vec::simple::OrderedVec;
use rendering::basics::model::{Model2D, Model2DBuffers};

use crate::Root;

// A buffer containing the instanced model that we will use for rendering
// We will always at the end of the frame as well
pub struct UIRenderingBuffer {
    // Instanced model buffer
    pub vao: u32,
    pub verex_buf: u32,
    pub instanced_model: Model2D,

    // Per instance settings
    pub depth: Vec<f32>,
    pub uvs: Vec<(veclib::Vector2<f32>, veclib::Vector2<f32>)>,
    pub instance_count: usize,
}

impl UIRenderingBuffer {
    // Update our internal instanced model data
    pub fn update_data(&mut self, root: &Root) {
    }
    // Draw all the elements that are part of the root
    pub fn draw(&self, root: &Root) {
    }
}