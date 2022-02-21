// Gotta rewrite this CSG shit
#[repr(u8)]
#[derive(Clone, Copy)]
pub enum CSGOperation {
    Union = 0,
    Subtraction,
}
