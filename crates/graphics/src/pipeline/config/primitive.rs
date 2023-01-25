use vulkan::vk;

// How rasterized triangles should be culled
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FaceCullMode {
    Front(bool),
    Back(bool),
}

// Depicts the exact primitives we will use to draw the VAOs
#[derive(Clone, Copy, PartialEq)]
pub enum Primitive {
    Triangles {
        cull: Option<FaceCullMode>,
        wireframe: bool,
    },
    Lines {
        width: f32,
    },
    Points,
}

impl Primitive {
    // Get the cull mode direction from the primtive mode
    fn build_front_face(&self) -> vk::FrontFace {
        match self {
            Primitive::Triangles { cull, .. } => cull
                .map(|mode| {
                    let ccw = match mode {
                        FaceCullMode::Front(ccw) => ccw,
                        FaceCullMode::Back(ccw) => ccw,
                    };

                    if ccw {
                        vk::FrontFace::COUNTER_CLOCKWISE
                    } else {
                        vk::FrontFace::CLOCKWISE
                    }
                })
                .unwrap_or(vk::FrontFace::CLOCKWISE),
            Primitive::Lines { .. } => vk::FrontFace::CLOCKWISE,
            Primitive::Points => vk::FrontFace::CLOCKWISE,
        }
    }

    // Get the cull mode flags from the primitive mode
    fn build_cull_mode_flags(&self) -> vk::CullModeFlags {
        match self {
            Primitive::Triangles { cull, .. } => cull
                .map(|face_cull_mode| match face_cull_mode {
                    FaceCullMode::Front(_) => {
                        vk::CullModeFlags::FRONT
                    }
                    FaceCullMode::Back(_) => vk::CullModeFlags::BACK,
                })
                .unwrap_or(vk::CullModeFlags::NONE),
            Primitive::Lines { .. } => vk::CullModeFlags::NONE,
            Primitive::Points => vk::CullModeFlags::NONE,
        }
    }

    // Get the polygon mode from the primitive mode
    fn build_polygon_mode(&self) -> vk::PolygonMode {
        match self {
            Primitive::Triangles { wireframe, .. } => {
                if *wireframe {
                    vk::PolygonMode::LINE
                } else {
                    vk::PolygonMode::FILL
                }
            }
            Primitive::Lines { .. } => vk::PolygonMode::FILL,
            Primitive::Points => vk::PolygonMode::POINT,
        }
    }

    // Create the input assembly state for this primitive
    pub fn apply_input_assembly_state<'a>(
        &self,
        builder: vk::PipelineInputAssemblyStateCreateInfoBuilder<'a>,
    ) -> vk::PipelineInputAssemblyStateCreateInfoBuilder<'a> {
        let topology = match self {
            crate::Primitive::Triangles { .. } => {
                vk::PrimitiveTopology::TRIANGLE_LIST
            }
            crate::Primitive::Lines { .. } => {
                vk::PrimitiveTopology::LINE_LIST
            }
            crate::Primitive::Points => {
                vk::PrimitiveTopology::POINT_LIST
            }
        };

        builder.topology(topology).primitive_restart_enable(false)
    }

    // Create initialize a pipeline rasterization state builder using this primitive
    pub fn apply_rasterization_state<'a>(
        &self,
        builder: vk::PipelineRasterizationStateCreateInfoBuilder<'a>,
    ) -> vk::PipelineRasterizationStateCreateInfoBuilder<'a> {
        let polygon_mode = self.build_polygon_mode();
        let cull_mode = self.build_cull_mode_flags();
        let front_face = self.build_front_face();

        builder
            .front_face(front_face)
            .cull_mode(cull_mode)
            .polygon_mode(polygon_mode)
            .line_width(1.0)
    }
}
