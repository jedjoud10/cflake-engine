use ahash::AHashMap;

// Struct that contains all the [name -> location] mappings for each uniform in a shader
pub struct UniformsDefinitionMap {
    mappings: AHashMap<String, i32>,
} 

impl UniformsDefinitionMap {
    // Get a single uniform using it's name
    pub fn get(&self, name: &str) -> Option<i32> {
        self.mappings.get(name).cloned()
    }
}