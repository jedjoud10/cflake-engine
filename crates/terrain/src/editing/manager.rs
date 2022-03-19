use super::{Edit, PackedEdit};
use crate::{pack_color, ChunkCoords};
use half::f16;
use math::{octrees::Octree, shapes::ShapeType};
use slotmap::SlotMap;

slotmap::new_key_type! {
    pub struct EditKey; 
}

// An editing manager that contains all the world edits
#[derive(Default)]
pub struct EditingManager {
    // New edits and total edits
    mutated: Vec<EditKey>,
    edits: SlotMap<EditKey, Edit>,
}

impl EditingManager {
    // Get a specific edit using an edit key
    pub fn get(&self, key: EditKey) -> Option<&Edit> { self.edits.get(key) }
    pub fn get_mut(&mut self, key: EditKey) -> Option<&mut Edit> { self.mutated.push(key); self.edits.get_mut(key) }
    // Add a new edit
    pub fn edit(&mut self, edit: Edit) -> EditKey {
        let key = self.edits.insert(edit);
        self.mutated.push(key);
        key
    }
    // Using an octree, check what chunks need to be edited
    pub fn get_influenced_chunks(&mut self, octree: &Octree) -> Vec<ChunkCoords> {
        // Get the nodes
        let shapes = self.mutated.drain(..).map(|key| self.edits.get(key).unwrap().shape.clone()).collect::<Vec<_>>();
        let nodes = math::intersection::shapes_octree(&shapes, octree);
        // Get the chunks coordiantes
        nodes.into_iter().map(ChunkCoords::new).collect::<Vec<_>>()
    }
    // Check if we have any new pending edits
    pub fn is_pending(&self) -> bool {
        !self.mutated.is_empty()
    }
    // Convert the list of edits to a list of packed edits
    pub fn convert(&self) -> Vec<PackedEdit> {
        self.edits
            .iter()
            .map(|(_, edit)| {
                // Center, size, shapetype
                let (center, size, shapetype) = match &edit.shape {
                    ShapeType::Cuboid(cuboid) => (cuboid.center, cuboid.size, 0u8),
                    ShapeType::Sphere(sphere) => (sphere.center, vek::Vec3::new(sphere.radius, 0.0, 0.0), 1u8),
                    ShapeType::VerticalCapsule(_) => todo!(),
                };
                // Get the edittype
                let params = edit.params.clone();
                let rgbcolor = (pack_color(params.color) as u32) << 16; // 2
                let shape_type_edit_type = (((shapetype << 4) | (!params._union as u8)) as u32) << 8; // 1
                let material = params.material.unwrap_or(255) as u32; // 1
                let rgbcolor_shape_type_edit_type_material = rgbcolor | shape_type_edit_type | material;

                PackedEdit {
                    center: vek::Vec3::new(f16::from_f32(center.x), f16::from_f32(center.y), f16::from_f32(center.z)),
                    size: vek::Vec3::new(f16::from_f32(size.x), f16::from_f32(size.y), f16::from_f32(size.z)),
                    rgbcolor_shape_type_edit_type_material,
                }
            })
            .collect::<Vec<_>>()
    }
}
