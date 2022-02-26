use half::f16;

#[repr(align(16), C)]
#[derive(Default, Clone, Copy, Debug)]
// A packed edit that will be sent to the GPU
pub struct PackedEdit {
    pub center: veclib::Vector3<f16>,
    pub size: veclib::Vector3<f16>,
    pub rgbcolor_shape_type_edit_type_material: u32,
}
