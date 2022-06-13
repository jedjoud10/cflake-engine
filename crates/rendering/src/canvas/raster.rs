use crate::{
    buffer::ElementBuffer,
    canvas::blend::{BlendMode, Factor},
    canvas::Canvas,
    commons::Comparison,
    context::Context,
    mesh::attributes::AttributeSet,
    object::ToGlName,
    shader::Shader,
};
use std::{
    mem::{transmute, transmute_copy},
    ptr::null,
};

// How rasterized triangles should be culled
#[derive(Clone)]
pub enum FaceCullMode {
    // The boolean specifies if the culling should be Counter Clockwise
    Front(bool),
    Back(bool),

    // Don't cull anything
    None,
}

// Main rasterizer settings like sissor tests and depth tests
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

// Depicts the exact primitives we will use to draw the mesh
#[derive(Clone)]
pub enum PrimitiveMode {
    Triangles { cull: FaceCullMode },
    Points { diameter: f32 },
}

// An object that can be rasterized and drawn onto the screen
pub trait ToRasterBuffers {
    // Get the VAO handle of the object
    fn vao(&self) -> &AttributeSet;

    // Get the EBO handle of the object
    fn ebo(&self) -> &ElementBuffer<u32>;
}

// A rasterizer is what will draw our vertices and triangles onto the screen, so we can actually see them as lit pixels
// Each rasterizer will use a unique shared shader
pub struct Rasterizer<'canvas, 'shader, 'context> {
    pub(super) canvas: &'canvas mut Canvas,
    pub(super) shader: &'shader mut Shader,
    pub(super) context: &'context mut Context,
}

impl<'canvas, 'shader, 'context> Rasterizer<'canvas, 'shader, 'context> {
    // Get a mutable reference to the shader
    pub fn shader_mut(&mut self) -> &mut Shader {
        &mut self.shader
    }

    // Get an immutable reference to the shader
    pub fn shader(&self) -> &Shader {
        self.shader
    }

    // Prepare the rasterizer by setting the global raster settings
    fn prepare(&mut self, settings: &RasterSettings) -> u32 {
        self.context
            .bind(gl::PROGRAM, self.shader.as_ref().name(), |name| unsafe {
                gl::UseProgram(name)
            });

        // Get the OpenGL primitive type
        let primitive = match settings.primitive {
            PrimitiveMode::Triangles { .. } => gl::TRIANGLES,
            PrimitiveMode::Points { .. } => gl::POINTS,
        };

        // Set the global OpenGL face culling mode
        unsafe fn set_cull_mode(mode: &FaceCullMode) {
            // Check if we must cull the faces or not
            if let FaceCullMode::None = mode {
                gl::Disable(gl::CULL_FACE);
                return;
            } else {
                gl::Enable(gl::CULL_FACE)
            };

            // Get the face culling direction, either front or back, and winding order
            let (direction, ccw) = match mode {
                FaceCullMode::Front(ccw) => (gl::FRONT, ccw),
                FaceCullMode::Back(ccw) => (gl::BACK, ccw),
                _ => todo!(),
            };

            // Set the face culling direction
            gl::CullFace(direction);

            // And set winding order
            gl::FrontFace(if *ccw { gl::CCW } else { gl::CW });
        }

        // Set the global OpenGL point size
        unsafe fn set_point_size(diameter: &f32) {
            gl::PointSize(*diameter);
        }

        // Set the OpenGL primitive parameters
        match &settings.primitive {
            PrimitiveMode::Triangles { cull } => unsafe { set_cull_mode(cull) },
            PrimitiveMode::Points { diameter } => unsafe { set_point_size(diameter) },
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
                gl::Scissor(
                    origin.x,
                    self.canvas.size().h as i32 - origin.y,
                    size.w,
                    size.h,
                );
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
                    transmute::<Factor, u32>(mode.s_factor),
                    transmute::<Factor, u32>(mode.d_factor),
                );
            } else {
                gl::Disable(gl::BLEND)
            }
        }

        primitive
    }

    // Rasterize the raw VAO an EBO without setting the mode or binding the shader
    unsafe fn draw_from_raw_parts(&mut self, primitive: u32, vao: u32, ebo: u32, count: u32) {
        gl::BindVertexArray(vao);
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
        gl::DrawElements(primitive, count as i32, gl::UNSIGNED_INT, null());
    }

    // Draw a single VAO and a EBO using their raw OpenGL names directly
    pub unsafe fn draw_unchecked(
        &mut self,
        vao: u32,
        ebo: u32,
        count: u32,
        settings: &RasterSettings,
    ) {
        let primitive = self.prepare(settings);

        // Draw the VAO and EBO
        self.draw_from_raw_parts(primitive, vao, ebo, count);
    }

    // Draw a single VAO and EBO
    pub fn draw<T: ToRasterBuffers>(&mut self, obj: &T, settings: &RasterSettings) {
        let primitive = self.prepare(settings);

        let vao = obj.vao();
        let ebo = obj.ebo();

        unsafe { self.draw_from_raw_parts(primitive, vao.name(), ebo.name(), ebo.len() as u32) }
    }

    // This will draw a set of VAOs and EBOs directly onto the screen
    pub fn draw_batch<T: ToRasterBuffers>(&mut self, objects: &[&T], settings: &RasterSettings) {
        let primitive = self.prepare(settings);

        // Iterate through each object and draw it
        for object in objects {
            // Get the raw OpenGL names
            let vao = object.vao();
            let ebo = object.ebo();

            unsafe { self.draw_from_raw_parts(primitive, vao.name(), ebo.name(), ebo.len() as u32) }
        }
    }
}
