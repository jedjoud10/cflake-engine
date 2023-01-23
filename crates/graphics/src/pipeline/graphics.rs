use crate::{
    AttachmentBlendConfig, BlendConfig, ColorLayout, Compiled,
    DepthConfig, DepthStencilLayout, Graphics,
    PipelineInitializationError, PipelineVertexConfigError,
    Primitive, RenderPass, Shader, ShaderModule, StencilConfig,
    VertexConfig, BindingConfig, PipelineBindingConfigError, ModuleKind, CompiledDescription, PushConstBlockError,
};

use ahash::AHashMap;
use vulkan::vk;

// A vulkan GRAPHICS pipeline abstraction that will handle initialization / destruction for us manually
// This will abstract most of the initialization and pain staking work of pipelines

// This pipeline is only to be used with the Material system in the "rendering" crate
// By itself, it only contains only 2 dynamic state that we can change after building it,
// which is it's viewport size and scissor testing
pub struct GraphicsPipeline {
    // Raw Vulkan
    pipeline: vk::Pipeline,
    layout: vk::PipelineLayout,

    // Immutable data set at build time
    depth_config: DepthConfig,
    stencil_config: StencilConfig,
    blend_config: BlendConfig,
    vertex_config: VertexConfig,
    primitive: Primitive,
    binding_config: BindingConfig,

    // Keep the shader modules alive
    shader: Shader,

    // Keep the graphics API alive
    graphics: Graphics,
}

impl Drop for GraphicsPipeline {
    fn drop(&mut self) {
        unsafe {
            self.graphics
                .device()
                .destroy_pipeline_layout(self.layout);
            self.graphics.device().destroy_pipeline(self.pipeline);
        }
    }
}

// Initialization of the graphics pipeline
impl GraphicsPipeline {
    // Create a new pipeline with the specified configs
    pub fn new<C: ColorLayout, DS: DepthStencilLayout>(
        graphics: &Graphics,
        depth_config: DepthConfig,
        stencil_config: StencilConfig,
        blend_config: BlendConfig,
        vertex_config: VertexConfig,
        primitive: Primitive,
        binding_config: BindingConfig,
        render_pass: &RenderPass<C, DS>,
        shader: Shader,
    ) -> Result<Self, PipelineInitializationError> {
        let (pipeline, layout) = unsafe {
            // Common values that we reuse
            let depth_config = &depth_config;
            let primitive = &primitive;
            let stencil_config = &stencil_config;
            let binding_config = &binding_config;
            let shader = &shader;

            // All the pipeline state builders
            let (viewport, scissor) = viewport_and_scissor();
            let viewport_state =
                viewport_state_builder(&viewport, &scissor);
            let input_assembly_state =
                input_assembly_state_builder(primitive);
            let rasterization_state =
                rasterization_state_builder(primitive, depth_config);
            let attachments = color_blend_attachments(blend_config);
            let color_blend_state =
                color_blend_state_builder(&attachments);
            let depth_stencil_state = depth_stencil_state_builder(
                depth_config,
                stencil_config,
            );

            // Vertex input state
            let vertex_attribute_descriptions =
                vertex_attribute_descriptions(&vertex_config)
                .map_err(PipelineInitializationError::VertexConfigError)?;
            let vertex_binding_descriptions =
                vertex_intput_binding_descriptions(&vertex_config)
                .map_err(PipelineInitializationError::VertexConfigError)?;
            let vertex_input_state = vertex_input_state_builder(
                &vertex_attribute_descriptions,
                &vertex_binding_descriptions,
            );

            // Dyanmic state (only viewport and scissor)
            let dynamic = dynamic_states();
            let dynamic_state = dynamic_state_builder(&dynamic);
            
            // Pipeline layout (descriptors / push constants)
            let layout = pipeline_layout(graphics, binding_config, shader)
                .map_err(PipelineInitializationError::BindingConfigError)?;

            // Multisampling (I HATE ANTIALISATION. FUCK YOU. COPE)
            let multisample_state = multisample_state_builder();

            // Pipeline stages create info
            let stages = shader
                .descriptions()
                .into_iter()
                .map(|description| {
                    let stage =
                        description.kind.into_shader_stage_flags();
                    *vk::PipelineShaderStageCreateInfo::builder()
                        .name(description.entry)
                        .flags(description.flags)
                        .module(*description.module)
                        .stage(stage)
                        .specialization_info(
                            &description.constants.raw,
                        )
                })
                .collect::<Vec<_>>();

            // Create info for the graphics pipeline
            let create_info_builder =
                vk::GraphicsPipelineCreateInfo::builder()
                    .color_blend_state(&color_blend_state)
                    .depth_stencil_state(&depth_stencil_state)
                    .dynamic_state(&dynamic_state)
                    .input_assembly_state(&input_assembly_state)
                    .layout(layout)
                    .rasterization_state(&rasterization_state)
                    .multisample_state(&multisample_state)
                    .render_pass(render_pass.renderpass())
                    .viewport_state(&viewport_state)
                    .stages(&stages)
                    .subpass(0)
                    .vertex_input_state(&vertex_input_state);
            let create_info = *create_info_builder;

            (
                graphics
                    .device()
                    .create_graphics_pipeline(create_info),
                layout,
            )
        };

        Ok(Self {
            pipeline,
            layout,
            depth_config,
            stencil_config,
            blend_config,
            vertex_config,
            primitive,
            binding_config,
            shader,
            graphics: graphics.clone(),
        })
    }
}

impl GraphicsPipeline {
    // Get the underlying raw Vulkan pipeline
    pub fn raw(&self) -> vk::Pipeline {
        self.pipeline
    }

    // Get the pipeline layout that we have specified
    pub fn layout(&self) -> vk::PipelineLayout {
        self.layout
    }

    // Get the depth config used when creating this pipeline
    pub fn depth_config(&self) -> &DepthConfig {
        &self.depth_config
    }

    // Get the stencil config used when creating this pipeline
    pub fn stencil_config(&self) -> &StencilConfig {
        &self.stencil_config
    }

    // Get the blend config used when creating this pipeline
    pub fn blend_config(&self) -> &BlendConfig {
        &self.blend_config
    }

    // Get the vertex config used when creating this pipeline
    pub fn vertex_config(&self) -> &VertexConfig {
        &self.vertex_config
    }

    // Get the internally used shader for this graphics pipeline
    pub fn shader(&self) -> &Shader {
        &self.shader
    }

    // Get the primitive config used when creating this pipeline
    pub fn primitive(&self) -> &Primitive {
        &self.primitive
    }
}

// Create the vertex input state builder
fn vertex_input_state_builder<'a>(
    vertex_attribute_descriptions: &'a [vk::VertexInputAttributeDescription],
    vertex_binding_descriptions: &'a [vk::VertexInputBindingDescription],
) -> vk::PipelineVertexInputStateCreateInfoBuilder<'a> {
    vk::PipelineVertexInputStateCreateInfo::builder()
        .vertex_attribute_descriptions(&vertex_attribute_descriptions)
        .vertex_binding_descriptions(&vertex_binding_descriptions)
}

// Check if a module's push constants match up
fn check_push_constants(
    binding_config: &BindingConfig,
    description: CompiledDescription,
) -> Result<(), PushConstBlockError> {
    // Combine the required layout and the current one in one option
    let reflected = description.reflected.push_constant_block();
    let defined = binding_config.block_definitions().get(&description.kind);
    let zipped = reflected.zip(defined);
    let kind = description.kind;

    // Make sure the "defined" layout fits the layout of "required"
    if let Some((required, defined)) = zipped {
        // Check name mismatch
        if required.name != defined.name {
            return Err(PushConstBlockError::NameMismatch(kind));
        }
        
        // Check variables mismatch (check required)
        for (name, required_var) in &required.variables {
            if defined.variables.get(name) {

            } else {
                return Err(PushConstBlockError::VariableNotDefinedBindings((), (), ()))
            }
        }

        // Check variables mismatch (check defined)
        for (name, defined_var) in &defined.variables {
            retrun
        }

        if !size || !variables {
            return Err(PushConstBlockError::Mismatch(kind));
        } else {
            return Ok(());
        }

    } else {
        // Handle the layout being not defined in either the shader or in the confi
        if reflected.is_none() {
            return Err(PushConstBlockError::NotDefinedShader(defined.unwrap().name.clone(), kind));
        } else {
            return Err(PushConstBlockError::NotDefinedBindings(reflected.unwrap().name.clone(), kind));
        }
    }
}

// Create the pipeline layout (cached)
unsafe fn pipeline_layout(
    graphics: &Graphics,
    binding_config: &BindingConfig,
    shader: &Shader,
) -> Result<vk::PipelineLayout, PipelineBindingConfigError> {
    // Check if the shader push constants match up
    for description in shader.descriptions() {
        check_push_constants(binding_config, description)?;
    }


    

    // Reserve the required layout for bindless
    // Reserve the required layout for non bindless (UBO, push constant)

    let layout = {
        let create_info = vk::PipelineLayoutCreateInfo::builder()
            .flags(vk::PipelineLayoutCreateFlags::empty());
        graphics.device().create_pipeline_layout(*create_info)
    };
    Ok(layout)
}

// Create the multisampling state
fn multisample_state_builder<'a>(
) -> vk::PipelineMultisampleStateCreateInfoBuilder<'a> {
    let multisample_state =
        vk::PipelineMultisampleStateCreateInfo::builder()
            .sample_shading_enable(false)
            .rasterization_samples(vk::SampleCountFlags::TYPE_1)
            .sample_mask(&[])
            .min_sample_shading(1.0f32)
            .alpha_to_coverage_enable(false)
            .alpha_to_one_enable(false);
    multisample_state
}

// Create the vertex input bindings descriptions
fn vertex_intput_binding_descriptions(
    vertex_config: &VertexConfig,
) -> Result<Vec<vk::VertexInputBindingDescription>, PipelineVertexConfigError> {
    let vertex_binding_descriptions = vertex_config
        .bindings
        .iter()
        .map(|binding| {
            *vk::VertexInputBindingDescription::builder()
                .binding(binding.binding)
                .input_rate(vk::VertexInputRate::VERTEX)
                .stride(
                    binding.format.bits_per_axii as u32
                        * binding.format.channels.count()
                        / 8,
                )
        })
        .collect::<Vec<_>>();
    Ok(vertex_binding_descriptions)
}

// Create the vertex attribute descriptions
fn vertex_attribute_descriptions(
    vertex_config: &VertexConfig,
) -> Result<Vec<vk::VertexInputAttributeDescription>, PipelineVertexConfigError> {
    let vertex_attribute_descriptions = vertex_config
        .attributes
        .iter()
        .map(|attribute| {
            *vk::VertexInputAttributeDescription::builder()
                .binding(attribute.binding)
                .format(attribute.format.format)
                .location(attribute.location)
                .offset(attribute.offset)
        })
        .collect::<Vec<_>>();
    Ok(vertex_attribute_descriptions)
}

// Create the dynamic states
fn dynamic_state_builder<'a>(
    dynamic: &'a [vk::DynamicState],
) -> vk::PipelineDynamicStateCreateInfoBuilder<'a> {
    vk::PipelineDynamicStateCreateInfo::builder()
        .dynamic_states(dynamic)
}

// Required
fn dynamic_states() -> [vk::DynamicState; 2] {
    [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR]
}

fn depth_stencil_state_builder<'a>(
    depth_config: &DepthConfig,
    stencil_config: &StencilConfig,
) -> vk::PipelineDepthStencilStateCreateInfoBuilder<'a> {
    let mut builder =
        vk::PipelineDepthStencilStateCreateInfo::builder();
    builder = depth_config.apply_depth_stencil_state(builder);
    builder = stencil_config.apply_depth_stencil_state(builder);
    let depth_stencil_state = builder;
    depth_stencil_state
}

fn color_blend_attachments(
    blend_config: BlendConfig,
) -> [vk::PipelineColorBlendAttachmentState; 1] {
    // Color blend state attachment 0
    let attachment_builder = blend_config.attachments.map(|attachment| {
        attachment[0].apply_color_blend_attachment_state(vk::PipelineColorBlendAttachmentState::builder())
    }).unwrap_or_else(|| AttachmentBlendConfig::apply_default_color_blend_attachment_state(
        vk::PipelineColorBlendAttachmentState::builder()
    ));
    // Apply the color blend attachments to the state
    // TODO: Add render graph so we can support multiple attachments
    [*attachment_builder]
}

fn color_blend_state_builder<'a>(
    attachment_state: &'a [vk::PipelineColorBlendAttachmentState],
) -> vk::PipelineColorBlendStateCreateInfoBuilder<'a> {
    let mut color_blend_builder =
        vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op_enable(false)
            .logic_op(vk::LogicOp::COPY)
            .attachments(&[]);

    color_blend_builder =
        color_blend_builder.attachments(attachment_state);
    let color_blend_state = color_blend_builder;
    color_blend_state
}

fn rasterization_state_builder<'a>(
    primitive: &Primitive,
    depth_config: &DepthConfig,
) -> vk::PipelineRasterizationStateCreateInfoBuilder<'a> {
    let mut builder =
        vk::PipelineRasterizationStateCreateInfo::builder();
    builder = primitive.apply_rasterization_state(builder);
    builder = depth_config.apply_rasterization_state(builder);
    let rasterization_state = builder;
    rasterization_state
}

fn input_assembly_state_builder(
    primitive: &Primitive,
) -> vk::PipelineInputAssemblyStateCreateInfoBuilder {
    let mut builder =
        vk::PipelineInputAssemblyStateCreateInfo::builder();
    builder = primitive.apply_input_assembly_state(builder);
    let input_assembly_state = builder;
    input_assembly_state
}

fn viewport_and_scissor() -> ([vk::Viewport; 1], [vk::Rect2D; 1]) {
    ([vk::Viewport::default(); 1], [vk::Rect2D::default(); 1])
}

fn viewport_state_builder<'a>(
    viewports: &'a [vk::Viewport; 1],
    scissors: &'a [vk::Rect2D; 1],
) -> vk::PipelineViewportStateCreateInfoBuilder<'a> {
    vk::PipelineViewportStateCreateInfo::builder()
        .viewports(viewports)
        .scissors(scissors)
}
