use crate::engine::{math, rendering::model::Model};

// Debug functionality
pub struct Debug {
    pub debug_renderers: Vec<DebugRendererType>, 
}

impl Debug {
    // Draw the debug renderers
    pub fn draw_debug(&self, vp_matrix: glam::Mat4) {
        // Loop each one and construct lines out of them
        let mut lines: Vec<math::shapes::Line> = Vec::new();
        for renderer in self.debug_renderers.iter() {
            match renderer {
                DebugRendererType::CUBE(corners) => {
                    // Turn the corners into lines
                    // Bottom
                    lines.push(math::shapes::Line::construct(corners[0], corners[1]));
                    lines.push(math::shapes::Line::construct(corners[1], corners[2]));
                    lines.push(math::shapes::Line::construct(corners[2], corners[3]));
                    lines.push(math::shapes::Line::construct(corners[3], corners[0]));

                    // Side
                    lines.push(math::shapes::Line::construct(corners[0], corners[4]));
                    lines.push(math::shapes::Line::construct(corners[1], corners[5]));
                    lines.push(math::shapes::Line::construct(corners[2], corners[6]));
                    lines.push(math::shapes::Line::construct(corners[3], corners[7]));

                    // Top
                    lines.push(math::shapes::Line::construct(corners[4], corners[5]));
                    lines.push(math::shapes::Line::construct(corners[5], corners[6]));
                    lines.push(math::shapes::Line::construct(corners[6], corners[7]));
                    lines.push(math::shapes::Line::construct(corners[7], corners[4]));
                },
                DebugRendererType::SPHERE(center, radius) => todo!(),
                DebugRendererType::LINE(line) => {
                    // Just use the line lol
                    lines.push(*line);
                },
                DebugRendererType::MODEL(model) => todo!(),
            }
        }
    }
}

// The types of debug renderers
pub enum DebugRendererType {
    CUBE(Vec<glam::Vec3>),
    SPHERE(glam::Vec3, f32),
    LINE(math::shapes::Line),
    MODEL(Model),
}

// Trait
pub trait DebugRenderer {
    // Get the debug renderer from the current struct
    fn get_debug_renderer(&self) -> DebugRendererType;
}