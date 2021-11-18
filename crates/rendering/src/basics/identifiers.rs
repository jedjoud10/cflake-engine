// Some identifiers that we will use to communicate from the Render Thread -> Main Thread
#[derive(Clone)]
pub enum GPUObject {
    None, // This value was not initalized yet
    Model(usize),
    Shader(usize),
    Texture(usize),
}

impl Default for GPUObject {
    fn default() -> Self {
        Self::None
    }
}