use half::f16;

#[repr(align(8))]
#[derive(Default, Clone, Copy)]
// A packed edit that will be sent to the GPU
pub struct PackedEdit {
    pub center: veclib::Vector3<f16>, // 6
    pub size: veclib::Vector3<f16>,     // 6
    pub rgb_color: u16,                 // 2
    pub shapetype_edittype: u8,         // 1
    pub material: u8,                   // 1
}