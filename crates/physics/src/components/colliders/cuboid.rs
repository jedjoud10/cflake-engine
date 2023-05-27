use ecs::Component;

// Cuboid colliders represent a cuboid in 3D space
// The position and rotation of the cuboid will be fetched from it's Position component and Rotation component
#[derive(Component)]
pub struct CuboidCollider {
    pub half_extent: vek::Extent3<f32>,
    pub mass: f32,
    pub friction: f32,
    pub restitution: f32,
    pub(crate) handle: Option<rapier3d::geometry::ColliderHandle>,
}

impl Clone for CuboidCollider {
    fn clone(&self) -> Self {
        Self {
            half_extent: self.half_extent,
            mass: self.mass,
            handle: None,
            friction: self.friction,
            restitution: self.restitution,
        }
    }
} 

impl CuboidCollider {
    // Create a new cuboid collider with a specific half-extent and mass
    pub fn new(half_extent: vek::Extent3<f32>, mass: f32, friction: f32, restitution: f32,) -> Self {
        Self {
            half_extent,
            mass,
            handle: None,
            friction: 0.2,
            restitution: 0.2,
        }
    }
}