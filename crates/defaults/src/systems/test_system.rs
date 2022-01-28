use ecs::component::*;
use main::core::{Context, WriteContext};
use main::ecs;
use main::rendering::advanced::compute::{ComputeShader, ComputeShaderExecutionSettings};
use main::rendering::advanced::shaderstorage::ShaderStorage;
use main::rendering::basics::readwrite::ReadBytes;
use main::rendering::basics::shader::ShaderSettings;
use main::rendering::basics::transfer::Transferable;
use main::rendering::basics::uniforms::ShaderUniformsGroup;
use main::rendering::object::{ObjectID, PipelineTrackedTask, TrackedTaskID};
use main::rendering::pipeline::pipec;
use main::rendering::utils::AccessType::Read;
use main::rendering::utils::UpdateFrequency;

// Some global data for the test system
pub(crate) struct TestSystemData {
    shader_storage: ObjectID<ShaderStorage>,
    compute_shader: ObjectID<ComputeShader>,
    transfer: Option<ReadBytes>,
    finalized: TrackedTaskID,
}
ecs::impl_component!(TestSystemData);

// A simple system that we can use for testing
fn run(mut context: Context, query: ComponentQuery) {
    let mut write = context.write();
    // Execute the shader
    let mut data = write.ecs.global_mut::<TestSystemData>().unwrap();
    let pipeline = write.pipeline.read().unwrap();
    if data.transfer.is_none() && pipeline.get_shader_storage(data.shader_storage).is_some() {
        
        // Make the shader group
        let mut group = ShaderUniformsGroup::new();
        group.set_shader_storage("_", data.shader_storage, 0);
        
        let settings = ComputeShaderExecutionSettings::new(data.compute_shader, (4, 1, 1)).set_uniforms(group);
        let compute = pipec::tracked_task(PipelineTrackedTask::RunComputeShader(data.compute_shader, settings), None, &*pipeline);
        let read = ReadBytes::default();
        let transfer = read.transfer();
        data.transfer = Some(read);
        let read = pipec::tracked_task(PipelineTrackedTask::ShaderStorageReadBytes(data.shader_storage, transfer), Some(compute), &*pipeline);
        let finalized = pipec::tracked_finalizer(vec![compute, read], &*pipeline).unwrap();
        data.finalized = finalized;
    } else {
        if pipec::has_task_executed(data.finalized, &*pipeline).unwrap_or_default() {
            let taken = data.transfer.take().unwrap();
            // Read the bytes as a slice
            let read = taken.fill_vec::<i32>().unwrap();
            println!("{:?}", read);
        }
    }
}
    
// Create the system
pub fn system(write: &mut WriteContext) {
    write.ecs.create_system_builder().set_run_event(run).build();

    let pipeline = write.pipeline.read().unwrap();
    let arr = [1, 5, 2, -9];
    let shader_storage = pipec::construct(ShaderStorage::new_default(UpdateFrequency::Static, Read, arr, 4 * 4), &*pipeline);
    
    let ss = ShaderSettings::default().source("defaults\\shaders\\others\\template_compute.cmpt.glsl");
    let compute_shader = ComputeShader::new(ss).unwrap();
    let compute_shader = pipec::construct(compute_shader, &*pipeline);
    
    
    drop(pipeline);
    write.ecs.add_global(TestSystemData {
        shader_storage,
        compute_shader,
        transfer: None,
        finalized: TrackedTaskID::default(),
    }).unwrap();
}
