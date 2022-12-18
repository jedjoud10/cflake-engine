use assets::Asset;

// This trait is implemented for each shader module, like the vertex module or fragment module
pub trait Module: Sized {
    // Get the file name of the module
    fn name(&self) -> &str;

    // Get the source code of the module
    fn source(&self) -> &str;

    // Convert the module into it's source code and name
    fn into_raw_parts(self) -> (String, String);

    // Convert some source code and a file name into a module
    fn from_raw_parts(source: String, name: String) -> Self;
}

// A vertex module that will be loaded from .vrtx files
pub struct VertexModule {
    source: String,
    name: String,
}

// A fragment module that will be loaded from .frag files
pub struct FragmentModule {
    source: String,
    name: String,
}

// A compute module (only for compute shaders) that will be loaded from .cmpt files
pub struct ComputeModule {
    source: String,
    name: String,
}


// I love procedural programming
macro_rules! impl_module_traits {
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

impl_module_traits!(VertexModule, gl::VERTEX_SHADER, "vrsh.glsl");
impl_module_traits!(FragmentModule, gl::FRAGMENT_SHADER, "frsh.glsl");
impl_module_traits!(ComputeModule, gl::COMPUTE_SHADER, "cmpt.glsl");