
// Some settings that tell us how exactly we should create the texture
#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum TextureMode {
    // Dynamic textures can be modified throughout their lifetime, but they cannot be resized
    Dynamic,
    
    // Resizable textures are just dynamic textures that we can resize
    #[default]
    Resizable,
}
