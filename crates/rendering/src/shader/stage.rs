use std::{num::NonZeroU32, ffi::CString, ptr::{null, null_mut}};
use assets::Asset;
use crate::context::{Context, ToGlType};


// This trait is implemented for each shader stage, like the vertex stage or fragment stage
pub trait Stage: Sized + From<String> + Into<String> + AsRef<str> + ToGlType {
    // The file extension for this stage
    fn extension() -> &'static str;
}

// A vertex stage that will be loaded from .vrsh files
pub struct VertexStage(String);
// A fragment stage that will be loaded from .frsh files
pub struct FragmentStage(String);
// A compute stage (only for compute shaders) that will be loaded from .cmpt files
pub struct ComputeStage(String);

// I love procedural programming
macro_rules! impl_stage_traits {
    ($t: ty, $gl: expr, $ext: expr) => {
        
        impl From<String> for $t {
            fn from(s: String) -> Self {
                Self(s)
            }
        }

        impl Into<String> for $t {
            fn into(self) -> String {
                self.0
            }
        }

        impl AsRef<str> for $t {
            fn as_ref(&self) -> &str {
                &self.0
            }
        }

        impl ToGlType for $t {
            fn target(&self) -> u32 {
                $gl
            }
        }

        impl Stage for $t {
        }

        impl Asset<'static> for $t {
            type Args = ();
        
            fn extensions() -> &'static [&'static str] {
                &[$ext]
            }
        
            fn deserialize(bytes: assets::loader::CachedSlice, args: Self::Args) -> Self {
                Self::from(String::from_utf8(bytes.as_ref().to_vec()).unwrap())
            }
        }
    };
}

impl_stage_traits!(VertexStage, gl::VERTEX_SHADER, ".vrsh.glsl");
impl_stage_traits!(FragmentStage, gl::FRAGMENT_SHADER, ".frsh.glsl");
impl_stage_traits!(ComputeStage, gl::COMPUTE_SHADER, ".cmpt.glsl");


// Compile a single shader stage, and handle errors
pub unsafe fn compile<S: Stage>(ctx: &mut Context, stage: S) -> NonZeroU32 {
    // Create the stage source
    let program = gl::CreateShader(stage.target());

    // Specify the stage source and compile it
    let source: String = stage.into();
    let cstring = CString::new(source.clone()).unwrap();
    let shader_source: *const i8 = cstring.as_ptr();
    gl::ShaderSource(program, 1, &shader_source, null());
    gl::CompileShader(program);

    // Check for any errors
    let mut len: i32 = 0;
    gl::GetShaderiv(program, gl::INFO_LOG_LENGTH, &mut len);
    if len > 0 {
        // Create a string that will contain the error message
        let message = String::from_utf8({
            let mut vec = Vec::with_capacity(len as usize + 1);
            gl::GetShaderInfoLog(program, len, null_mut(), vec.spare_capacity_mut().as_mut_ptr() as _);
            vec
        }).unwrap();

        // Get the source code for this stage, and identify each line with it's line out
        let source = source.lines().enumerate().map(|(count, line)| format!("({}): {}", count + 1, line)).collect::<Vec<String>>().join("\n");
        
        // Print the error message
        panic!("Source: \n{}\n Error: \n{}", source, message);
    }

    // Return the program GL name
    NonZeroU32::new(program).unwrap()
} 