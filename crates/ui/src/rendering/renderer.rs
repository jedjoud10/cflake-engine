use crate::{InstancedBatch, InstancedBatchIdentifier, Root};
use rendering::{
    basics::{
        shader::{Shader, ShaderSettings},
        uniforms::{ShaderUniformsGroup, ShaderUniformsSettings},
    },
    object::ObjectID,
    pipeline::{pipec, Pipeline},
};
use std::{collections::HashMap, ptr::null};
use veclib::Swizzable;

// The two default UI shaders that we will use as fallback
pub const DEFAULT_UI_SHADER_VERT: &str = "defaults\\shaders\\ui\\panel.vrsh.glsl";
pub const DEFAULT_UI_SHADER_FRAG: &str = "defaults\\shaders\\ui\\panel.frsh.glsl";

// The renderer that will render our UI
// We can create the renderer on the main thread, then send it to the render thread
pub struct Renderer {
    // Instanced batches used for rendering
    pub batches: HashMap<InstancedBatchIdentifier, InstancedBatch>,
    // The default UI shader that we must use whenever we don't speciy a shader
    pub default_shader: ObjectID<Shader>,
}

impl Renderer {
    // Create a new UI renderer
    pub fn new(pipeline: &Pipeline) -> Self {
        // Load the default UI shader
        let settings = ShaderSettings::default().source(DEFAULT_UI_SHADER_VERT).source(DEFAULT_UI_SHADER_FRAG);
        let shader = Shader::new(settings).unwrap();
        let shader = pipec::construct(shader, pipeline);
        Self {
            batches: HashMap::with_capacity(1),
            default_shader: shader,
        }
    }
    // Draw all the elements that are part of the root
    // We must run this one the render thread
    pub fn draw(&mut self, pipeline: &mut Pipeline, root: &mut Root, window_size: veclib::Vector2<u16>) {
        let window_size: veclib::Vector2<f32> = window_size.into();
        // Get the elements that we have added and add them
        let added = std::mem::take(&mut root.added);
        let max_depth = root.calculate_max_depth();
        for element_id in added.iter() {
            let element = root.get_element(*element_id).unwrap();
            // Add the element to the respective batch
            let identifier = InstancedBatchIdentifier {
                shader: element.shader,
                texture: element.texture,
            };
            let batch = self.batches.entry(identifier).or_insert(InstancedBatch::new());
            // Add the per instance parameters now
            // We will all the default values for these, since we're going to be updating them in a later step anyways
            batch.depth_buf.push(0.0);
            batch.colors_buf.push(element.color);
            batch.screen_verts_center_and_size_buf.push(Default::default());
            batch.screen_verts_center_and_size_buf.push(Default::default());
            for _ in 0..4 {
                batch.texture_uvs_buf.push(Default::default());
            }
            batch.instance_count += 1;
        }
        // Remove
        let removed = std::mem::take(&mut root.removed);
        for (_, batch_id) in removed {
            // Remove the elements from the buffers
            let batch = self.batches.get_mut(&batch_id).unwrap();
            // Now we can remove the value from the buffers
            // Since we will be updating the buffers every time, we don't care that we remove a wrong element, as long as we just remove an element
            batch.depth_buf.pop();
            batch.colors_buf.pop();
            batch.screen_verts_center_and_size_buf.pop();
            batch.screen_verts_center_and_size_buf.pop();
            for _ in 0..4 {
                batch.texture_uvs_buf.pop();
            }
            batch.instance_count -= 1;
        }
        // We gotta update the depth and screen_uvs values for every element, even if we didn't mutate it
        let resolution: veclib::Vector2<f32> = pipeline.window.dimensions.into();
        for (index, (_, element)) in root.elements.iter().enumerate() {
            let identifier = InstancedBatchIdentifier {
                shader: element.shader,
                texture: element.texture,
            };
            let batch = self.batches.get_mut(&identifier).unwrap();
            let center: veclib::Vector2<f32> = element.center.into();
            let size: veclib::Vector2<f32> = element.size.into();
            // I am a masochist
            batch.screen_verts_center_and_size_buf.update(index * 4 + 0, |x| *x = center / resolution);
            batch.screen_verts_center_and_size_buf.update(index * 4 + 1, |x| *x = size / resolution);
            /*
            batch.texture_uvs_buf.update(index * 4 + 0, |x| *x = element.texture_uvs.get2([0, 1]));
            batch.texture_uvs_buf.update(index * 4 + 1, |x| *x = element.texture_uvs.get2([0, 3]));
            batch.texture_uvs_buf.update(index * 4 + 2, |x| *x = element.texture_uvs.get2([2, 1]));
            batch.texture_uvs_buf.update(index * 4 + 3, |x| *x = element.texture_uvs.get2([2, 3]));
            */
            batch.depth_buf.update(index, |x| *x = (element.depth as f32 * 0.995) / max_depth as f32);

            // Update some values if needed
            batch.colors_buf.update(index, |x| *x = element.color);
        }

        // Now we can actually draw the elements as multiple instanced batches
        unsafe {
            gl::Disable(gl::CULL_FACE);
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
            gl::Clear(gl::DEPTH_BUFFER_BIT);
        }

        for (id, batch) in self.batches.iter() {
            // Get the shader, create some uniforms, then draw
            let mut group = ShaderUniformsGroup::default();
            let id_shader = if !id.shader.is_some() { self.default_shader } else { id.shader };
            let id_texture = if !id.texture.is_some() { pipeline.defaults.as_ref().unwrap().white } else { id.texture };
            // If the shader ID is the default one, that means that we have to use the default UI shader
            group.set_texture("main_texture", id_texture, 0);
            if pipeline.get_shader(id_shader).is_some() {
                let settings = ShaderUniformsSettings::new(id_shader);
                group.execute(pipeline, settings).expect("Forgot to set shader or main texture!");

                unsafe {
                    //println!("{:?}", batch);
                    gl::BindVertexArray(batch.vao);
                    gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, batch.ebo);
                    gl::DrawElementsInstanced(gl::TRIANGLES, 6, gl::UNSIGNED_INT, null(), batch.instance_count as i32);
                }
            }
        }
    }
}
