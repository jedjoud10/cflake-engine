use crate::{
    context::Context,
    object::{ToGlName, ToGlTarget},
};
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

// A vertex stage that will be loaded from .vrsh files
pub struct VertexStage {
    source: String,
    name: String,
}
// A fragment stage that will be loaded from .frsh files
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
                Self {
                    source,
                    name
                }
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
                    name: data.path().file_name().unwrap().to_str().unwrap().to_string(),
                }
            }
        }
    };
}

impl_stage_traits!(VertexStage, gl::VERTEX_SHADER, "vrsh.glsl");
impl_stage_traits!(FragmentStage, gl::FRAGMENT_SHADER, "frsh.glsl");
impl_stage_traits!(ComputeStage, gl::COMPUTE_SHADER, "cmpt.glsl");

// This implies that the source code for the underlying stage has been filtered and is ready for compliation
pub(super) struct Processed<T: Stage>(pub(super) T);

// This implies that the underlying shader source has been compiled
pub(super) struct Compiled<T: Stage>(PhantomData<T>, u32);

impl<T: Stage> ToGlName for Compiled<T> {
    fn name(&self) -> u32 {
        self.1
    }
}

impl<T: Stage> Drop for Compiled<T> {
    fn drop(&mut self) {
        // Automatically delete the stage shader after we successfully use it
        unsafe { gl::DeleteShader(self.1) }
    }
}

// Compile a single shader stage, and handle errors
pub(super) unsafe fn compile<U: Stage>(_ctx: &mut Context, stage: Processed<U>) -> Compiled<U> {
    // Create the stage source
    let stage = stage.0;
    let shader = gl::CreateShader(U::target());
    let (source, name) = stage.into_raw_parts();

    // Specify the stage source and compile it
    let cstring = CString::new(source.clone()).unwrap();
    let shader_source: *const i8 = cstring.as_ptr();
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
            gl::GetShaderInfoLog(
                shader,
                len,
                null_mut(),
                vec.as_mut_ptr() as _,
            );
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

    // Return the stage GL name
    println!("Compiled shader source {name} successfully");
    Compiled(Default::default(), shader)
}
