use std::cell::Cell;

use ecs::Entity;
use utils::Handle;

use crate::PhysicsSurface;

// Implemented for all types of colliders to fetch common data
pub(crate) trait GenericCollider {
    type RawRapierCollider;
    fn handle(&self) -> Option<rapier3d::geometry::ColliderHandle>;
    fn set_handle(&mut self, handle: rapier3d::geometry::ColliderHandle);
    fn modified(&self) -> &Cell<bool>;
    fn mass(&self) -> f32;
    fn material(&self) -> &Option<Handle<PhysicsSurface>>;
    fn cast_rapier_collider(
        generic: &mut rapier3d::geometry::Collider,
    ) -> &mut Self::RawRapierCollider;
    fn regenerate_when_updating() -> bool;
    fn build_collider(&mut self, entity: &Entity) -> Option<rapier3d::geometry::Collider>;
    fn set_custom_rapier_collider_settings(&self, custom: &mut Self::RawRapierCollider);
}
