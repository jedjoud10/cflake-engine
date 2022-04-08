// Stores the pointer and lengths of each component storage
pub struct QueryCache {
    // Mask of the component storage, the 
    ptrs: Vec<(Mask, *mut c_void, usize)>
}