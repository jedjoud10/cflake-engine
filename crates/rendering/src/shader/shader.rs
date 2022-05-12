use assets::Asset;

// A shader linker that will take multiple sources and link them together into a shader


// A shader that will render our objects onto the screen
// This will make use of two shader programs, the vertex programs, and fragment program
pub struct Shader {
    vertex: super::Source,
    fragment: super::Source,
    geometry: Option<super::Source>,
    tesselation: Option<super::Source>,
}
