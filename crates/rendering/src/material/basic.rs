use std::any::TypeId;

use crate::{Material, EnabledMeshAttributes};
use ahash::AHashMap;
use assets::Assets;
use graphics::{
    Compiled, FragmentModule, Graphics, Normalized,
    Texture2D, VertexModule, RGB, Compiler, Sampler, ModuleKind, BindingConfig, ModuleBindingConfig,
};
use utils::{Storage, Handle};

// Basic type aliases
type AlbedoTexel = RGB<Normalized<u8>>;
type NormalTexel = RGB<Normalized<i8>>;
type AlbedoMap = Texture2D<AlbedoTexel>;
type NormalMap = Texture2D<NormalTexel>;

// A basic forward rendering material that will read from a diffuse map and normal map
// This does not implement the PBR workflow, and it's only used for simplicity at first
pub struct Basic {
    // Textures used by this basic material
    pub diffuse_map: Option<Handle<AlbedoMap>>,
    pub normal_map: Option<Handle<NormalMap>>,

    // Simple Basic Parameters
    pub roughness: f32,
    pub tint: vek::Rgb<f32>, 
}

impl Material for Basic {
    type Resources<'w> = world::Read<'w, Storage<Box<u32>>>;

    // Load the vertex shader for this material
    fn vertex(
        graphics: &Graphics,
        assets: &Assets,
    ) -> Compiled<VertexModule> {
        let vert = assets
            .load::<VertexModule>("engine/shaders/basic.vert")
            .unwrap();
        Compiler::new(vert).compile(assets, graphics).unwrap()
    }

    // Load the fragment shader for this material
    fn fragment(
        graphics: &Graphics,
        assets: &Assets,
    ) -> Compiled<FragmentModule> {
        let frag = assets
            .load::<FragmentModule>("engine/shaders/basic.frag")
            .unwrap();
        Compiler::new(frag).compile(assets, graphics).unwrap()
    }

    fn attributes() -> EnabledMeshAttributes {
        //EnabledMeshAttributes::POSITIONS | EnabledMeshAttributes::NORMALS | EnabledMeshAttributes::TEX_COORDS
        EnabledMeshAttributes::empty()
        //EnabledMeshAttributes::POSITIONS 
    }

    fn binding_config() -> BindingConfig {
        let module_binding_config = ModuleBindingConfig {
            push_constant: Some((graphics::PushConstantBlock {
                name: "MeshConstants".to_string(),
                variables: AHashMap::from_iter([(("test".to_string(), 
                    graphics::BlockVariable::Unit {
                        name: "test".to_string(),
                        size: 4,
                        offset: 0,
                        _type: graphics::VariableType::Float { size: 32 }
                    }
                )), ("test2".to_string(), 
                    graphics::BlockVariable::Unit {
                        name: "test2".to_string(),
                        size: 4,
                        offset: 4,
                        _type: graphics::VariableType::Int { size: 32, signed: false }
                })]),
                size: 4,
                offset: 0,
            }, TypeId::of::<u32>())),
        };

        BindingConfig::from_modules([(ModuleKind::Vertex, module_binding_config)])
    }

    fn fetch<'w>(world: &'w world::World) -> Self::Resources<'w> {
        let storage = world.get::<Storage<Box<u32>>>().unwrap();
        storage
    }
}
