use ecs::Component;

// Sphere colliders represent perfect spheres in 3D space
// The position of the sphere will be fetched from it's Position component
#[derive(Component)]
pub struct SphereCollider {
    pub radius: f32,
    pub mass: f32,
    pub friction: f32,
    pub restitution: f32,
    pub(crate) handle: Option<rapier3d::geometry::ColliderHandle>,
}

impl Clone for SphereCollider {
    fn clone(&self) -> Self {
        Self {
            radius: self.radius.clone(),
            mass: self.mass.clone(),
            handle: None,
            friction: self.friction,
            restitution: self.restitution,
        }
    }
} 

impl SphereCollider {
    // Create a new sphere collider with a specific radius and mass
    pub fn new(radius: f32, mass: f32, friction: f32, restitution: f32,) -> Self {
        Self {
            radius,
            mass,
            handle: None,
            friction,
            restitution,
        }
    }
}