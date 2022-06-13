use super::{
    attributes::{output as out, Attribute},
    VertexLayout,
};

// A vertex assembly is just a collection of multiple vertices that are stored on the CPU
#[derive(Default)]
pub struct VertexAssembly {
    // Rust vectors of vertex attributes
    pub(super) positions: Option<Vec<out::VePos>>,
    pub(super) normals: Option<Vec<out::VeNormal>>,
    pub(super) tangents: Option<Vec<out::VeTangent>>,
    pub(super) colors: Option<Vec<out::VeColor>>,
    pub(super) tex_coord_0: Option<Vec<out::VeTexCoord0>>,

    // The vertex attributes that are enabled
    layout: VertexLayout,
}

impl VertexAssembly {
    // Insert an attribute vector into the assembly
    pub fn insert<U: Attribute>(&mut self, vec: Vec<U::Out>) {
        U::insert(self, vec);
        self.layout.insert(U::LAYOUT);
    }

    // Get an attribute vector immutably
    pub fn get<U: Attribute>(&self) -> Option<&Vec<U::Out>> {
        U::get_from_assembly(self)
    }

    // Get an attribute vector mutably
    pub fn get_mut<U: Attribute>(&mut self) -> Option<&mut Vec<U::Out>> {
        U::get_from_assembly_mut(self)
    }

    // Get the layout that we have created
    pub fn layout(&self) -> VertexLayout {
        self.layout
    }

    // Get the number of vertices that we have in total (this will return None if one or more vectors have different lengths)
    pub fn len(&self) -> Option<usize> {
        // This function just takes an Option<Vec<T>> and returns an Option<usize>
        fn len<T>(vec: &Option<Vec<T>>) -> Option<usize> {
            vec.as_ref().map(Vec::len)
        }

        // Make sure all the lengths (that are valid) be equal to each other
        let arr = [
            len(&self.positions),
            len(&self.normals),
            len(&self.tangents),
            len(&self.colors),
            len(&self.tex_coord_0),
        ];
        let first = arr.iter().find(|opt| opt.is_some()).cloned().flatten()?;

        // Iterate and check
        let valid = arr.into_iter().flatten().all(|len| len == first);

        // Trollinnggggg
        valid.then(|| first)
    }
}

// Assembly that stores the indices that we will use to conenct each vertex to each other
pub type IndexAssembly = Vec<u32>;
