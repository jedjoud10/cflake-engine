use crate::context::{Context, ToGlName, ToGlTarget};
use assets::Asset;
use std::{
    ffi::CString,
    marker::PhantomData,
    ptr::{null, null_mut},
};

// This trait is implemented for each shader stage, like the vertex stage or fragment stage
pub trait Stage: Sized + ToGlTarget {
    // Get the file name of the stage
    fn name(&self) -> &str;

    // Get the source code of the stage
    fn source(&self) -> &str;

    // Convert the stage into it's source code and name
    fn into_raw_parts(self) -> (String, String);

    // Convert some source code and a file name into a stage
    fn from_raw_parts(source: String, name: String) -> Self;
}

// A vertex stage that will be loaded from .vrtx files
pub struct VertexStage {
    source: String,
    name: String,
}
// A fragment stage that will be loaded from .frag files
pub struct FragmentStage {
    source: String,
    name: String,
}
// A compute stage (only for compute shaders) that will be loaded from .cmpt files
pub struct ComputeStage {
    source: String,
    name: String,
}

// I love procedural programming
macro_rules! impl_stage_traits {
    ($t: ty, $gl: expr, $ext: expr) => {
        impl ToGlTarget for $t {
            fn target() -> u32 {
                $gl
            }
        }

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
        }

        impl Asset<'static> for $t {
            type Args = ();

            fn extensions() -> &'static [&'static str] {
                &[$ext]
            }

            fn deserialize(data: assets::Data, _args: Self::Args) -> Self {
                Self {
                    source: String::from_utf8(data.bytes().to_vec()).unwrap(),
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

impl_stage_traits!(VertexStage, gl::VERTEX_SHADER, "vrtx.glsl");
impl_stage_traits!(FragmentStage, gl::FRAGMENT_SHADER, "frag.glsl");
impl_stage_traits!(ComputeStage, gl::COMPUTE_SHADER, "cmpt.glsl");

// This implies that the source code for the underlying stage has been filtered and is ready for compliation
pub(super) struct Processed<T: Stage>(pub(super) T);

// This hints that the underlying shader source has been compiled
pub(super) struct Compiled<T: Stage> {
    name: u32,
    _phantom: PhantomData<T>,
}

impl<T: Stage> ToGlName for Compiled<T> {
    fn name(&self) -> u32 {
        self.name
    }
}

impl<T: Stage> Drop for Compiled<T> {
    fn drop(&mut self) {
        // Automatically delete the stage shader after we use it
        unsafe { gl::DeleteShader(self.name) }
    }
}

// Compile a single shader stage, and handle errors
pub(super) unsafe fn compile<U: Stage>(ctx: &mut Context, stage: Processed<U>) -> Compiled<U> {
    // Create the stage source
    let stage = stage.0;
    let (source, name) = stage.into_raw_parts();

    // If the stage was already compiled, simply just reuse it
    if ctx.stages.0.contains(&name) {
        let hash = crc32fast::hash(source.as_bytes());
        println!("Reused shader source {name}");
        let name = *ctx.stages.1.get(&hash).unwrap();
        return Compiled {
            name,
            _phantom: PhantomData,
        };
    }
    
    // Specify the stage source and compile it
    let cstring = CString::new(source.clone()).unwrap();
    let shader_source: *const i8 = cstring.as_ptr();
    let shader = gl::CreateShader(U::target());
    gl::ShaderSource(shader, 1, &shader_source, null());
    gl::CompileShader(shader);

    // Check for errors
    let mut len: i32 = 0;
    gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
    if len > 0 {
        println!("{}", len);
        // Create a string that will contain the error message
        let message = String::from_utf8({
            let mut vec = vec![0; len as usize + 1];
            gl::GetShaderInfoLog(shader, len, null_mut(), vec.as_mut_ptr() as _);
            vec
        })
        .unwrap();

        // Get the source code for this stage, and identify each line with it's line out
        let source = source
            .lines()
            .enumerate()
            .map(|(count, line)| format!("({}): {}", count + 1, line))
            .collect::<Vec<String>>()
            .join("\n");

        // Print the error message
        panic!("Source: \n{}\n Error: \n{}", source, message);
    }

    // Cache the shader source for later reusage
    let hash = crc32fast::hash(source.as_bytes());
    println!("Compiled shader source {name} successfully");
    ctx.stages.0.insert(name);
    ctx.stages.1.insert(hash, shader);
    
    Compiled {
        name: shader,
        _phantom: Default::default(),
    }
}
