#[derive(Clone, Copy, Hash, PartialEq, Eq, Default)]
// Just a simple ID stored in each GPU object, that way we can save a bit of memory when dealing with GPU objects that do not have any functions
pub struct GPUObjectID {
    pub index: usize,
}
