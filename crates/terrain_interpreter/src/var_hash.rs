// A variable hash
#[derive(Clone, Copy)]
pub struct VarHash {
    pub name: usize,
    pub _type: VarHashType
}

#[derive(Clone, Copy)]
pub enum VarHashType {
    // Density values are complitely different than normal values
    Density,
    // Multiple values
    Float,
    Vec2,
    Vec3,
}

impl VarHashType {
    // Convert this var hash type to a string prefix
    pub fn to_string(&self) -> String {
        match &self {
            VarHashType::Density => "d",
            VarHashType::Float => "v1",
            VarHashType::Vec2 => "v2",
            VarHashType::Vec3 => "v3",
        }.to_string()
    }
    // Get the HLSL type for this var hash type
    pub fn to_hlsl_type(&self) -> String {
        match &self {
            VarHashType::Density => "float",
            VarHashType::Float => "float",
            VarHashType::Vec2 => "vec2",
            VarHashType::Vec3 => "vec3",
        }.to_string()
    }
}

impl VarHash {
    // Get variable name using a prefix from the varhashtype
    pub fn get_name(&self) -> String {
        format!("{}_{}", self._type.to_string(), self.name)
    }
}