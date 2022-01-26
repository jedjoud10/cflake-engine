use ecs::component::defaults::*;
use ecs::component::*;

use main::core::{Context, WriteContext};
use main::ecs;
use main::rendering::advanced::compute::{ComputeShader, ComputeShaderExecutionSettings};
use main::rendering::basics::shader::ShaderSettings;
use main::rendering::basics::texture::{Texture, TextureType, TextureFormat, TextureFilter, TextureWrapping, TextureReadBytes, TextureAccessType};
use main::rendering::basics::uniforms::ShaderUniformsGroup;
use main::rendering::object::{ObjectID, PipelineTrackedTask, TrackedTaskID};
use main::rendering::pipeline::pipec;
use main::rendering::utils::DataType;

// Some global data for the test system
pub(crate) struct TestSystemData {
    pub compute_shader: ObjectID<ComputeShader>,
    pub texture: ObjectID<Texture>,
    pub generating: bool,
    pub tracker: TrackedTaskID,
    pub reader: Option<TextureReadBytes>,
}

ecs::impl_component!(TestSystemData);

// A simple system that we can use for testing
fn run(mut context: Context, query: ComponentQuery) {
    let mut write = context.write();
    let mut global = write.ecs.global_mut::<TestSystemData>().unwrap();
    let pipeline = write.pipeline.read().unwrap();

    // Update the texture every time
    if !global.generating {
        println!("Start: {}", write.time.frame_count);
        let es = ComputeShaderExecutionSettings::new(global.compute_shader, (64, 64, 64));
        let mut group = ShaderUniformsGroup::new();
        group.set_image("output_img", global.texture, TextureAccessType::WRITE);
        
        let es = es.set_uniforms(group);
        let partial = pipec::tracked_task(PipelineTrackedTask::RunComputeShader(global.compute_shader, es), None, &*pipeline); 
        let read = TextureReadBytes::default();
        let partial2 = pipec::tracked_task(PipelineTrackedTask::TextureReadBytes(global.texture, read.clone()), Some(partial), &*pipeline); 
        let full = pipec::tracked_finalizer(vec![partial, partial2], &*pipeline).unwrap();
        global.tracker = full;
        global.generating = true;
        global.reader = Some(read);
    } else {
        println!("Check: {}", write.time.frame_count);
        global.generating = !pipec::has_task_executed(global.tracker, &*pipeline).unwrap();
        // Finished executing
        if !global.generating {
            println!("Finish: {}", write.time.frame_count);
            let test = global.reader.take().unwrap().fill_vec::<(u8, u8, u8, u8)>().unwrap();
            dbg!(test.len());
        }
    }
}

// Create the system
pub fn system(write: &mut WriteContext) {
    write.ecs.create_system_builder().set_run_event(run).build();

    // Load the compute shader
    let pipeline = write.pipeline.read().unwrap();
    let ss = ShaderSettings::default().source("defaults\\shaders\\others\\template_compute.cmpt.glsl");
    let compute_shader = ComputeShader::new(ss).unwrap();
    let compute_shader = pipec::construct(compute_shader, &*pipeline);

    // Create le texture
    let texture = Texture::default()
        .set_dimensions(TextureType::Texture3D(64, 64, 64))
        .set_format(TextureFormat::RGBA8R)
        .set_data_type(DataType::U8)
        .set_filter(TextureFilter::Nearest)
        .set_wrapping_mode(TextureWrapping::ClampToBorder);
    let texture = pipec::construct(texture, &*pipeline);
    drop(pipeline);
    write.ecs.add_global(TestSystemData {
        compute_shader,
        texture,
        generating: false,
        tracker: TrackedTaskID::new(true),
        reader: None,
    }).unwrap();
}
