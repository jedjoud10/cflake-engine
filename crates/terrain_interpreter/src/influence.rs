// Influence
pub enum Influence {
    None,
    Default,
    // Influence range. This is used to do that smart bound checking
    Modified(f32, f32)
}