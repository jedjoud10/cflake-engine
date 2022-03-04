use super::{material::Material, mesh::Mesh, uniforms::SetUniformsCallback};
use bitflags::bitflags;
bitflags! {
    pub struct RendererFlags: u8 {
        const VISIBLE = 0b00000001;
        const SHADOW_CASTER = 0b00000010;
        const SHOULD_DELETE_MESH = 0b00000100;
        const DEFAULT = Self::VISIBLE.bits | Self::SHADOW_CASTER.bits | Self::SHOULD_DELETE_MESH.bits;
    }
}

// A render unit that will render a single mesh with a material
pub struct RenderUnit {
    pub mesh: MeshKey,
    pub material: MaterialKey,
    pub matrix: veclib::Matrix4x4<f32>,
    pub flags: RendererFlags,
    pub uniforms: SetUniformsCallback,
}

impl PipelineObject for Renderer {
    // Reserve an ID for this renderer
    fn reserve(self, pipeline: &Pipeline) -> Option<(Self, ObjectID<Self>)> {
        Some((self, pipeline.renderers.gen_id()))
    }
    // Send this rendererer to the pipeline for construction
    fn send(self, id: ObjectID<Self>) -> ConstructionTask {
        ConstructionTask::Renderer(Construct::<Self>(self, id))
    }
    // Create a deconstruction task
    fn pull(id: ObjectID<Self>) -> DeconstructionTask {
        DeconstructionTask::Renderer(Deconstruct::<Self>(id))
    }
    // Add the renderer to our ordered vec
    fn add(self, pipeline: &mut Pipeline, id: ObjectID<Self>) -> Option<()> {
        // Add the renderer
        pipeline.renderers.insert(id, self)?;
        Some(())
    }
    // Delete the renderer from the pipeline
    fn delete(pipeline: &mut Pipeline, id: ObjectID<Self>) -> Option<Self> {
        let me = pipeline.renderers.remove(id)?;
        // Also remove the mesh if we want to
        if me.flags.contains(RendererFlags::SHOULD_DELETE_MESH) {
            let _removed_mesh = Mesh::delete(pipeline, me.mesh)?;
        }
        Some(me)
    }
}

// Everything related to the creation of a renderer
impl Renderer {
    // Set a mesh
    pub fn with_mesh(mut self, mesh: ObjectID<Mesh>) -> Self {
        self.mesh = mesh;
        self
    }
    // With a specific material
    pub fn with_material(mut self, material: ObjectID<Material>) -> Self {
        self.material = material;
        self
    }
    // Set the mesh matrix for this renderer
    pub fn with_matrix(mut self, matrix: veclib::Matrix4x4<f32>) -> Self {
        self.matrix = matrix;
        self
    }
    // Set a uniform callback for this renderer (This is not ideal, but it's better than the last method)
    pub fn with_uniform(mut self, callback: SetUniformsCallback) -> Self {
        self.uniforms = callback;
        self
    }
}
