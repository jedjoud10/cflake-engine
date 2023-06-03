

use assets::Assets;
use bytemuck::{Pod, Zeroable};
use graphics::{
    ActiveRenderPass, ActiveRenderPipeline, BufferMode, BufferUsage, CompareFunction, Compiler,
    Depth, DepthConfig, Face, FragmentModule, GpuPod, Graphics, LayeredTexture2D, LoadOp,
    ModuleVisibility, Operation, PrimitiveConfig, PushConstantLayout, RenderPass,
    RenderPipeline, SamplerSettings, Shader, StoreOp, Texture, TextureMipMaps,
    TextureMode, TextureUsage, UniformBuffer, VertexModule, WindingOrder,
};
use math::ExplicitVertices;
use vek::FrustumPlanes;

use crate::MeshAttributes;

// This is what will write to the depth texture
pub type ShadowDepthLayout = Depth<f32>;
pub type ShadowMap = LayeredTexture2D<ShadowDepthLayout>;

// Directional shadow mapping for the main sun light
// The shadows must be rendered before we render the main frame
pub struct ShadowMapping {
    // Everything required to render to the depth texture
    pub render_pass: RenderPass<(), ShadowDepthLayout>,

    // Multilayered shadow map texture
    pub depth_tex: ShadowMap,

    // Cached matrices
    pub percents: [f32; 4],

    // Resolution of the base level
    pub resolution: u32,
    pub depth: f32,

    // Contains shadow parameters
    pub parameters: ShadowUniform,
    pub parameter_buffer: UniformBuffer<ShadowUniform>,

    // Contains the light space shadow matrices
    pub lightspace_buffer: UniformBuffer<vek::Vec4<vek::Vec4<f32>>>,

    // Contains the depth distances for each plane
    pub cascade_distances: UniformBuffer<f32>,
}

// This is the uniform that is defined in the Vertex Module
#[derive(Clone, Copy, PartialEq, Pod, Zeroable, Default)]
#[repr(C)]
pub struct ShadowUniform {
    pub strength: f32,
    pub spread: f32,
    pub base_bias: f32,
	pub bias_bias: f32,
	pub bias_factor_base: f32,
	pub normal_offset: f32,
}

impl ShadowMapping {
    // Create a new shadow mapper that will create some nice crispy shadows
    pub(crate) fn new(
        depth: f32,
        resolution: u32,
        percents: [f32; 4],
        graphics: &Graphics,
        assets: &mut Assets,
    ) -> Self {
        // Create the shadow map render pass
        let render_pass = RenderPass::<(), ShadowDepthLayout>::new(
            graphics,
            (),
            Operation {
                load: LoadOp::Clear(f32::MAX),
                store: StoreOp::Store,
            },
        );

        // Create the depth textures that we will render to
        let depth_tex = ShadowMap::from_texels(
            graphics,
            None,
            (vek::Extent2::broadcast(resolution), 4),
            TextureMode::Dynamic,
            TextureUsage::TARGET | TextureUsage::SAMPLED,
            Some(SamplerSettings::default()),
            TextureMipMaps::Disabled,
        )
        .unwrap();

        let parameters = ShadowUniform {
            strength: 1.0,
            spread: 0.01,
            base_bias: -0.0002,
            bias_bias: 0.0006,
            bias_factor_base: 1.55,
            normal_offset: 0.09,
        };

        // Create a buffer that will contain shadow parameters
        let parameter_buffer = UniformBuffer::from_slice(
            graphics,
            &[parameters],
            BufferMode::Dynamic,
            BufferUsage::WRITE,
        )
        .unwrap();

        // We can initialize these to zero since the first frame would update the buffer anyways
        let lightspace_buffer = UniformBuffer::<vek::Vec4<vek::Vec4<f32>>>::zeroed(
            graphics,
            4,
            BufferMode::Dynamic,
            BufferUsage::WRITE,
        )
        .unwrap();

        // We can initialize these to zero since the first frame would update the buffer anyways
        let cascade_distances =
            UniformBuffer::<f32>::zeroed(graphics, 4, BufferMode::Dynamic, BufferUsage::WRITE)
                .unwrap();

        Self {
            render_pass,
            depth_tex,
            resolution,
            parameter_buffer,
            lightspace_buffer,
            depth,
            percents,
            cascade_distances,
            parameters,
        }
    }

    // Update the rotation of the sun shadows using a new rotation
    // Returns the newly created lightspace matrix (only one)
    // https://learnopengl.com/Guest-Articles/2021/CSM
    pub(crate) fn update(
        &mut self,
        rotation: vek::Quaternion<f32>,
        view: vek::Mat4<f32>,
        mut projection: vek::Mat4<f32>,
        _camera: vek::Vec3<f32>,
        camera_near_plane: f32,
        camera_far_plane: f32,
        i: usize,
    ) -> vek::Mat4<f32> {
        /*
        // Update the projection matrix' far and near planes
        let near = self
            .percents
            .get(i.wrapping_sub(1))
            .map(|x| x * &camera_far_plane)
            .unwrap_or(camera_near_plane);
        let far = self
            .percents
            .get(i)
            .map(|x| x * &camera_far_plane)
            .unwrap_or(camera_far_plane);
        let m22 = far / (near - far);
        let m23 = -(far * near) / (far - near);
        projection.cols[2][2] = m22;
        projection.cols[3][2] = m23;

        // Get the corners of the frustum matrix
        let ndc = math::Aabb::<f32>::ndc();
        let inverse = (projection * view).inverted();
        let corners = ndc.points().map(|x| {
            let vec4 = inverse * vek::Vec4::<f32>::from_point(x);
            vec4.xyz() / vec4.w
        });

        // Calculate a new view matrix and set it
        let rot = vek::Mat4::from(rotation);

        // Calculate light view matrix
        let view = vek::Mat4::<f32>::look_at_rh(
            vek::Vec3::zero(),
            rot.mul_point(-vek::Vec3::unit_z()),
            rot.mul_point(-vek::Vec3::unit_y()),
        );

        // Get the AABB that contains the whole corners
        let mut min = vek::Vec3::broadcast(f32::MAX);
        let mut max = vek::Vec3::broadcast(f32::MIN);

        for point in corners {
            // Project point using view matrix
            // Note: W component should be 1 since it is not a projection matrix, only view matrix
            let point = view * (point).with_w(1.0);

            // Update the "max" bound element wise
            min.x = min.x.min(point.x);
            min.y = min.y.min(point.y);
            min.z = min.z.min(point.z);

            // Update the "min" bound element wise
            max.x = max.x.max(point.x);
            max.y = max.y.max(point.y);
            max.z = max.z.max(point.z);
        }

        // The shadow frustum is the cuboid that will contain the shadow map
        let frustum = FrustumPlanes::<f32> {
            left: min.x,
            right: max.x,
            bottom: min.y,
            top: max.y,
            near: -self.depth,
            far: self.depth,
        };

        // Create the projection matrix (orthographic)
        let projection = vek::Mat4::orthographic_rh_zo(frustum);

        // Calculate light skin rizz (real) (I have gone insane)
        let lightspace = projection * view;

        // Update the internally stored buffer
        self.lightspace_buffer.write(&[lightspace.cols], i).unwrap();
        self.cascade_distances.write(&[far], i).unwrap();
        lightspace
        */

        let frustum = FrustumPlanes::<f32> {
            left: -200.0,
            right: 200.0,
            bottom: -200.0,
            top: 200.0,
            near: -self.depth,
            far: self.depth,
        };

        
        // Calculate a new view matrix and set it
        let rot = vek::Mat4::from(rotation);

        // Calculate light view matrix
        let view = vek::Mat4::<f32>::look_at_rh(
            vek::Vec3::zero(),
            rot.mul_point(-vek::Vec3::unit_z()),
            rot.mul_point(-vek::Vec3::unit_y()),
        );

        // Create the projection matrix (orthographic)
        let projection = vek::Mat4::orthographic_rh_zo(frustum);

        // Calculate light skin rizz (real) (I have gone insane)
        let lightspace = projection * view;
        self.lightspace_buffer.write(&[lightspace.cols], i).unwrap();
        self.parameter_buffer.write(&[self.parameters], 0).unwrap();
        lightspace
    }
}