use assets::Asset;

// The type of shader module that the shader files represent
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ModuleKind {
    // Vertex shaders get executed on a per vertex basis
    Vertex,
    
    // Fragment shaders get executed for each fragment, or each pixel (in case of no MSAA)
    Fragment
}

// This trait is implemented for each shader module, like the vertex module or fragment module
pub trait Module: Sized {
    // Get the main properties of the module
    fn file_name(&self) -> &str;
    fn source(&self) -> &str;
    fn kind(&self) -> ModuleKind;

    // Convert the module into it's source code and name
    fn into_raw_parts(self) -> (String, String);

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
    ($t: ty, $kind: expr, $ext: expr) => {
        impl Module for $t {
            fn file_name(&self) -> &str {
                &self.name
            }

            fn source(&self) -> &str {
                &self.source
            }

            fn kind(&self) -> ModuleKind {
                $kind
            }

            fn into_raw_parts(self) -> (String, String) {
                (self.source, self.name)
            }
        }

        impl Asset for $t {
            type Args<'a> = ();

            fn extensions() -> &'static [&'static str] {
                &[$ext]
            }

            fn deserialize(data: assets::Data, _args: Self::Args<'_>) -> Self {
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

impl_module_traits!(VertexModule, ModuleKind::Vertex, "vrsh.glsl");
impl_module_traits!(FragmentModule, ModuleKind::Fragment, "frsh.glsl");