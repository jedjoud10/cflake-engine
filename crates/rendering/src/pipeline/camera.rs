// A camera object that can be sent to the render thread to update the main camera that is rendering the world
#[derive(Default)]
pub struct Camera {
    // Position and rotation
    pub position: veclib::Vector3<f32>,
    pub rotation: veclib::Quaternion<f32>,

    // View and projection matrices
    pub viewm: veclib::Matrix4x4<f32>,
    pub projm: veclib::Matrix4x4<f32>,
    
    // Near-Far clip planes    
    pub clip_planes: veclib::Vector2<f32>,
}