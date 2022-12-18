use assets::Asset;

// This trait is implemented for each shader stage, like the vertex stage or fragment stage
pub trait Module: Sized {
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
        impl Module for $t {
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

        /*
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
        */
    };
}

impl_stage_traits!(VertexStage, gl::VERTEX_SHADER, "vrsh.glsl");
impl_stage_traits!(FragmentStage, gl::FRAGMENT_SHADER, "frsh.glsl");
impl_stage_traits!(ComputeStage, gl::COMPUTE_SHADER, "cmpt.glsl");