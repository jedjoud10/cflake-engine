use ash::vk;
use assets::Asset;
use std::{
    ffi::CString,
    marker::PhantomData,
    ptr::{null, null_mut},
};

use crate::Graphics;

// This trait is implemented for each shader stage, like the vertex stage or fragment stage
pub trait Stage: Sized {
    // Main functions to get name and source code
    fn name(&self) -> &str;
    fn source(&self) -> &str;
    fn into_raw_parts(self) -> (String, String);
    fn from_raw_parts(source: String, name: String) -> Self;

    // Get the type of stage for Vulkan
    fn kind() -> vk::ShaderStageFlags;
}

// A vertex stage that will be loaded from .vrtx files
#[derive(Clone)]
pub struct VertexStage {
    source: String,
    name: String,
}

#[derive(Clone)]
// A fragment stage that will be loaded from .frag files
pub struct FragmentStage {
    source: String,
    name: String,
}

#[derive(Clone)]
// A compute stage (only for compute shaders) that will be loaded from .cmpt files
pub struct ComputeStage {
    source: String,
    name: String,
}

// I love procedural programming
macro_rules! impl_stage_traits {
    ($t: ty, $type: ident, $ext: expr) => {
        impl Stage for $t {
            fn name(&self) -> &str {
                &self.name
            }

            fn source(&self) -> &str {
                &self.source
            }

            fn into_raw_parts(self) -> (String, String) {
                (self.source, self.name)
            }

            fn from_raw_parts(source: String, name: String) -> Self {
                Self { source, name }
            }

            fn kind() -> vk::ShaderStageFlags {
                vk::ShaderStageFlags::$type
            }
        }

        impl Asset for $t {
            type Args<'args> = ();

            fn extensions() -> &'static [&'static str] {
                &[$ext]
            }

            fn deserialize(
                data: assets::Data,
                _args: Self::Args<'_>,
            ) -> Self {
                Self {
                    source: String::from_utf8(data.bytes().to_vec())
                        .unwrap(),
                    name: data
                        .path()
                        .file_name()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .to_string(),
                }
            }
        }
    };
}

impl_stage_traits!(VertexStage, VERTEX, "vrtx.glsl");
impl_stage_traits!(FragmentStage, FRAGMENT, "frag.glsl");
impl_stage_traits!(ComputeStage, COMPUTE, "cmpt.glsl");

// This implies that the source code for the underlying stage has been filtered and is ready for compliation
pub(super) struct Processed<T: Stage> {
    bytecode: Vec<u8>,
    _phantom: PhantomData<T>,
}

// This hints that the underlying shader module has been compiled
pub(super) struct Compiled<T: Stage> {
    module: vk::ShaderModule,
    _phantom: PhantomData<T>,
}

// Compile GLSL code to SPIRV at runtime
pub(super) unsafe fn compile_spirv() -> Vec<u8> {
    todo!()
}

// Compile a SPIRV shader into a proper shader module
pub(super) unsafe fn compile_module<U: Stage>(
    graphics: &Graphics,
    stage: Processed<U>,
) -> Compiled<U> {
    todo!()
}
