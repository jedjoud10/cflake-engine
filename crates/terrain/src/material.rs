use rendering::{
    AlbedoTexel, CameraUniform, DefaultMaterialResources, EnvironmentMap, MaskTexel, Material, MultiDrawIndirectCount, NormalTexel,
    Renderer, SceneUniform, ShadowMap, ShadowMapping, ShadowUniform, Pass,
};

use assets::Assets;

use graphics::{
    BindGroup, Compiler, FragmentModule, GpuPod, Graphics, LayeredTexture2D, Shader, StorageAccess, VertexModule, PrimitiveConfig, WindingOrder, PushConstantLayout, ModuleVisibility, XY, Vertex, DrawIndexedIndirect, DrawIndexedIndirectBuffer, Texture,
};
use utils::{Storage, Time};

use crate::{Terrain, TerrainSettings, PermVertices, PermTriangles, MemoryManager};

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
        world::Read<'w, Terrain>,
        world::Read<'w, Storage<PermVertices>>,
        world::Read<'w, Storage<PermTriangles>>,
        world::Read<'w, Storage<DrawIndexedIndirectBuffer>>,
        usize,
    );

    type Settings<'s> = (&'s TerrainSettings, &'s MemoryManager);
    type Query<'a> = &'a ();
    type RenderPath = MultiDrawIndirectCount;

    // Load the terrain material shaders and compile them
    fn shader<P: Pass>(terrain: &Self::Settings<'_>, graphics: &Graphics, assets: &Assets) -> Option<Shader> {
        let settings = terrain.0;
        let size = terrain.0.mesher.size;
        let memory = terrain.1;
        match P::pass_type() {
            rendering::PassType::Deferred => {
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

                // Define the types for the user textures
                if settings.rendering.submaterials.is_some() {
                    compiler.use_define("submaterials", "");
                    compiler.use_sampled_texture::<LayeredAlbedoMap>("layered_albedo_map", false);
                    compiler.use_sampled_texture::<LayeredNormalMap>("layered_normal_map", false);
                    compiler.use_sampled_texture::<LayeredMaskMap>("layered_mask_map", false);
                    compiler.use_sampler::<AlbedoTexel>("layered_albedo_map_sampler", false);
                    compiler.use_sampler::<NormalTexel>("layered_normal_map_sampler", false);
                    compiler.use_sampler::<MaskTexel>("layered_mask_map_sampler", false);
                }

                // Set the scaling factor for the vertex positions
                compiler.use_constant(0, (size as f32) / (size as f32 - 4.0));

                if settings.rendering.flat_colors {
                    compiler.use_define("flatcolors", "");
                }

                if settings.rendering.derived_normals {
                    compiler.use_define("derivednormals", "");
                }

                if settings.rendering.flat_normals {
                    compiler.use_define("flatnormals", "");
                }

                // Multi-draw indirect youpieee
                compiler.use_storage_buffer::<vek::Vec4<vek::Vec4<f32>>>(
                    "position_scale_buffer",
                  StorageAccess::ReadOnly,
                );

                compiler.use_storage_buffer::<<XY<f32> as Vertex>::Storage>(
                    "input_vertices",
                    StorageAccess::ReadOnly,
                );
                compiler.use_storage_buffer::<u32>("input_triangles", StorageAccess::ReadOnly);
                compiler.use_storage_buffer::<DrawIndexedIndirect>("indirect", StorageAccess::ReadOnly);
                compiler.use_constant(1, memory.output_vertex_buffer_length as u32);
                compiler.use_constant(2, memory.output_triangle_buffer_length as u32);

                // Compile the modules into a shader
                Some(Shader::new(vert, frag, &compiler).unwrap())
            },
            rendering::PassType::Shadow => {
                let vert = assets
                    .load::<VertexModule>("engine/shaders/scene/terrain/shadow.vert")
                    .unwrap();
                let frag = assets
                    .load::<FragmentModule>("engine/shaders/common/empty.frag")
                    .unwrap();

                
                // Define the type layouts for the UBOs
                let mut compiler = Compiler::new(assets, graphics);
                
                compiler.use_constant(0, (size as f32) / (size as f32 - 4.0));
                let layout = PushConstantLayout::vertex(<vek::Vec4<vek::Vec4<f32>> as GpuPod>::size()).unwrap();
                compiler.use_push_constant_layout(layout);
                compiler.use_storage_buffer::<vek::Vec4<vek::Vec4<f32>>>(
                    "position_scale_buffer",
                    StorageAccess::ReadOnly,
                );

                // Compile the modules into a shader
                Some(Shader::new(vert, frag, &compiler).unwrap())
            },
        }
    }

    // Terrain only needs packed positions
    fn attributes<P: Pass>() -> rendering::MeshAttributes {
        rendering::MeshAttributes::POSITIONS
    }

    // Disable frustum culling since we do that on the GPU
    fn cull<P: Pass>() -> bool {
        false
    }

    // Fetch the texture storages
    fn fetch<P: Pass>(world: &world::World) -> Self::Resources<'_> {
        let albedo_maps = world.get::<Storage<LayeredAlbedoMap>>().unwrap();
        let normal_maps = world.get::<Storage<LayeredNormalMap>>().unwrap();
        let mask_maps = world.get::<Storage<LayeredMaskMap>>().unwrap();
        let vertices = world.get::<Storage<PermVertices>>().unwrap();
        let triangles = world.get::<Storage<PermTriangles>>().unwrap();
        let indirect = world.get::<Storage<DrawIndexedIndirectBuffer>>().unwrap();
        let terrain = world.get::<Terrain>().unwrap();
        (
            albedo_maps,
            normal_maps,
            mask_maps,
            terrain,
            vertices,
            triangles,
            indirect,
            0,
        )
    }

    // Set the static bindings that will never change
    fn set_global_bindings<'r, P: Pass>(
        resources: &'r mut Self::Resources<'_>,
        group: &mut BindGroup<'r>,
        default: &DefaultMaterialResources<'r>,
    ) {
        let (albedo_maps,
            normal_maps,
            mask_maps,
            terrain,
            ..) = resources;

        // Set the sub-material textures
        if P::is_deferred_pass() {
            // Set the required common buffers
            group
                .set_uniform_buffer("camera", default.camera_buffer, ..)
                .unwrap();

            if let (Some(albedo), Some(normal), Some(mask)) = (
                &terrain.manager.layered_albedo_map,
                &terrain.manager.layered_normal_map,
                &terrain.manager.layered_mask_map,
            ) {
                // Get the layered textures, without any fallback
                let albedo_map = albedo_maps.get(albedo);
                let normal_map = normal_maps.get(normal);
                let mask_map = mask_maps.get(mask);

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

                // Set the material texture samplers
                group
                    .set_sampler("layered_albedo_map_sampler", albedo_map.sampler().unwrap())
                    .unwrap();
                group
                    .set_sampler("layered_normal_map_sampler", normal_map.sampler().unwrap())
                    .unwrap();
                group
                    .set_sampler("layered_mask_map_sampler", mask_map.sampler().unwrap())
                    .unwrap();
            }
        }
    }

    // Set the per-surface bindings for the material
    // Since the terrain mesh only contains "allocation" count of sub-surfaces, this will be executed for each allocation
    fn set_surface_bindings<'r, 'w, P: Pass>(
        _renderer: &Renderer,
        resources: &'r mut Self::Resources<'w>,
        _default: &DefaultMaterialResources<'w>,
        _query: &Self::Query<'w>,
        group: &mut BindGroup<'r>,
    ) {
        let (.., terrain, vertices, triangles, indirect, index) = resources;

        // Set the storage buffer that contains ALL the matrices
        group
            .set_storage_buffer(
                "position_scale_buffer",
                &terrain.memory.culled_position_scaling_buffers[*index],
                ..,
            )
            .unwrap();

        // Pass vertex and triangles as storage buffers
        if P::is_deferred_pass() {
            let triangles = triangles.get(&terrain.memory.shared_triangle_buffers[*index]);
            let vertices = vertices.get(&terrain.memory.shared_positions_buffers[*index]);
            let indirect = indirect.get(&terrain.memory.culled_indexed_indirect_buffers[*index]);
            
            group
                .set_storage_buffer(
                    "input_triangles",
                    &triangles,
                    ..,
                )
                .unwrap();

            group
                .set_storage_buffer(
                    "input_vertices",
                    &vertices,
                    ..,
                )
                .unwrap();

            group
                .set_storage_buffer(
                    "indirect",
                    &indirect,
                    ..,
                )
                .unwrap();
        }

        // Increment the index (aka the allocation index)
        *index += 1;
    }

    // Only used for setting the shadow lightspace matrix
    fn set_push_constants<'r, 'w, P: Pass>(
        &self,
        _renderer: &Renderer,
        _resources: &'r mut Self::Resources<'w>,
        _default: &DefaultMaterialResources<'r>,
        _query: &Self::Query<'w>,
        constants: &mut graphics::PushConstants<graphics::ActiveRenderPipeline<P::C, P::DS>>,
    ) {
        if P::is_shadow_pass() {
            let lightspace = _default.lightspace.unwrap();
            let bytes = GpuPod::into_bytes(&lightspace.cols);
            constants.push(bytes, 0, ModuleVisibility::Vertex).unwrap();
        }
    }
}
