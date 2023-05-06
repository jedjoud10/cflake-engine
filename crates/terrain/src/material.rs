use rendering::{
    ActiveScenePipeline, AlbedoTexel, CameraUniform, DefaultMaterialResources, Indirect, MaskTexel,
    Material, NormalTexel, Renderer, SceneUniform, ShadowMap, ShadowMapping, ShadowUniform, MultiDrawIndirect,
};

use assets::Assets;

use graphics::{
    BindGroup, Compiler, FragmentModule, GpuPod, Graphics, LayeredTexture2D, ModuleVisibility,
    PrimitiveConfig, PushConstantLayout, PushConstants, Shader, StorageAccess, VertexModule,
    WindingOrder,
};
use utils::{Handle, Storage, Time};

use crate::{TerrainSettings, Terrain};

// Type aliases for layered textures
pub type LayeredAlbedoMap = LayeredTexture2D<AlbedoTexel>;
pub type LayeredNormalMap = LayeredTexture2D<NormalTexel>;
pub type LayeredMaskMap = LayeredTexture2D<MaskTexel>;

// Terrain shader that contains physically based lighting, but suited for terrain rendering
// Contains multiple Layered2D textures for each PBR parameters
// Currently, there is no blending that is occuring between different terrain sub-materials
// The actual terrain material data is stored within the terrain because there can only be ONE terrain material per world
pub struct TerrainMaterial;

impl Material for TerrainMaterial {
    type Resources<'w> = (
        world::Read<'w, Storage<LayeredAlbedoMap>>,
        world::Read<'w, Storage<LayeredNormalMap>>,
        world::Read<'w, Storage<LayeredMaskMap>>,
        world::Read<'w, ShadowMapping>,
        world::Read<'w, Terrain>,
        world::Read<'w, Time>,
        usize,
    );

    type Settings<'s> = &'s TerrainSettings;
    type Query<'a> = &'a ();
    type RenderPath = MultiDrawIndirect;

    // Load the terrain material shaders and compile them
    fn shader(settings: &Self::Settings<'_>, graphics: &Graphics, assets: &Assets) -> Shader {
        // Load the vertex module from the assets
        let vert = assets
            .load::<VertexModule>("engine/shaders/scene/terrain/terrain.vert")
            .unwrap();

        // Load the fragment module from the assets
        let frag = assets
            .load::<FragmentModule>("engine/shaders/scene/terrain/terrain.frag")
            .unwrap();

        // Define the type layouts for the UBOs
        let mut compiler = Compiler::new(assets, graphics);

        // Set the UBO types that we will use
        compiler.use_uniform_buffer::<CameraUniform>("camera");
        compiler.use_uniform_buffer::<SceneUniform>("scene");

        // Define the types for the user textures
        if settings.sub_materials.is_some() {
            compiler.use_define("submaterials", "");
            compiler.use_sampled_texture::<LayeredAlbedoMap>("layered_albedo_map");
            compiler.use_sampled_texture::<LayeredNormalMap>("layered_normal_map");
            compiler.use_sampled_texture::<LayeredMaskMap>("layered_mask_map");
        }

        // Shadow parameters
        compiler.use_uniform_buffer::<ShadowUniform>("shadow_parameters");
        compiler.use_uniform_buffer::<vek::Vec4<vek::Vec4<f32>>>("shadow_lightspace_matrices");
        compiler.use_uniform_buffer::<f32>("cascade_plane_distances");

        // Define the types for the user textures
        compiler.use_sampled_texture::<ShadowMap>("shadow_map");

        // Set the scaling factor for the vertex positions
        compiler.use_constant(0, (settings.size as f32) / (settings.size as f32 - 3.0));

        // Define the "lowpoly" macro
        if settings.lowpoly {
            compiler.use_define("lowpoly", "");
        }

        // Multi-draw indirect youpieee
        compiler.use_storage_buffer::<vek::Vec4<vek::Vec4<f32>>>("position_scale_buffer", StorageAccess::ReadOnly);

        // Compile the modules into a shader
        Shader::new(vert, frag, &compiler).unwrap()
    }

    // Terrain only needs tex coordinates (packed)
    fn attributes() -> rendering::MeshAttributes {
        rendering::MeshAttributes::TEX_COORDS
    }

    // Custom shadow mapper (due to packed tex coordinates)
    fn casts_shadows() -> rendering::CastShadowsMode<Self> {
        rendering::CastShadowsMode::Disabled
        /*
        let callback: Box<dyn FnOnce(&Self::Settings<'_>, &Graphics, &Assets) -> Shader> =
            Box::new(|settings, graphics, assets| {
                // Load the modified vertex module for the shadowmap shader
                let vertex = assets
                    .load::<VertexModule>("engine/shaders/scene/shadow/terrain.vert")
                    .unwrap();

                // Load the fragment module for the shadowmap shader
                let fragment = assets
                    .load::<FragmentModule>("engine/shaders/scene/shadow/shadow.frag")
                    .unwrap();

                // Create the bind layout for the shadow map shader
                let mut compiler = Compiler::new(assets, graphics);

                // Set the scaling factor for the vertex positions
                compiler.use_constant(0, (settings.size as f32) / (settings.size as f32 - 3.0));

                // Contains the mesh matrix and the lightspace uniforms
                let layout = PushConstantLayout::single(
                    <vek::Vec4<vek::Vec4<f32>> as GpuPod>::size() * 2,
                    ModuleVisibility::Vertex,
                )
                .unwrap();
                compiler.use_push_constant_layout(layout);

                // Combine the modules to the shader
                Shader::new(vertex, fragment, compiler).unwrap()
            });

        rendering::CastShadowsMode::Enabled(Some(callback))
        */
    }

    // Fetch the texture storages
    fn fetch<'w>(world: &'w world::World) -> Self::Resources<'w> {
        let albedo_maps = world.get::<Storage<LayeredAlbedoMap>>().unwrap();
        let normal_maps = world.get::<Storage<LayeredNormalMap>>().unwrap();
        let mask_maps = world.get::<Storage<LayeredMaskMap>>().unwrap();
        let shadow = world.get::<ShadowMapping>().unwrap();
        let time: world::Read<Time> = world.get::<Time>().unwrap();
        let terrain = world.get::<Terrain>().unwrap();
        (albedo_maps, normal_maps, mask_maps, shadow, terrain, time, 0)
    }

    // Set the static bindings that will never change
    fn set_global_bindings<'r, 'w>(
        resources: &'r mut Self::Resources<'w>,
        group: &mut BindGroup<'r>,
        default: &DefaultMaterialResources<'r>,
    ) {
        let (
            albedo_maps,
            normal_maps,
            mask_maps,
            shadow,
            terrain,
            time,
            _
        ) = resources;

        // Set the required common buffers
        group
            .set_uniform_buffer("camera", default.camera_buffer, ..)
            .unwrap();
        group
            .set_uniform_buffer("scene", default.scene_buffer, ..)
            .unwrap();
        group
            .set_uniform_buffer("shadow_parameters", &shadow.parameter_buffer, ..)
            .unwrap();
        group
            .set_uniform_buffer(
                "shadow_lightspace_matrices",
                &shadow.lightspace_buffer,
                ..,
            )
            .unwrap();
        group
            .set_uniform_buffer(
                "cascade_plane_distances",
                &shadow.cascade_distances,
                ..,
            )
            .unwrap();

        // Set the scene shadow map
        group
            .set_sampled_texture("shadow_map", &shadow.depth_tex)
            .unwrap();

        if let (Some(albedo), Some(normal), Some(mask)) = (
                &terrain.manager.layered_albedo_map,
                &terrain.manager.layered_normal_map,
                &terrain.manager.layered_mask_map,
            ) {
                // Get the layered textures, without any fallback
                let albedo_map = albedo_maps.get(&albedo);
                let normal_map = normal_maps.get(&normal);
                let mask_map = mask_maps.get(&mask);
    
                // Set the material textures
                group
                    .set_sampled_texture("layered_albedo_map", albedo_map)
                    .unwrap();
                group
                    .set_sampled_texture("layered_normal_map", normal_map)
                    .unwrap();
                group
                    .set_sampled_texture("layered_mask_map", mask_map)
                    .unwrap();
            }
    }

    // Set the surface bindings that only contain the allocation data
    // This will be executed only 7-8 times per frame since we have few big allocations/subsurfaces
    fn set_surface_bindings<'r, 'w>(
        _renderer: &Renderer,
        resources: &'r mut Self::Resources<'w>,
        _default: &mut DefaultMaterialResources<'w>,
        _query: &Self::Query<'w>,
        group: &mut BindGroup<'r>,
    ) {
        let (
            albedo_maps,
            normal_maps,
            mask_maps,
            shadow,
            terrain,
            time,
            i,
        ) = resources;

        // Set the storage buffer that contains ALL the matrices
        group.set_storage_buffer(
            "position_scale_buffer",
            &terrain.manager.position_scaling_buffers[*i],
            ..
        ).unwrap();   

        // Since the ordering of entities within an archetype is always deterministic we can do this without worrying
        resources.6 += 1;
    }
}
