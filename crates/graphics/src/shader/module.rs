use assets::Asset;
use naga::ShaderStage;

// This trait is implemented for each shader module, like the vertex module or fragment module
// Modules are uncompiled shaders that will later be converted to SPIRV and linked together
pub trait ShaderModule: Sized {
    // Get the main properties of the module
    fn file_name(&self) -> &str;
    fn source(&self) -> &str;
    fn stage(&self) -> ShaderStage;

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

// A function module is fucking useless
pub struct FunctionModule {
    pub(crate) source: String,
    name: String,
}

macro_rules! impl_asset_for_module {
    ($t: ty, $ext: expr) => {
        impl Asset for $t {
            type Context<'ctx> = ();
            type Settings<'stg> = ();
            type Err = std::string::FromUtf8Error;

            fn extensions() -> &'static [&'static str] {
                &[$ext]
            }

            fn deserialize<'c, 's>(
                data: assets::Data,
                _: Self::Context<'c>,
                _: Self::Settings<'s>,
            ) -> Result<Self, Self::Err> {
                let source =
                    String::from_utf8(data.bytes().to_vec())?;
                let name = data
                    .path()
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string();

                Ok(Self { source, name })
            }
        }
    };
}

// I love procedural programming
macro_rules! impl_module_trait {
    ($t: ty, $kind: expr) => {
        impl ShaderModule for $t {
            fn file_name(&self) -> &str {
                &self.name
            }

            fn source(&self) -> &str {
                &self.source
            }

            fn stage(&self) -> ShaderStage {
                $kind
            }

            fn into_raw_parts(self) -> (String, String) {
                (self.name, self.source)
            }
        }
    };
}

// Implement the module trait
impl_module_trait!(VertexModule, ShaderStage::Vertex);
impl_module_trait!(FragmentModule, ShaderStage::Fragment);
impl_module_trait!(ComputeModule, ShaderStage::Compute);

// Implement the asset trait
impl_asset_for_module!(VertexModule, "vert");
impl_asset_for_module!(FragmentModule, "frag");
impl_asset_for_module!(ComputeModule, "comp");
impl_asset_for_module!(FunctionModule, "glsl");
