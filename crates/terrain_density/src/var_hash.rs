// A variable hash
pub struct VarHash {
    // Hash
    pub hash: u64,
    // Type
    pub _type: VarHashType
}

pub enum VarHashType {
    Density,
    Position,
}