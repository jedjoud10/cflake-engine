use std::path::{PathBuf, Path};

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
    // Combine other into self
    // Returns None if the operation failed (in case it's not possible)
    pub fn try_insert(&mut self, other: Self) -> Option<()> {
        match self {
            ModuleVisibility::Vertex
                if matches!(other, Self::Fragment) =>
            {
                *self = ModuleVisibility::VertexFragment;
                Some(())
            }

            ModuleVisibility::Fragment
                if matches!(other, Self::Vertex) =>
            {
                *self = ModuleVisibility::VertexFragment;
                Some(())
            }

            ModuleVisibility::Compute
                if matches!(other, Self::Compute) =>
            {
                Some(())
            }
            _ => None,
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
    fn new(path: impl AsRef<Path>, source: impl ToString) -> Self;

    // Get the main properties of the module
    fn path(&self) -> &Path;
    fn source(&self) -> &str;
    fn kind() -> ModuleKind;
    fn visibility() -> ModuleVisibility;

    // Convert the module into it's source code and path
    fn into_raw_parts(self) -> (PathBuf, String);
}

// A vertex module that will be loaded from .vrtx files
pub struct VertexModule {
    pub source: String,
    pub path: PathBuf,
}

// A fragment module that will be loaded from .frag files
pub struct FragmentModule {
    pub source: String,
    pub path: PathBuf,
}

// A compute module (only for compute shaders) that will be loaded from .cmpt files
pub struct ComputeModule {
    pub source: String,
    pub path: PathBuf,
}

// A function module is fucking useless
pub struct FunctionModule {
    pub source: String,
    pub path: PathBuf,
}

macro_rules! impl_asset_for_module {
    ($t: ty, $ext: expr) => {
        impl Asset for $t {
            type Context<'ctx> = ();
            type Settings<'stg> = ();
            type Err = std::string::FromUtf8Error;

            fn extensions() -> &'static [&'static str] {
                $ext
            }

            fn deserialize<'c, 's>(
                data: assets::Data,
                _: Self::Context<'c>,
                _: Self::Settings<'s>,
            ) -> Result<Self, Self::Err> {
                let source =
                    String::from_utf8(data.bytes().to_vec())?;
                let path = data
                    .path()
                    .to_path_buf();

                Ok(Self { source, path })
            }
        }
    };
}

// I love procedural programming
macro_rules! impl_module_trait {
    ($t: ty, $kind: ident) => {
        impl ShaderModule for $t {
            fn new(
                path: impl AsRef<Path>,
                source: impl ToString,
            ) -> Self {
                Self {
                    path: path.as_ref().to_path_buf(),
                    source: source.to_string(),
                }
            }

            fn path(&self) -> &Path {
                &self.path
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

            fn into_raw_parts(self) -> (PathBuf, String) {
                (self.path, self.source)
            }
        }
    };
}

// Implement the module trait
impl_module_trait!(VertexModule, Vertex);
impl_module_trait!(FragmentModule, Fragment);
impl_module_trait!(ComputeModule, Compute);

// Implement the asset trait
impl_asset_for_module!(VertexModule, &["vert", "vertex"]);
impl_asset_for_module!(FragmentModule, &["frag", "fragment"]);
impl_asset_for_module!(ComputeModule, &["comp", "compute"]);
impl_asset_for_module!(FunctionModule, &["glsl"]);
