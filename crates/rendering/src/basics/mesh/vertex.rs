use std::{ptr::NonNull, ops::Index, alloc::Layout, mem::ManuallyDrop};
use bitflags::bitflags;

// What attribute storages are enabled
bitflags! {
    pub struct AttributeLayout: u8 {
        const POSITION = 1;
        const NORMAL = 1 << 1;
        const TANGENT = 1 << 2;
        const COLOR = 1 << 3;
        const TEXCOORd = 1 << 4;
    }
}

// Marker attributes to make things look pretty lul
pub struct Position;
pub struct Normal;
pub struct Tangent;
pub struct Color;
pub struct TexCoord;


// A marker attribute trait that will be implemented for the marker attributes
pub trait Attribute {
    type Item;

    // Given a vertex set, get the corresponding attribute storage pointer
    fn storage(set: &VertexSet) -> &Storage<Self::Item>;  

    // Same as the function above, but this time it gets a mutable reference to the storage
    fn storage_mut(set: &mut VertexSet) -> &mut Storage<Self::Item>;
}

// Le implementation
impl Attribute for Position {
    type Item = vek::Vec3<f32>;

    fn storage(set: &VertexSet) -> &Storage<Self::Item> {
        &set.positions
    }

    fn storage_mut(set: &mut VertexSet) -> &mut Storage<Self::Item> {
        &mut set.positions
    }
}

impl Attribute for Normal {
    type Item = vek::Vec3<i8>;

    fn storage(set: &VertexSet) -> &Storage<Self::Item> {
        &set.normals
    }

    fn storage_mut(set: &mut VertexSet) -> &mut Storage<Self::Item> {
        &mut set.positions
    }
}

impl Attribute for Tangent {
    type Item = vek::Vec4<i8>;

    fn storage(set: &VertexSet) -> &Storage<Self::Item> {
        &set.tangents
    }

    fn storage_mut(set: &mut VertexSet) -> &mut Storage<Self::Item> {
        &mut set.tangents
    }
}

impl Attribute for Color {
    type Item = vek::Rgb<u8>;

    fn storage(set: &VertexSet) -> &Storage<Self::Item> {
        &set.colors
    }

    fn storage_mut(set: &mut VertexSet) -> &mut Storage<Self::Item> {
        &mut set.colors
    }
}

impl Attribute for TexCoord {
    type Item = vek::Vec2<u8>;

    fn storage(set: &VertexSet) -> &Storage<Self::Item> {
        &set.uvs
    }

    fn storage_mut(set: &mut VertexSet) -> &mut Storage<Self::Item> {
        &mut set.uvs
    }
}


// Vertex attributes that will be stored for layouts that implement Vertex
pub struct Storage<T>(Option<NonNull<T>>);

// A vertex set that contains multiple vertex attributes
pub struct VertexSet {
    // Unique vertex attributes
    positions: Storage<vek::Vec3<f32>>,
    normals: Storage<vek::Vec3<i8>>,
    tangents: Storage<vek::Vec4<i8>>,
    colors: Storage<vek::Rgb<u8>>,
    uvs: Storage<vek::Vec2<u8>>,

    // Allocation shit by ourselves
    len: usize,
    cap: usize,
}

// Create a dangling attribute vector
fn null<T>() -> Storage<T> {
    Storage(None)
}

// Convert an attribute storage into an immutable slice
unsafe fn convert<'a, T>(storage: &'a Storage<T>, len: usize) -> Option<&'a [T]> {
    storage.0.map(|ptr| std::slice::from_raw_parts(ptr.as_ptr() as *const T, len))    
}

// Convert an attribute storage into a mutable slice
unsafe fn convert_mut<'a, T>(storage: &'a Storage<T>, len: usize) -> Option<&'a mut [T]> {
    storage.0.map(|ptr| std::slice::from_raw_parts_mut(ptr.as_ptr(), len))   
}

// Convert an attribute arr
impl VertexSet {
    // Create a new empty vertex set
    pub fn new() -> Self {
        Self {
            positions: null(),
            normals: null(),
            tangents: null(),
            colors: null(),
            uvs: null(),
            len: 0,
            cap: 0,
        }
    }

    // Get the vertex attributes so we can read them
    pub fn attributes<'a>(&'a self) -> RefAttributes<'a> {
        unsafe {
            RefAttributes {
                positions: convert(&self.positions, self.len),
                normals: convert(&self.normals, self.len),
                tangents: convert(&self.tangents, self.len),
                colors: convert(&self.colors, self.len),
                uvs: convert(&self.uvs, self.len)
            }
        }
    }

    // Get the vertex attributes so we can mutate/write to them
    pub fn attributes_mut<'a>(&'a mut self) -> MutAttributes<'a> {
        unsafe {
            MutAttributes {
                positions: convert_mut(&self.positions, self.len),
                normals: convert_mut(&self.normals, self.len),
                tangents: convert_mut(&self.tangents, self.len),
                colors: convert_mut(&self.colors, self.len),
                uvs: convert_mut(&self.uvs, self.len)
            }
        }
    }

    // Set a specific attribute storage using a vector of a specific attribute
    pub fn overwrite<A: Attribute>(&mut self, vec: Vec<A::Item>) {
        // Make sure the lengths match
        assert!(vec.len() == self.len, "Length mismatch, cannot overwrite storage");
        let len = self.len;

        // Get the old attribute storage
        let ptr = &mut A::storage_mut(self).0;
        
        // Just in case
        let mut manual = ManuallyDrop::new(vec);

        // Deallocate the old storage if we can, and replace it with the new vector storage
        let layout = Layout::array::<A::Item>(len).unwrap();
        
        // Simply overwrite the pointer lol
        ptr.replace(NonNull::new(vec.as_mut_ptr()).unwrap());
    }

    // Check if a specific attribute is enabled or not
    pub fn enabled<A: Attribute>(&self) -> bool {
        A::storage(self).0.is_some()
    } 
}


// This allows us to read from the attributes all at the same time
struct RefAttributes<'a> {
    pub positions: Option<&'a [vek::Vec3<f32>]>,
    pub normals: Option<&'a [vek::Vec3<i8>]>,
    pub tangents: Option<&'a [vek::Vec4<i8>]>,
    pub colors: Option<&'a [vek::Rgb<u8>]>,
    pub uvs: Option<&'a [vek::Vec2<u8>]>,
}

// This is the same as RefAttributes, but it allows to also write to the attributes while we're at it
struct MutAttributes<'a> {
    pub positions: Option<&'a mut [vek::Vec3<f32>]>,
    pub normals: Option<&'a mut [vek::Vec3<i8>]>,
    pub tangents: Option<&'a mut [vek::Vec4<i8>]>,
    pub colors: Option<&'a mut [vek::Rgb<u8>]>,
    pub uvs: Option<&'a mut [vek::Vec2<u8>]>,
}