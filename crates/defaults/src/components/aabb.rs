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
    // Offset a pre-existing AABB with a transform
    pub fn offset(mut aabb: math::bounds::AABB, transform: &super::Transform) -> math::bounds::AABB {
        // Offset the shit
        aabb.min += transform.position;
        aabb.max += transform.position;
        // Scale it, the position is the center
        aabb.scale(transform.position, transform.scale);
        // Recalculate the center
        aabb.center = (aabb.min + aabb.max) / 2.0;
        aabb
    }
    // Generate the AABB from a renderer entity
    pub fn from_components(entity: &Entity, component_manager: &ComponentManager) -> Self {
        let renderer = &entity.get_component::<rendering::Renderer>(component_manager).unwrap();
        let transform = entity.get_component::<super::Transform>(component_manager).unwrap();
        // Check if we are using a multi material renderer
        match &renderer.multi_material {
            Some(x) => {
                // Get the AABB of each sub model and take the biggest one
                let mut aabb: Option<math::bounds::AABB> = None;
                for i in 0..(x.sub_models.len()) {
                    let aabb2 = math::bounds::AABB::new_vertices(&x.sub_models.get(i).unwrap().0.vertices);
                    match &mut aabb {
                        Some(aabb_valid) => {
                            aabb_valid.min = aabb_valid.min.min(aabb2.min);
                            aabb_valid.max = aabb_valid.max.max(aabb2.max);
                        },
                        None => { /* Set the default one */ aabb = Some(aabb2) },
                    }
                    
                }
                // Check if we have a valid AABB
                match aabb {
                    Some(mut aabb) => {
                        aabb.center = (aabb.min + aabb.max) / 2.0;
                        Self {
                            aabb: Self::offset(aabb, transform),
                            ..Self::default()
                        }
                    },
                    None => Self {
                        aabb: math::bounds::AABB::default(),
                        ..Self::default()
                    },
                }
                
            }
            None => {
                let aabb = math::bounds::AABB::new_vertices(&renderer.model.vertices);
                Self {
                    aabb: Self::offset(aabb, transform),
                    ..Self::default()
                }
            }
        }
    }
}

// Main traits implemented
ecs::impl_component!(AABB);
