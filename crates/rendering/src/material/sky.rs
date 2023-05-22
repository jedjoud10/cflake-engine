use crate::{
    AlbedoMap, CameraUniform, DefaultMaterialResources, Direct, EnvironmentMap,
    Material, MeshAttributes, SceneUniform,
};

use assets::Assets;
use graphics::{
    BindGroup, Compiler, Face, FragmentModule, Graphics, PrimitiveConfig, Shader, VertexModule,
    WindingOrder,
};
use utils::Storage;

// A very simple sky material which uses a procedural sky system
// This is private since the user will NEVER create a sky entity and at it manually (it's added automatically)
pub(crate) struct SkyMaterial {}

impl Material for SkyMaterial {
    type Resources<'w> = world::Read<'w, Storage<AlbedoMap>>;
    type RenderPath = Direct;
    type Settings<'s> = ();
    type Query<'a> = &'a ();

    // Load the respective Sky shader modules and compile them
    fn shader(_settings: &Self::Settings<'_>, graphics: &Graphics, assets: &Assets) -> Shader {
        // Load the vertex module from the assets
        let vert = assets
            .load::<VertexModule>("engine/shaders/scene/sky/sky.vert")
            .unwrap();

        // Load the fragment module from the assets
        let frag = assets
            .load::<FragmentModule>("engine/shaders/scene/sky/sky.frag")
            .unwrap();

        // Define the type layouts for the UBOs
        let mut compiler = Compiler::new(assets, graphics);
        compiler.use_uniform_buffer::<CameraUniform>("camera");
        compiler.use_uniform_buffer::<SceneUniform>("scene");
        compiler.use_sampled_texture::<EnvironmentMap>("environment_map");

        // Compile the modules into a shader
        Shader::new(vert, frag, &compiler).unwrap()
    }

    // Get the required mesh attributes that we need to render a surface
    fn attributes() -> MeshAttributes {
        MeshAttributes::POSITIONS
    }

    // The sky does NOT cast shadows
    fn casts_shadows() -> bool {
        false
    }

    // The sky does NOT use frustum culling
    fn frustum_culling() -> bool {
        false
    }

    // Sky-spheres are always flipped inside out
    fn primitive_config() -> PrimitiveConfig {
        PrimitiveConfig::Triangles {
            winding_order: WindingOrder::Ccw,
            cull_face: Some(Face::Front),
            wireframe: false,
        }
    }

    // Fetch the texture storages
    fn fetch(world: &world::World) -> Self::Resources<'_> {
        let albedo_maps = world.get::<Storage<AlbedoMap>>().unwrap();
        albedo_maps
    }

    // Set the static bindings that will never change
    fn set_global_bindings<'r>(
        _resources: &'r mut Self::Resources<'_>,
        group: &mut BindGroup<'r>,
        default: &DefaultMaterialResources<'r>,
    ) {
        group
            .set_uniform_buffer("camera", default.camera_buffer, ..)
            .unwrap();
        group
            .set_uniform_buffer("scene", default.scene_buffer, ..)
            .unwrap();
        group
            .set_sampled_texture("environment_map", default.environment_map)
            .unwrap();
    }
}
