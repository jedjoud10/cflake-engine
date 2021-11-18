

// Some identifiers that we will use to communicate from the Render Thread -> Main Thread
#[derive(Clone)]
pub enum GPUObject {
    None, // This value was not initalized yet
    Model(usize), // The VAO ID
    Shader(usize), // The shader program ID
    Texture(usize), // The texture ID
}

impl Default for GPUObject {
    fn default() -> Self {
        Self::None
    }
}