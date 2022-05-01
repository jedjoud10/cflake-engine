use std::{ptr::NonNull, alloc::Layout};


// Multiple vertices and their attributes
#[derive(Default)]
pub struct VertexSet {
    // Positions in 3D
    pub(super) positions: NonNull<vek::Vec3<f32>>,
    
    // Normal direction for each vertex
    pub(super)normals: NonNull<vek::Vec3<i8>>,

    // Tangents of the normals
    pub(super) tangents: NonNull<vek::Vec4<i8>>,
    
    // Texture coordinates for each vertex
    pub(super) uvs: NonNull<vek::Vec2<u8>>,

    // Unique vertex color, in case we need it
    pub(super) colors: NonNull<vek::Rgb<u8>>,

    // Number of vertices we have in total
    pub(super) len: usize,

    // I hate myself
    pub(super) cap: usize,
}

// Pre-allocate some memory with capacity "cap", for an array of elements of type "T"
unsafe fn with<T>(cap: usize) -> NonNull<T> {
    let ptr = std::alloc::alloc(Layout::new::<T>());
    NonNull::new(ptr as *mut T)
}

impl VertexSet {
    // Create a vertex set with a specific capacity count
    pub fn with_capacity(cap: usize) -> Self {
        unsafe {
            Self {
                positions: with(cap),
                normals: with(cap),
                tangents: with(cap),
                uvs: with(cap),
                colors: with(cap),
                len: 0,
                cap,
            }
        }
    }
    // Create an empty vertex set
    pub const fn new() -> Self {
        Self {
            positions: NonNull::dangling(),
            normals: NonNull::dangling(),
            tangents: NonNull::dangling(),
            uvs: NonNull::dangling(),
            colors: NonNull::dangling(),
            len: 0,
            cap: 0
        }
    }

    // Get an immutable slice of an attribute layout
    pub fn as_slice<VertexLayout>(&self) {
    }
}