use ecs::*;
use math;

// An AABB components
#[derive(Default)]
pub struct AABB {
    pub aabb: math::bounds::AABB,
    pub generation_type: AABBGenerationType,
}

// How we are going to generate the AABB
pub enum AABBGenerationType {
    RenderEntity,
    Manual,
}

// Automatically try to load the AABB from the components of a render entity (Position, Scale, Render)
impl Default for AABBGenerationType {
    fn default() -> Self {
        Self::RenderEntity
    }
}

// AABB component functions
impl AABB {
    // Generate the AABB from a renderer entity
    pub fn from_components(entity: &Entity, component_manager: &ComponentManager) -> Self {
        let renderer = &entity.get_component::<rendering::Renderer>(component_manager).unwrap();
        let transform = entity.get_component::<super::Transform>(component_manager).unwrap();
        // Check if we are using a multi material renderer
        match &renderer.multi_material {
            Some(x) => {
                // Get the AABB of each sub model and take the biggest one
                let mut aabb = math::bounds::AABB::default();
                for (sub_model, _) in x.sub_models.iter() {
                    let sub_aabb = math::bounds::AABB::from_model(&sub_model.vertices);
                    aabb.min = sub_aabb.min.min(aabb.min);
                    aabb.max = sub_aabb.max.max(aabb.max);
                }
                Self { aabb, ..Self::default() }
            },
            None => {
                let mut aabb = math::bounds::AABB::from_model(&renderer.model.vertices);
                aabb.transform(&transform.get_matrix());
                Self { aabb, ..Self::default() }
            },
        }        
    }
}

// Main traits implemented
ecs::impl_component!(AABB);
