use ecs::Component;

// Sphere colliders represent perfect spheres in 3D space
// The position of the sphere will be fetched from it's Position component
#[derive(Component)]
pub struct SphereCollider {
    pub radius: f32,
}