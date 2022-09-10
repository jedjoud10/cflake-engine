use crate::{
    context::Context, display::Display, mesh::Mesh, others::Comparison, prelude::ValidUniforms,
};
use std::{intrinsics::transmute, mem::transmute_copy, ptr::null};

use super::Viewport;

// Blend mode factor source
// This is a certified bruh moment classic
#[repr(u32)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Factor {
    Zero = gl::ZERO,
    One = gl::ONE,
    SrcColor = gl::SRC_COLOR,
    DstColor = gl::DST_COLOR,
    SrcAlpha = gl::SRC_ALPHA,
    DstAlpha = gl::DST_ALPHA,
    OneMinusSrcColor = gl::ONE_MINUS_SRC_COLOR,
    OneMinusDstColor = gl::ONE_MINUS_DST_COLOR,
    OneMinusSrcAlpha = gl::ONE_MINUS_SRC_ALPHA,
    OneMinusDstAlpha = gl::ONE_MINUS_DST_ALPHA,
}

// Tells us if we how we should blend between transparent objects
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct BlendMode {
    pub src: Factor,
    pub dest: Factor,
}

// How rasterized triangles should be culled
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FaceCullMode {
    Front(bool),
    Back(bool),
}

// Main rasterizer self like sissor tests and depth tests
#[derive(Clone, Copy, PartialEq)]
pub struct RasterSettings {
    // Should we check for vertex depth when rasteizing?
    pub depth_test: Option<Comparison>,

    // A scissor test basically limits the area of effect when rasterizing. Pretty useful for UI
    pub scissor_test: Option<Viewport>,

    // The current primitive that we will render with. Currently supported: Triangles and Points
    pub primitive: PrimitiveMode,

    // Should we render in SRGB or not?
    pub srgb: bool,

    // Transparancy blending mode
    pub blend: Option<BlendMode>,
}

// Depicts the exact primitives we will use to draw the VAOs
#[derive(Clone, Copy, PartialEq)]
pub enum PrimitiveMode {
    Triangles { cull: Option<FaceCullMode> },
    Lines { width: f32, smooth: bool },
    Points { diameter: f32 },
}

// A rasterizer will help us render specific shaded / colored objects onto the screen
// Painters can be fetched from any mutable reference to a canvas
pub struct Rasterizer<'d, 'context, D: Display> {
    display: &'d mut D,
    context: &'context mut Context,
    primitive: u32,
    vao: u32,
}

impl<'d, 'context, D: Display> Rasterizer<'d, 'context, D> {
    // Create a new rasterizer with the specified raster settings and display adapter
    pub(crate) unsafe fn new(
        display: &'d mut D,
        context: &'context mut Context,
        settings: RasterSettings,
    ) -> Self {
        // We must bind the display to the current opengl context
        gl::BindFramebuffer(gl::FRAMEBUFFER, display.name());

        // Update the settings of the OpenGL viewport
        if context.viewport != display.viewport() {
            context.viewport = display.viewport();
            gl::Viewport(
                context.viewport.origin.x as i32,
                context.viewport.origin.y as i32,
                context.viewport.extent.w as i32,
                context.viewport.extent.h as i32,
            );
        }

        // Get the OpenGL primitive type
        let primitive = get_primtive_gl_enum(settings.primitive);
        if context.raster.primitive != settings.primitive {
            context.raster.primitive = settings.primitive;
            set_state_primitive_mode(settings.primitive);
        }

        // Handle depth testing and it's parameters
        if context.raster.depth_test != settings.depth_test {
            context.raster.depth_test = settings.depth_test;
            set_state_depth_testing(settings.depth_test);
        }

        // Handle scissor testing and it's parameters
        if context.raster.scissor_test != settings.scissor_test {
            context.raster.scissor_test = settings.scissor_test;
            set_state_scissor_testing(settings.scissor_test, display);
        }

        // Handle framebuffer SRGB
        if context.raster.srgb != settings.srgb {
            context.raster.srgb = settings.srgb;
            set_state_fbo_srgb(settings.srgb);
        }

        // Handle transparent blending
        if context.raster.blend != settings.blend {
            context.raster.blend = settings.blend;
            set_state_blending(settings.blend);
        }

        Self {
            display,
            context,
            primitive,
            vao: u32::MAX,
        }
    }

    // Get the underlying display value
    pub fn display(&self) -> &D {
        self.display
    }

    // Draw a VAO by assuming that it has no EBO
    pub unsafe fn draw_vao_arrays<'a>(
        &mut self,
        vao: u32,
        primitive_count: usize,
        _uniforms: ValidUniforms,
    ) {
        if primitive_count > 0 {
            if self.vao != vao {
                gl::BindVertexArray(vao);
                self.vao = vao;
            }
            gl::DrawArrays(self.primitive, 0, primitive_count as i32);
        }
    }

    // Draw a VAO by assuming that it has a valid EBO linked to it
    pub unsafe fn draw_vao_elements<'a>(
        &mut self,
        vao: u32,
        primitive_count: usize,
        element_type: u32,
        _uniforms: ValidUniforms,
    ) {
        if primitive_count > 0 {
            if self.vao != vao {
                gl::BindVertexArray(vao);
                self.vao = vao;
            }
            gl::DrawElements(self.primitive, primitive_count as i32, element_type, null());
        }
    }

    // Draw a 3D engine mesh
    pub fn draw<'a>(&mut self, mesh: &Mesh, uniforms: ValidUniforms) {
        unsafe {
            let count = mesh.triangles().len();
            self.draw_vao_elements(mesh.vao, count * 3, gl::UNSIGNED_INT, uniforms)
        }
    }
}

// Set the OpenGL transparent blending settings
unsafe fn set_state_blending(blending: Option<BlendMode>) {
    if let Some(mode) = blending {
        gl::Enable(gl::BLEND);
        gl::BlendFunc(
            transmute::<Factor, u32>(mode.src),
            transmute::<Factor, u32>(mode.dest),
        );
    } else {
        gl::Disable(gl::BLEND)
    }
}

// Set the OpenGL SRGB framebuffer setting
unsafe fn set_state_fbo_srgb(srgb: bool) {
    if srgb {
        gl::Enable(gl::FRAMEBUFFER_SRGB);
    } else {
        gl::Disable(gl::FRAMEBUFFER_SRGB);
    }
}

// Set the OpenGL scissor testing settings
unsafe fn set_state_scissor_testing<D: Display>(scissor_test: Option<Viewport>, display: &mut D) {
    if let Some(Viewport { origin, extent }) = &scissor_test {
        gl::Enable(gl::SCISSOR_TEST);
        gl::Scissor(
            origin.x as i32,
            display.viewport().extent.h as i32 - origin.y as i32,
            extent.w as i32,
            extent.h as i32,
        );
    } else {
        gl::Disable(gl::SCISSOR_TEST);
    }
}

// Set the OpenGL depth testing settings
unsafe fn set_state_depth_testing(depth_test: Option<Comparison>) {
    if let Some(func) = &depth_test {
        gl::Enable(gl::DEPTH_TEST);
        gl::DepthFunc(transmute_copy::<Comparison, u32>(func));
    } else {
        gl::Disable(gl::DEPTH_TEST);
    }
}

// Get the raw OpenGL primitive ID from the primitive mode
fn get_primtive_gl_enum(primitive: PrimitiveMode) -> u32 {
    match primitive {
        PrimitiveMode::Triangles { .. } => gl::TRIANGLES,
        PrimitiveMode::Lines { .. } => gl::LINES,
        PrimitiveMode::Points { .. } => gl::POINTS,
    }
}

// Set the OpenGL primitive settings (along with face culling)
unsafe fn set_state_primitive_mode(primitive: PrimitiveMode) {
    match primitive {
        // Triangle primitive type
        PrimitiveMode::Triangles { cull } => {
            if let Some(cull) = cull {
                gl::Enable(gl::CULL_FACE);
                let (direction, ccw) = match cull {
                    FaceCullMode::Front(ccw) => (gl::FRONT, ccw),
                    FaceCullMode::Back(ccw) => (gl::BACK, ccw),
                };
                gl::CullFace(direction);
                gl::FrontFace(if ccw { gl::CCW } else { gl::CW });
            } else {
                gl::Disable(gl::CULL_FACE);
            }
        }

        // Point primitive type
        PrimitiveMode::Points { diameter } => {
            gl::PointSize(diameter);
        }

        // Line primitive type
        PrimitiveMode::Lines { width, smooth } => {
            if smooth {
                gl::Enable(gl::LINE_SMOOTH);
            } else {
                gl::Disable(gl::LINE_SMOOTH);
            }
            gl::LineWidth(width);
        }
    }
}
