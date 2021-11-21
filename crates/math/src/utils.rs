// Some transform that contains position, scale and rotation
#[derive(Default)]
pub struct Transform {  
    pub position: veclib::Vector3<f32>,
    pub rotation: veclib::Quaternion<f32>,
    pub scale: veclib::Vector3<f32>,
}