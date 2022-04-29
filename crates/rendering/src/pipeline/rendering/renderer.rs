use std::ffi::c_void;

use super::{common, RenderingSettings, ShadowMapping, cull_frustum, SceneRenderStats, RenderedModel, Sun, render};
use crate::{
    basics::{
        mesh::{Mesh, Vertices},
        shader::{Directive, Shader, ShaderInitSettings},
        texture::{ResizableTexture, Texture2D, TextureFilter, TextureFlags, TextureFormat, TextureLayout, TextureParams, TextureWrapMode, CubeMap},
        uniforms::Uniforms, lights::{LightParameters, LightTransform},
    },
    pipeline::{Framebuffer, FramebufferClearBits, Handle, Pipeline},
    utils::{DataType, DEFAULT_WINDOW_SIZE},
};
use getset::{Getters, MutGetters};
use math::bounds::aabb::AABB;


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

    // Try to get the main directional light, and fallbacks to default values if it failed
    fn get_sun<'a>(settings: &'a RenderingSettings) -> Sun {
        // Get the sun directional light, if possible
        let first = settings
            .lights
            .iter()
            .find_map(|(_type, params)| 
                _type.as_directional().map(|_type| (_type, params))
            );
        
        // Get the relevant info
        first.map(|(params, transform)| {
            // Calculate the direction
            let dir = vek::Mat4::from(*transform.rotation).mul_direction(-vek::Vec3::unit_z());
            
            // Passthrough
            let color = params.color;

            // Construction time
            Sun {
                dir,
                color,
            }
        }).unwrap_or_default()
    }

    // Render the whole scene using "clustered-forward" as the rendering method
    pub fn render(&mut self, pipeline: &Pipeline, mut settings: RenderingSettings) {
        // Scene statistics for the debugger
        let mut stats = SceneRenderStats { drawn: 0, culled: 0, shadowed: 0 };

        // Le sun moment
        let sun = Self::get_sun(&settings);
        
        // We should bind the default framebuffer just in case
        self.framebuffer.bind(|_| {
            // AABB frustum culling cause I'm cool
            let taken = std::mem::take(&mut settings.normal);
            let objects = cull_frustum(pipeline.camera(), taken, &mut stats);

            // Render each object that isn't culled
            for renderer in objects {
                // Load the default missing material if we don't have a valid one
                let handle = renderer.material.fallback_to(&pipeline.defaults().missing_pbr_mat);
                let material = pipeline.get(handle).unwrap();
                        
                // However, if we have an invalid shader, we must panic
                let shader = pipeline.get(material.shader().as_ref().unwrap()).unwrap();
                let mesh = pipeline.get(renderer.mesh).unwrap();
                        
                // Create some uniforms
                Uniforms::new(shader.program(), pipeline, |mut uniforms| {
                    // Set the PBR snippet values
                    uniforms.set_vec3f32("_sun_dir", sun.dir);
                    uniforms.set_vec3f32("_sun_intensity", sun.color.into());

                    // Set the model snippet uniforms
                    uniforms.set_mat44f32("_model_matrix", renderer.matrix);
                
                    // Execute the material 
                    material.execute(pipeline, uniforms);
                });
            
                // Finally render the mesh
                unsafe {
                    render(mesh);
                }
            }
        });

        // Store the stats in the pipeline
        *pipeline.stats().borrow_mut() = stats;
    }

    // Screenshot the current frame
    // This must be done after we render everything
    pub fn screenshot(&mut self, dimensions: vek::Extent2<u32>) -> Vec<u8> {
        // Create a vector thats shall hold all of our RGB bytes
        let bytes_num = dimensions.as_::<usize>().product() * 3;
        let mut bytes = vec![0; bytes_num];
        self.framebuffer.bind(|fb| {
            // Read from OpenGL (slow!)
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
        });        
        bytes
    }
}
