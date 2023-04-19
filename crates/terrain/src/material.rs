use rendering::{
    CameraUniform, DefaultMaterialResources, Indirect,
    Material, Renderer, SceneUniform, ShadowMap,
    ShadowMapping, ShadowUniform, ActiveScenePipeline, AlbedoTexel, NormalTexel, MaskTexel,
};

use assets::Assets;

use graphics::{
    BindGroup, Compiler, FragmentModule, GpuPod, Graphics,
    ModuleVisibility, PrimitiveConfig, PushConstantLayout,
    PushConstants, Shader, VertexModule, WindingOrder, StorageAccess, LayeredTexture2D,
};
use utils::{Time, Handle, Storage};

// Type aliases for layered textures
pub type LayeredAlbedoMap = LayeredTexture2D<AlbedoTexel>;
pub type LayeredNormalMap = LayeredTexture2D<NormalTexel>;
pub type LayeredMaskMap = LayeredTexture2D<MaskTexel>;


// Terrain shader that contains physically based lighting, but suited for terrain rendering
// Contains multiple Layered2D textures for each PBR parameters
// Currently, there is no blending that is occuring between different terrain sub-materials
pub struct TerrainMaterial {
    // Layered textures and their material index 
    pub layered_albedo_map: Handle<LayeredAlbedoMap>,
    pub layered_normal_map: Handle<LayeredNormalMap>,
    pub layered_mask_map: Handle<LayeredMaskMap>,

    // PBR Parameters
    pub bumpiness: f32,
    pub roughness: f32,
    pub metallic: f32,
    pub ambient_occlusion: f32,
}

impl Material for TerrainMaterial {
    type Resources<'w> = (
        world::Read<'w, Storage<LayeredAlbedoMap>>,
        world::Read<'w, Storage<LayeredNormalMap>>,
        world::Read<'w, Storage<LayeredMaskMap>>,
        world::Read<'w, ShadowMapping>,
        world::Read<'w, Time>,
    );

    // Load the terrain material shaders and compile them
    fn shader(graphics: &Graphics, assets: &Assets) -> Shader {
        // Load the vertex module from the assets
        let vert = assets
            .load::<VertexModule>(
                "engine/shaders/scene/terrain/terrain.vert",
            )
            .unwrap();

        // Load the fragment module from the assets
        let frag = assets
            .load::<FragmentModule>(
                "engine/shaders/scene/terrain/terrain.frag",
            )
            .unwrap();

        // Define the type layouts for the UBOs
        let mut compiler = Compiler::new(assets, graphics);

        // Set the UBO types that we will use
        compiler.use_uniform_buffer::<CameraUniform>("camera");
        compiler.use_uniform_buffer::<SceneUniform>("scene");

        // Define the types for the user textures
        compiler.use_sampled_texture::<LayeredAlbedoMap>("layered_albedo_map");
        compiler.use_sampled_texture::<LayeredNormalMap>("layered_normal_map");
        compiler.use_sampled_texture::<LayeredMaskMap>("layered_mask_map");

        // Shadow parameters
        compiler.use_uniform_buffer::<ShadowUniform>("shadow_parameters");
        compiler.use_uniform_buffer::<vek::Vec4<vek::Vec4<f32>>>("shadow_lightspace_matrices");
        compiler.use_uniform_buffer::<f32>("cascade_plane_distances");
    
        // Define the types for the user textures
        compiler.use_sampled_texture::<ShadowMap>("shadow_map");

        // Define the push ranges used by push constants
        compiler.use_push_constant_layout(
            PushConstantLayout::split(
                <vek::Vec4<vek::Vec4<f32>> as GpuPod>::size(),
                <vek::Rgba<f32> as GpuPod>::size() + f32::size(),
            )
            .unwrap(),
        );

        // Compile the modules into a shader
        Shader::new(vert, frag, compiler).unwrap()
    }

    // Terrain only needs positions and normals
    fn attributes() -> rendering::MeshAttributes {
        rendering::MeshAttributes::POSITIONS
    }

    fn primitive_config() -> PrimitiveConfig {
        PrimitiveConfig::Triangles {
            winding_order: WindingOrder::Cw,
            cull_face: Some(graphics::Face::Front),
            wireframe: false,
        }
    }

    // Fetch the texture storages
    fn fetch<'w>(world: &'w world::World) -> Self::Resources<'w> {
        let albedo_maps = world.get::<Storage<LayeredAlbedoMap>>().unwrap();
        let normal_maps = world.get::<Storage<LayeredNormalMap>>().unwrap();
        let mask_maps = world.get::<Storage<LayeredMaskMap>>().unwrap();
        let shadow = world.get::<ShadowMapping>().unwrap();
        let time = world.get::<Time>().unwrap();
        (albedo_maps, normal_maps, mask_maps, shadow, time)
    }

    // Set the static bindings that will never change
    fn set_global_bindings<'r, 'w>(
        resources: &'r mut Self::Resources<'w>,
        group: &mut BindGroup<'r>,
        default: &DefaultMaterialResources<'r>,
    ) {
        // Set the required common buffers
        group
            .set_uniform_buffer("camera", default.camera_buffer, ..)
            .unwrap();
        group
            .set_uniform_buffer("scene", default.scene_buffer, ..)
            .unwrap();
        group
            .set_uniform_buffer("shadow_parameters", &resources.3.parameter_buffer, ..)
            .unwrap();
        group
            .set_uniform_buffer("shadow_lightspace_matrices", &resources.3.lightspace_buffer, ..)
            .unwrap();
        group
            .set_uniform_buffer("cascade_plane_distances", &resources.3.cascade_distances, ..)
            .unwrap();


        // Set the scene shadow map
        group
            .set_sampled_texture("shadow_map", &resources.3.depth_tex)
            .unwrap();
    }

    // Set the instance bindings that will change per material
    fn set_instance_bindings<'r, 'w>(
        &self,
        resources: &'r mut Self::Resources<'w>,
        default: &DefaultMaterialResources<'r>,
        group: &mut BindGroup<'r>,
    ) {
        let (albedo_maps, normal_maps, mask_maps, _, _) = resources;

        // Get the layered textures, without any fallback
        let albedo_map = albedo_maps.get(&self.layered_albedo_map);
        let normal_map = normal_maps.get(&self.layered_normal_map);
        let mask_map = mask_maps.get(&self.layered_mask_map);

        // Set the material textures
        group.set_sampled_texture("layered_albedo_map", albedo_map).unwrap();
        group.set_sampled_texture("layered_normal_map", normal_map).unwrap();
        group.set_sampled_texture("layered_mask_map", mask_map).unwrap();
    }

    // Set the surface push constants
    fn set_push_constants<'r, 'w>(
        &self,
        renderer: &Renderer,
        resources: &'r mut Self::Resources<'w>,
        _default: &DefaultMaterialResources<'r>,
        constants: &mut PushConstants<ActiveScenePipeline>,
    ) {
        // Send the raw vertex bytes to the GPU
        let matrix = renderer.matrix;
        let cols = matrix.cols;
        let bytes = GpuPod::into_bytes(&cols);
        constants.push(bytes, 0, ModuleVisibility::Vertex).unwrap();

        // Convert the material parameters into a vec4
        let vector = vek::Vec4::new(
            self.bumpiness,
            self.metallic,
            self.ambient_occlusion,
            self.roughness,
        );

        // Send the raw fragment bytes to the GPU
        let bytes = GpuPod::into_bytes(&vector);
        constants
            .push(bytes, 0, ModuleVisibility::Fragment)
            .unwrap();

        // Calculate "fade" effect
        let duration =  resources.4.frame_start().saturating_duration_since(renderer.instant_initialized.unwrap());
        let fade = duration.as_secs_f32().clamp(0.0, 10.0);

        // Upload the fade effect to GPU
        let bytes2 = GpuPod::into_bytes(&fade);
        constants
            .push(bytes2, bytes.len() as u32, ModuleVisibility::Fragment)
            .unwrap();
    }

    type RenderPath = Indirect;
}
