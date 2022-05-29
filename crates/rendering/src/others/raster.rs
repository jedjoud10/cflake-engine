use std::{ptr::null, rc::Rc};
use crate::{mesh::{SubMesh, attributes::AttributeSet}, object::ToGlName, canvas::Canvas, shader::Shader, buffer::ElementBuffer, context::Context, blend::BlendMode};

// How rasterized triangles should be culled
#[derive(PartialEq)]
pub enum FaceCullMode {
    // The boolean specifies if the culling should be Counter Clockwise
    Front(bool), Back(bool),
    
    // Don't cull anything
    None,
}

// Main rasterizer settings like sissor tests and depth tests
pub struct RasterSettings {
    // Should we check for vertex depth when rasteizing?
    depth_test: bool,

    // A sissor test basically limits the area of effect when rasterizing. Pretty useful for UI
    sissor_test: Option<vek::Aabr<i32>>,

    // The current primitive that we will render with. Currently supported: Triangles and Points
    primitive: PrimitiveMode,

    // Should we render in SRGB or not?
    srgb: bool,

    // Transparancy blending mode
    blend: Option<BlendMode>
}

// Depicts the exact primitives we will use to draw the mesh
#[derive(PartialEq)]
pub enum PrimitiveMode {
    Triangles {
        cull: FaceCullMode,
    }, Points {
        diameter: f32
    }
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
    // The canvas we will be rasterizing onto
    pub(super) canvas: &'canvas mut Canvas,

    // The unique shader that we are using to rasterize our primitives
    pub(super) shader: &'shader mut Shader,

    pub(super) context: &'context mut Context,
}

impl<'canvas, 'shader, 'context> Rasterizer<'canvas, 'shader, 'context> {
    // Bind the shader and raster mode to the OpenGL context, and return the raw primitive type
    fn prepare(&mut self, settings: RasterSettings) -> u32 {
        // Bind the shader first
        self.context.bind(gl::PROGRAM, self.shader.as_ref().name(), |name| unsafe { gl::UseProgram(name) });

        // Get the primitive type for rasterization
        let primitive = match &settings.primitive {
            PrimitiveMode::Triangles { .. } => gl::TRIANGLES,
            PrimitiveMode::Points { .. } => gl::POINTS,
        };

        // Set the proper primitive settings
        // Set the global OpenGL face culling mode
        unsafe fn set_cull_mode(mode: FaceCullMode) {
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
            gl::FrontFace(if ccw { gl::CCW } else { gl::CW });
        }

        // Set the global OpenGL point size
        unsafe fn set_point_size(diameter: f32) {
            gl::PointSize(diameter);
        }
        
        // Set the primitive's settings (only if there was a change in primtive type tho)
        let primitive = if self.context.raster.primitive != settings.primitive {
            // Update the context state and OpenGL state
            self.context.raster.primitive = settings.primitive;
            match settings.primitive {
                PrimitiveMode::Triangles { cull } => unsafe { set_cull_mode(cull); gl::TRIANGLES },
                PrimitiveMode::Points { diameter } => unsafe { set_point_size(diameter); gl::POINTS },
            }
        } else { self.context.raster.primitive };
        
            
        primitive
    }

    // Rasterize the raw VAO an EBO without setting the mode or binding the shader
    unsafe fn draw_from_raw_parts(&mut self, primitive: u32, vao: u32, ebo: u32, count: u32) {
        gl::BindVertexArray(vao);
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
        gl::DrawElements(primitive, count as i32, gl::UNSIGNED_INT, null());
    }

    // Draw a single VAO and a EBO using their raw OpenGL names directly
    pub unsafe fn draw_unchecked(&mut self, vao: u32, ebo: u32, count: u32, mode: PrimitiveMode) {
        // Bind the shader/raster modes to the context
        let primitive = self.prepare(mode);

        // Draw the VAO and EBO
        self.draw_from_raw_parts(primitive, vao, ebo, count);
    }

    // Draw a single VAO and EBO
    pub fn draw<T: ToRasterBuffers>(&mut self, obj: T, mode: PrimitiveMode) {
        // Bind the shader/raster modes to the context
        let primitive = self.prepare(mode);

        // Get the raw OpenGL names
        let vao = obj.vao();
        let ebo = obj.ebo();

        unsafe {
            self.draw_from_raw_parts(primitive, vao.name(), ebo.name(), ebo.len() as u32)
        }
    }

    // This will draw a set of VAOs and EBOs directly onto the screen
    pub fn draw_batch<T: ToRasterBuffers>(&mut self, objects: &[&T], mode: PrimitiveMode) {
        // Bind the shader/raster modes to the context
        let primitive = self.prepare(mode);

        // Iterate through each object and draw it
        for object in objects {
            // Get the raw OpenGL names
            let vao = object.vao();
            let ebo = object.ebo();

            unsafe {
                self.draw_from_raw_parts(primitive, vao.name(), ebo.name(), ebo.len() as u32)
            }
        }
    }
}