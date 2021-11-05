// A variable hash
pub struct VarHash {
    // Hash
    pub hash: u64,
    // Type
    pub _type: VarHashType
}

pub enum VarHashType {
    // Density values are complitely different than normal values
    Density,
    // Multiple values
    Float,
    Vec2,
    Vec3,

}