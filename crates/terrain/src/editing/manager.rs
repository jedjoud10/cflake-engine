use super::{Edit, PackedEdit};
use crate::{ChunkCoords, pack_color};
use half::f16;
use math::{octrees::Octree, shapes::BasicShapeType};

// An editing manager that contains all the world edits
#[derive(Default)]
pub struct EditingManager {
    // Collection of total edits
    edits: Vec<Edit>,
}

impl EditingManager {
    // Add a new edit
    pub fn edit(&mut self, edit: Edit) {
        self.edits.push(edit);
    }

    // Using an octree, check what chunks need to be edited
    pub fn get_influenced_chunks(&self, octree: &Octree) -> Vec<ChunkCoords> {
        // Get the nodes
        let shapes = self.edits.iter().map(|edit| edit.shape.clone()).collect::<Vec<_>>();
        let nodes = math::intersection::shapes_octree(&shapes, octree);
        // Get the chunks coordiantes
        nodes.into_iter().map(ChunkCoords::new).collect::<Vec<_>>()
    }
    // Convert the list of edits to a list of packed edits
    pub fn convert(&self) -> Vec<PackedEdit> {
        self.edits.iter().map(|edit| {
            // Center, size, shapetype
            let (center, size, shapetype) = match &edit.shape {
                BasicShapeType::Cube(cube) => (cube.center, cube.size, 0u8),
                BasicShapeType::Sphere(sphere) => (sphere.center, veclib::vec3(sphere.radius * 2.0, 0.0, 0.0), 1u8)
            };
            // Get the edittype
            PackedEdit {
                center: veclib::vec3(f16::from_f32(center.x), f16::from_f32(center.y), f16::from_f32(center.z)),
                size: veclib::vec3(f16::from_f32(size.x), f16::from_f32(size.y), f16::from_f32(size.z)),
                rgb_color: pack_color(edit.color),
                shapetype_edittype: (shapetype << 4) | (edit.operation as u8),
                material: edit.material.unwrap_or(255),
            }
        }).collect::<Vec<_>>()
    }
}
