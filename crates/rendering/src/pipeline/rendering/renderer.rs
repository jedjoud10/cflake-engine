use std::ffi::c_void;

use super::{common, RenderingSettings, ShadowMapping, cull_frustum, SceneRenderStats};
use crate::{
    basics::{
        mesh::{Mesh, Vertices},
        shader::{Directive, Shader, ShaderInitSettings},
        texture::{ResizableTexture, Texture2D, TextureFilter, TextureFlags, TextureFormat, TextureLayout, TextureParams, TextureWrapMode, CubeMap},
        uniforms::Uniforms,
    },
    pipeline::{Framebuffer, FramebufferClearBits, Handle, Pipeline},
    utils::{DataType, DEFAULT_WINDOW_SIZE},
};
use getset::{Getters, MutGetters};


// Scene renderer that utilizes clustered-forward rendering to render the scene
#[derive(Getters, MutGetters)]
#[getset(get = "pub", get_mut = "pub")]
pub struct SceneRenderer {
    // Access to the default framebuffer
    framebuffer: Framebuffer,
}

impl SceneRenderer {
    // Initialize a new scene renderer
    pub(crate) unsafe fn new(pipeline: &mut Pipeline) -> Self {
        println!("Initializing the scene renderer...");
        println!("Successfully initialized the RenderPipeline Renderer!");
        Self {
            framebuffer: Framebuffer::default(pipeline),
        }
    }
    // Resize the renderer's textures
    pub(crate) fn resize(&mut self, pipeline: &mut Pipeline) {
        // Very simple since we use an array
        let dimensions = pipeline.window().dimensions();
        
        // Resize the default framebuffer
        self.framebuffer.bind(|mut fb| fb.viewport(dimensions))
    }

    // Initialize OpenGL at the start of the application
    pub(crate) unsafe fn init_opengl() {
        gl::ClearColor(0.0, 0.0, 0.0, 1.0);
        gl::Enable(gl::DEPTH_TEST);
        gl::Enable(gl::CULL_FACE);
        gl::CullFace(gl::BACK);
    }

    // Clear the default framebuffer and set it's size properly
    pub(crate) fn start_frame(&mut self, pipeline: &mut Pipeline) {
        self.framebuffer.bind(|mut f| {
            f.viewport(pipeline.window().dimensions());
            f.clear(FramebufferClearBits::COLOR | FramebufferClearBits::DEPTH);
        });
    }

    // Render the whole scene using "clustered-forward" as the rendering method
    pub fn render(&mut self, pipeline: &Pipeline, mut settings: RenderingSettings) {
        // Scene statistics for the debugger
        let mut stats = SceneRenderStats { drawn: 0, culled: 0, shadowed: 0 };

        // Bind the deferred renderer's framebuffer
        self.framebuffer.bind(|_| {
            // AABB frustum culling cause I'm cool
            let taken = std::mem::take(&mut settings.normal);
            let objects = cull_frustum(pipeline.camera(), taken, &mut stats);

            // Render each object that isn't culled
            for renderer in objects {
                common::render_model(renderer, pipeline)
            }
        });

        // Then render the shadows
        /*
        if let Some(mapping) = &mut self.shadow_mapping {
            unsafe {
                // Update the lightspace matrix
                // The first directional light that we find will be used as the sunlight
                let first = settings.lights.iter().find_map(|(_type, params)| _type.as_directional().map(|_type| (_type, params)));

                if let Some((_parameters, transform)) = first {
                    // No need to update if nothing has changed
                    if settings.redraw_shadows {
                        // Only render directional shadow map if we have a sun
                        mapping.update_matrix(*transform.rotation);
                        // Then render shadows
                        mapping.render_all_shadows(&settings.shadowed, pipeline, &mut stats);
                    }
                }
            }
        }
        */

        // Store the stats in the pipeline
        *pipeline.stats().borrow_mut() = stats;
    }

    // Screenshot the current frame
    // This must be done after we render everything
    pub fn screenshot(&mut self, dimensions: vek::Extent2<u32>) -> Vec<u8> {
        // Create a vector that'll hod all of our RGB bytes
        let bytes_num = dimensions.as_::<usize>().product() * 3;
        let mut bytes = vec![0; bytes_num];
        // Read
        unsafe {
            gl::ReadPixels(
                0,
                0,
                dimensions.w as i32,
                dimensions.h as i32,
                gl::RGB,
                gl::UNSIGNED_BYTE,
                bytes.as_mut_ptr() as *mut c_void,
            );
            gl::Finish();
        }
        bytes
    }
}
