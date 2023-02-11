use thiserror::Error;

#[derive(Error, Debug)]
pub enum PipelineInitializationError {
    #[error("{0}")]
    VertexConfigError(PipelineVertexConfigError),
}

#[derive(Error, Debug)]
pub enum PipelineVertexConfigError {
    #[error("The shader location {0} is used multiple times")]
    ShaderLocationRedefined(u32),
}

/*
#[derive(Error, Debug)]
pub enum PipelineBindingConfigError {
    #[error("{0:?}")]
    PushConstBlock(PushConstBlockError),
}

#[derive(Error, Debug)]
pub enum PushConstBlockError {
    #[error("The push constant block {0} is not defined in the {1:?} module")]
    NotDefinedShader(String, ModuleKind),

    #[error("The push constant block {0} is not defined in the {1:?} module in BindingsConfig")]
    NotDefinedBindings(String, ModuleKind),

    #[error("The push constant block name of the {0:?} module does match up")]
    NameMismatch(ModuleKind),

    #[error("The push constant variable {0} (from block {1}, {2:?} module) is not defined in the shader")]
    VariableNotDefinedShader(String, String, ModuleKind),

    #[error("The push constant variable {0} (from block {1}, {2:?} module) is not defined in BindingsConfig")]
    VariableNotDefinedBindings(String, String, ModuleKind),

    #[error("The push constant variable {0} has mismatching representations")]
    VariableTypeMismatch(String),
}
*/