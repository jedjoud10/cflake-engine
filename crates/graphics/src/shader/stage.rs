use assets::Asset;

// The type of shader module that the shader files represent
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ModuleKind {
    // Vertex shaders get executed on a per vertex basis
    Vertex,
    
    // Fragment shaders get executed for each fragment, or each pixel (in case of no MSAA)
    Fragment,

    // Compute shaders are arbitrary shaders that run on arbitrary input and output
    Compute,
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
macro_rules! impl_module {
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
            type Args<'args> = ();
            type Err = std::string::FromUtf8Error;
        
            fn extensions() -> &'static [&'static str] {
                &[$ext]
            }
        
            fn deserialize<'args>(
                data: assets::Data,
                _args: Self::Args<'args>,
            ) -> Result<Self, Self::Err> {
                let source = String::from_utf8(data.bytes().to_vec())?;
                let name = data.path()
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string();

                Ok(Self {
                    source, name
                })
            }
        }
        
    };
}

impl_module!(VertexModule, ModuleKind::Vertex, "vert");
impl_module!(FragmentModule, ModuleKind::Fragment, "frag");
impl_module!(ComputeModule, ModuleKind::Compute, "comp");