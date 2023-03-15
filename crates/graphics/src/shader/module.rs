use assets::Asset;

// The type of shader module that the shader source represent
#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub enum ModuleKind {
    Vertex,
    Fragment,
    Compute,
}

// Describes the types of shader modules that are
// used by push constants and bind resources

// Normally, a sane person would use bitflags for this
// However, I have lost sanity a long time ago, so fuck you
// Cope with this shit
// (real reason is cause bitflags could be zero/empty, and I don't want this to be zero since it don't make sense)
#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub enum ModuleVisibility {
    Vertex,
    Fragment,
    VertexFragment,
    Compute,
}

impl ModuleVisibility {
    // Combine other into self (panics if not possible)
    pub fn insert(&mut self, other: Self) {
        match self {
            ModuleVisibility::Vertex => {
                if !matches!(other, Self::Fragment) {
                    *self = ModuleVisibility::VertexFragment;
                }
            }

            ModuleVisibility::Fragment => {
                if !matches!(other, Self::Vertex) {
                    *self = ModuleVisibility::VertexFragment;
                }
            }

            ModuleVisibility::VertexFragment => {
                if matches!(other, Self::Compute) {
                    panic!()
                }
            }

            ModuleVisibility::Compute => {
                if !matches!(other, Self::Compute) {
                    panic!()
                }
            }
        }
    }

    // Check if Self contains other
    pub fn contains(&self, other: Self) -> bool {
        match self {
            ModuleVisibility::Vertex => matches!(other, Self::Vertex),
            ModuleVisibility::Fragment => {
                matches!(other, Self::Fragment)
            }
            ModuleVisibility::VertexFragment => {
                !matches!(other, Self::Compute)
            }
            ModuleVisibility::Compute => {
                matches!(other, Self::Compute)
            }
        }
    }

    // Check if the given ShaderModule is visible
    pub fn visible<M: ShaderModule>(&self) -> bool {
        let visibility = M::visibility();
        self.contains(visibility)
    }
}

// Convert module visibility to wgpu ShaderStages
pub(crate) fn visibility_to_wgpu_stage(
    visibility: &ModuleVisibility,
) -> wgpu::ShaderStages {
    match visibility {
        ModuleVisibility::Vertex => wgpu::ShaderStages::VERTEX,
        ModuleVisibility::Fragment => wgpu::ShaderStages::FRAGMENT,
        ModuleVisibility::VertexFragment => {
            wgpu::ShaderStages::VERTEX_FRAGMENT
        }
        ModuleVisibility::Compute => wgpu::ShaderStages::COMPUTE,
    }
}

// Convert module kind to module visibility
pub(crate) fn kind_to_visibility(
    kind: &ModuleKind,
) -> ModuleVisibility {
    match kind {
        ModuleKind::Vertex => ModuleVisibility::Vertex,
        ModuleKind::Fragment => ModuleVisibility::Fragment,
        ModuleKind::Compute => ModuleVisibility::Compute,
    }
}

// Convert a module kind to WGPU shader stage bitfield
pub(crate) fn kind_to_wgpu_stage(
    kind: &ModuleKind,
) -> wgpu::ShaderStages {
    match *kind {
        ModuleKind::Vertex => wgpu::ShaderStages::VERTEX,
        ModuleKind::Fragment => wgpu::ShaderStages::FRAGMENT,
        ModuleKind::Compute => wgpu::ShaderStages::COMPUTE,
    }
}

// This trait is implemented for each shader module, like the vertex module or fragment module
// Modules are uncompiled shaders that will later be converted to SPIRV and linked together
pub trait ShaderModule: Sized {
    // Create a new fake module with a name and source code
    fn new(name: impl ToString, source: impl ToString) -> Self;

    // Get the main properties of the module
    fn name(&self) -> &str;
    fn source(&self) -> &str;
    fn kind() -> ModuleKind;
    fn visibility() -> ModuleVisibility;

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
    ($t: ty, $kind: ident) => {
        impl ShaderModule for $t {
            fn new(
                name: impl ToString,
                source: impl ToString,
            ) -> Self {
                Self {
                    name: name.to_string(),
                    source: source.to_string(),
                }
            }

            fn name(&self) -> &str {
                &self.name
            }

            fn source(&self) -> &str {
                &self.source
            }

            fn kind() -> ModuleKind {
                ModuleKind::$kind
            }

            fn visibility() -> ModuleVisibility {
                ModuleVisibility::$kind
            }

            fn into_raw_parts(self) -> (String, String) {
                (self.name, self.source)
            }
        }
    };
}

// Implement the module trait
impl_module_trait!(VertexModule, Vertex);
impl_module_trait!(FragmentModule, Fragment);
impl_module_trait!(ComputeModule, Compute);

// Implement the asset trait
impl_asset_for_module!(VertexModule, "vert");
impl_asset_for_module!(FragmentModule, "frag");
impl_asset_for_module!(ComputeModule, "comp");
impl_asset_for_module!(FunctionModule, "glsl");
