use crate::{
    AlbedoMap, CameraUniform, DefaultMaterialResources, Direct,
    EnvironmentMap, MaskMap, Material, NormalMap, Renderer, SceneUniform, ShadowMap, ShadowMapping,
    ShadowUniform, Pass,
};

use assets::Assets;

use graphics::{
    BindGroup, Compiler, FragmentModule, GpuPod, Graphics, ModuleVisibility, PushConstantLayout,
    PushConstants, Shader, VertexModule, ActiveRenderPipeline,
};
use utils::{Handle, Storage};

// A PBR shader that will try to fake how real light works in the real world
pub struct PbrMaterial {
    // Textures used by the PBR workflow material
    pub albedo_map: Option<Handle<AlbedoMap>>,
    pub normal_map: Option<Handle<NormalMap>>,
    pub mask_map: Option<Handle<MaskMap>>,

    // PBR Parameters
    pub bumpiness_factor: f32,
    pub roughness_factor: f32,
    pub metallic_factor: f32,
    pub ambient_occlusion_factor: f32,
    pub scale: vek::Extent2<f32>,
    pub tint: vek::Rgb<f32>,
}

impl Material for PbrMaterial {
    type Resources<'w> = (
        world::Read<'w, Storage<AlbedoMap>>,
        world::Read<'w, Storage<NormalMap>>,
        world::Read<'w, Storage<MaskMap>>,
    );

    type RenderPath = Direct;
    type Settings<'s> = ();
    type Query<'a> = &'a ();

    // Load the respective PBR shader modules and compile them
    fn shader<P: Pass>(_settings: &Self::Settings<'_>, graphics: &Graphics, assets: &Assets) -> Option<Shader> {
        match P::pass_type() {
            crate::PassType::Deferred => {
                // Load the vertex module from the assets
                let vert = assets
                    .load::<VertexModule>("engine/shaders/scene/pbr/pbr.vert")
                    .unwrap();

                // Load the fragment module from the assets
                let frag = assets
                    .load::<FragmentModule>("engine/shaders/scene/pbr/pbr.frag")
                    .unwrap();

                // Define the type layouts for the UBOs
                let mut compiler = Compiler::new(assets, graphics);

                // Set the UBO types that we will use
                compiler.use_uniform_buffer::<CameraUniform>("camera");

                // Define the types for the user textures
                compiler.use_sampled_texture::<AlbedoMap>("albedo_map", false);
                compiler.use_sampled_texture::<NormalMap>("normal_map", false);
                compiler.use_sampled_texture::<MaskMap>("mask_map", false);

                // Define the push ranges used by push constants
                compiler.use_push_constant_layout(
                    PushConstantLayout::split(
                        <vek::Vec4<vek::Vec4<f32>> as GpuPod>::size(),
                        <vek::Rgba<f32> as GpuPod>::size() * 2 + <vek::Extent2<f32> as GpuPod>::size(),
                    )
                    .unwrap(),
                );
            
                // Compile the modules into a shader
                Some(Shader::new(vert, frag, &compiler).unwrap())
            },
            crate::PassType::Shadow => {
                let vert = assets
                    .load::<VertexModule>("engine/shaders/scene/pbr/shadow.vert")
                    .unwrap();
                let frag = assets
                    .load::<FragmentModule>("engine/shaders/common/empty.frag")
                    .unwrap();

                let mut compiler = Compiler::new(assets, graphics);
                let layout = PushConstantLayout::vertex(<vek::Vec4<vek::Vec4<f32>> as GpuPod>::size() * 2).unwrap();
                compiler.use_push_constant_layout(layout);
            
                // Compile the modules into a shader
                Some(Shader::new(vert, frag, &compiler).unwrap())
            },
        }

        
    }

    // Fetch the texture storages
    fn fetch<P: Pass>(world: &world::World) -> Self::Resources<'_> {
        let albedo_maps = world.get::<Storage<AlbedoMap>>().unwrap();
        let normal_maps = world.get::<Storage<NormalMap>>().unwrap();
        let mask_maps = world.get::<Storage<MaskMap>>().unwrap();
        (albedo_maps, normal_maps, mask_maps)
    }

    // Set the static bindings that will never change
    fn set_global_bindings<'r, P: Pass>(
        resources: &'r mut Self::Resources<'_>,
        group: &mut BindGroup<'r>,
        default: &DefaultMaterialResources<'r>,
    ) {
        group
            .set_uniform_buffer("camera", default.camera_buffer, ..)
            .unwrap();
    }

    // Set the instance bindings that will change per material
    fn set_instance_bindings<'r, P: Pass>(
        &self,
        resources: &'r mut Self::Resources<'_>,
        default: &DefaultMaterialResources<'r>,
        group: &mut BindGroup<'r>,
    ) {
        // Only required for the deferred pass
        if !P::is_deferred_pass() {
            return;
        }

        let (albedo_maps, normal_maps, mask_maps) = resources;

        // Get the albedo texture, and fallback to a white one
        let albedo_map = self
            .albedo_map
            .as_ref()
            .map_or(default.white, |h| albedo_maps.get(h));

        // Get the normal map, and fallback to the default one
        let normal_map = self
            .normal_map
            .as_ref()
            .map_or(default.normal, |h| normal_maps.get(h));

        // Get the mask map, and fallback to the default one
        let mask_map = self
            .mask_map
            .as_ref()
            .map_or(default.mask, |h| mask_maps.get(h));

        // Set the material textures
        group.set_sampled_texture("albedo_map", albedo_map).unwrap();
        group.set_sampled_texture("normal_map", normal_map).unwrap();
        group.set_sampled_texture("mask_map", mask_map).unwrap();
    }

    // Set the surface push constants
    fn set_push_constants<'r, 'w, P: Pass>(
        &self,
        renderer: &Renderer,
        _resources: &'r mut Self::Resources<'w>,
        _default: &DefaultMaterialResources<'r>,
        _query: &Self::Query<'w>,
        constants: &mut PushConstants<ActiveRenderPipeline<P::C, P::DS>>,
    ) {
        // Send the raw vertex bytes to the GPU
        let matrix = renderer.matrix;
        let cols = matrix.cols;
        let bytes = GpuPod::into_bytes(&cols);
        constants.push(bytes, 0, ModuleVisibility::Vertex).unwrap();

        // Set the shadow lightspace matrix
        if P::is_shadow_pass() {
            let lightspace = _default.lightspace.unwrap();
            let offset = bytes.len();
            let bytes = GpuPod::into_bytes(&lightspace.cols);
            constants.push(bytes, offset as u32, ModuleVisibility::Vertex).unwrap();
        }

        // The rest is fragment data for the deferred pass
        if !P::is_deferred_pass() {
            return;
        }

        // Convert the material parameters into a vec4
        let vector = vek::Vec4::new(
            self.bumpiness_factor,
            self.metallic_factor,
            self.ambient_occlusion_factor,
            self.roughness_factor,
        );

        // Send the raw fragment bytes to the GPU
        let bytes = GpuPod::into_bytes(&vector);
        constants
            .push(bytes, 0, ModuleVisibility::Fragment)
            .unwrap();
        let mut offset = bytes.len();

        // Send the bytes containing tint of the object
        let vector = vek::Rgba::<f32>::from(self.tint);
        let bytes = GpuPod::into_bytes(&vector);
        constants
            .push(bytes, offset as u32, ModuleVisibility::Fragment)
            .unwrap();
        offset += bytes.len();

        // Send the bytes containing the UV scale of the textures to be sampled
        let bytes = GpuPod::into_bytes(&self.scale);
        constants
            .push(bytes, offset as u32, ModuleVisibility::Fragment)
            .unwrap();
    }
}