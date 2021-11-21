use crate::ChunkCoords;
use ecs::EntityManager;
use rendering::ComputeShaderGPUObject;
use std::collections::HashMap;

// Some detail that has a specific entity instance ID related to it that will be spawned in the wolrd
pub struct TDetail {
    pub entity_instance_id: usize,
    pub entity_base_id: usize,
    pub location: veclib::Vector3<f32>,
    pub rotation: veclib::Quaternion<f32>,
    pub scale: f32,
}

// Manager for detail spawning and such
pub struct DetailManager {
    pub chunks_to_groups: HashMap<veclib::Vector3<i64>, Vec<TDetail>>,
}

impl DetailManager {
    // Read back the details from the compute shader for a specific chunk
    pub fn load_details(&mut self, _chunk_entity_id: usize, _compute_shader: &mut ComputeShaderGPUObject, _entity_manager: &mut EntityManager) {
        // Load back the detail
        // Append the generated detail to it's corresponding detail group
    }
    // Delete a specific detail group from the detail manager when it's chunk gets unloaded
    pub fn unload_detail(&mut self, chunk_coords: &ChunkCoords, entity_manager: &mut EntityManager) {
        // The removed detail
        let mut removed_entities: Vec<usize> = Vec::new();
        // Ez check first
        if self.chunks_to_groups.contains_key(&chunk_coords.center) {
            for tdetail in self.chunks_to_groups.get(&chunk_coords.center).unwrap().iter() {
                removed_entities.push(tdetail.entity_instance_id);
            }
        }
        // Remove the detail etities from the entity manager
        for entity_id in removed_entities {
            entity_manager.remove_entity_s(entity_id).unwrap();
        }
    }
}
