use vulkan::{vk, Device};

// Compile GLSL data into SPIRV byte code
pub unsafe fn translate_glsl_spirv(device: &Device, code: &str) -> Vec<u32> {
    todo!()
}

// Compile an actual SPIRV module into a Vulkan module
pub unsafe fn compile_shader_module(device: &Device, bytecode: Vec<u32>) -> vk::ShaderModule {
    device.compile_shader_module(&bytecode)
}