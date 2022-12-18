use vulkan::{vk, Device};

// Compile GLSL data into SPIRV byte code
pub unsafe fn translate_glsl_spirv(device: &Device, file_name: &str, code: &str) -> Vec<u32> {
    todo!()
    //device.translate_glsl_spirv(code, file_name, entry_point, kind)
}

// Compile an actual SPIRV module into a Vulkan module
pub unsafe fn compile_shader_module(device: &Device, bytecode: Vec<u32>) -> vk::ShaderModule {
    device.compile_shader_module(&bytecode)
}