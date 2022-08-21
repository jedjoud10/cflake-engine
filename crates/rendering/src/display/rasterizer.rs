use std::{intrinsics::transmute, mem::transmute_copy, ptr::null};

use super::{Canvas, CanvasLayout};
use crate::{context::Context, mesh::Mesh, others::Comparison, prelude::ValidUniforms};

// Blend mode factor source
// This is a certified bruh moment classic
#[repr(u32)]
#[derive(Clone, Copy)]
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
#[derive(Clone, Copy)]
pub struct BlendMode {
    pub src: Factor,
    pub dest: Factor,
}

// How rasterized triangles should be culled
#[derive(Clone, Copy)]
pub enum FaceCullMode {
    Front(bool),
    Back(bool),
}

// Main rasterizer self like sissor tests and depth tests
#[derive(Clone)]
pub struct RasterSettings {
    // Should we check for vertex depth when rasteizing?
    pub depth_test: Option<Comparison>,

    // A scissor test basically limits the area of effect when rasterizing. Pretty useful for UI
    pub scissor_test: Option<(vek::Vec2<i32>, vek::Extent2<i32>)>,

    // The current primitive that we will render with. Currently supported: Triangles and Points
    pub primitive: PrimitiveMode,

    // Should we render in SRGB or not?
    pub srgb: bool,

    // Transparancy blending mode
    pub blend: Option<BlendMode>,
}

// Depicts the exact primitives we will use to draw the VAOs
#[derive(Clone)]
pub enum PrimitiveMode {
    Triangles { cull: Option<FaceCullMode> },
    Lines { width: f32, smooth: bool },
    Points { diameter: f32 },
}

// A rasterizer will help us render specific shaded / colored objects onto the screen
// Painters can be fetched from any mutable reference to a canvas
pub struct Rasterizer<'canvas, 'context, L: CanvasLayout> {
    canvas: &'canvas mut Canvas<L>,
    ctx: &'context mut Context,
    primitive: u32,
}

impl<'canvas, 'context, L: CanvasLayout> Rasterizer<'canvas, 'context, L> {
    // Create a new rasterizer with the specified raster self
    pub(crate) fn new(
        canvas: &'canvas mut Canvas<L>,
        ctx: &'context mut Context,
        settings: RasterSettings,
    ) -> Self {
        // Get the OpenGL primitive type
        let primitive = match settings.primitive {
            PrimitiveMode::Triangles { .. } => gl::TRIANGLES,
            PrimitiveMode::Points { .. } => gl::POINTS,
            PrimitiveMode::Lines { .. } => gl::LINES,
        };

        // Set the OpenGL primitive parameters (along with face culling)
        match &settings.primitive {
            // Triangle primitive type
            PrimitiveMode::Triangles { cull } => unsafe {
                if let Some(cull) = cull {
                    gl::Enable(gl::CULL_FACE);
                    let (direction, ccw) = match cull {
                        FaceCullMode::Front(ccw) => (gl::FRONT, ccw),
                        FaceCullMode::Back(ccw) => (gl::BACK, ccw),
                    };
                    gl::CullFace(direction);
                    gl::FrontFace(if *ccw { gl::CCW } else { gl::CW });
                } else {
                    gl::Disable(gl::CULL_FACE);
                };
            },

            // Point primitive type
            PrimitiveMode::Points { diameter } => unsafe {
                gl::PointSize(*diameter);
            },

            // Line primitive type
            PrimitiveMode::Lines { width, smooth } => unsafe {
                if *smooth {
                    gl::Enable(gl::LINE_SMOOTH);
                } else {
                    gl::Disable(gl::LINE_SMOOTH);
                }
                gl::LineWidth(*width);
            },
        }

        // Handle depth testing and it's parameters
        unsafe {
            if let Some(func) = &settings.depth_test {
                gl::Enable(gl::DEPTH_TEST);
                gl::DepthFunc(transmute_copy::<Comparison, u32>(func));
            } else {
                gl::Disable(gl::DEPTH_TEST);
            }
        }

        // Handle scissor testing and it's parameters
        unsafe {
            if let Some((origin, size)) = &settings.scissor_test {
                gl::Enable(gl::SCISSOR_TEST);
                gl::Scissor(origin.x, canvas.size().h as i32 - origin.y, size.w, size.h);
            } else {
                gl::Disable(gl::SCISSOR_TEST);
            }
        }

        // Handle the SRGB framebuffer mode
        unsafe {
            if settings.srgb {
                gl::Enable(gl::FRAMEBUFFER_SRGB);
            } else {
                gl::Disable(gl::FRAMEBUFFER_SRGB);
            }
        }

        // Handle blending and it's parameters
        unsafe {
            if let Some(mode) = settings.blend {
                gl::Enable(gl::BLEND);
                gl::BlendFunc(
                    transmute::<Factor, u32>(mode.src),
                    transmute::<Factor, u32>(mode.dest),
                );
            } else {
                gl::Disable(gl::BLEND)
            }
        }

        Self {
            canvas,
            ctx,
            primitive,
        }
    }

    // Get an immutable reference to the underlying canvas
    pub fn canvas(&self) -> &Canvas<L> {
        self.canvas
    }

    // Get an immutable reference to the underlying context
    pub fn context(&self) -> &Context {
        self.ctx
    }

    // Draw a VAO by assuming that it has no EBO
    pub unsafe fn draw_vao_arrays<'a>(
        &mut self,
        vao: u32,
        primitive_count: usize,
        _uniforms: ValidUniforms,
    ) {
        if primitive_count > 0 {
            gl::BindVertexArray(vao);
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
            gl::BindVertexArray(vao);
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
