use std::{collections::{HashMap, HashSet}, io::BufRead};

use super::{ShaderSource, ShaderSettings, load_includes};

// Some shader source info that we can retrieve just by reading the glsl file, no need for OpenGL context
pub struct ShaderSourceInfo {
    // All the types that have been declared or automatically declared, and their respective size in bytes
    pub types: HashMap<String, usize>,
}

impl ShaderSourceInfo {
    // Create some shader source info from a glsl file that we get from some shader settings using a source path
    pub fn new(source_path: &str, settings: &ShaderSettings) -> Self {
        let source = settings.sources.get(source_path).unwrap();
        let mut text = source.text.clone();
        // We gotta expand some custom includes so we get the whole file
        let mut included_paths: HashSet<String> = HashSet::new();
        while load_includes(&settings, &mut text, &mut included_paths).unwrap() {
            // We are still including paths
        }

        // Some default types and their byte sizes
        let mut types = HashMap::new();
        types.insert("float".to_string(), 4);
        types.insert("int".to_string(), 4);
        types.insert("uint".to_string(), 4);
        types.insert("bool".to_string(), 1);
        types.insert("double".to_string(), 8);
        // Also vector types
        for x in 2..5 {
            types.insert(format!("bvec{}", x), x);
            types.insert(format!("ivec{}", x), x * 4);
            types.insert(format!("uvec{}", x), x * 4);
            types.insert(format!("vec{}", x), x * 4);
            types.insert(format!("dvec{}", x), x * 8);
        }
        
        let mut me = Self {
            types,
        };
        me.analyze(text);
        me
    }

    // Analyse a specific glsl file
    pub fn analyze(&mut self, glsl: String) {
        // First, loop through the whole file and keep track of the struct definitions that we come across
        // After scanning the whole file, we go to each struct definition and replace it's types (types and names) to their underlying size bytes
        // Then all we gotta do is sum the byte sizes up for each struct

        println!("{}", glsl);

        // Split the file into multiple lines first
        let lines = glsl.lines().collect::<Vec<_>>();
        // Some variables used to keep track of our current state
        let mut struct_def = false;
        let mut struct_def_start = false;
        let mut found_chars = Vec::<char>::new();
        // Whenever we completely find a struct definition, we must reset our values 
        let reset = || {
            struct_def = false;
            struct_def_start = false;
            found_chars.clear();
        };

        for line in lines {
            // If we have a struct definition, the line would start with "struct"
            if line.starts_with("struct") {
                // This is a struct definition
                struct_def = true;
                let chars = line.chars();
                let next_valid_char = chars.position(|x| x == '{');
                let mut closing_bracket = false;
                if let Some(mut i) = next_valid_char {
                    chars.nth(i);
                    // We found the starting bracket on the same line as the struct name definition
                    struct_def_start = true;
                    
                    // Start looping through each an single character until we find a closing bracket
                    while !closing_bracket {
                        let maybe = chars.next();
                        if let Some(maybe) = maybe {
                            // Check
                            if maybe == '}' {
                                // We found the closing curly bracket on the same line, we can add this as a new struct def and reset our values
                                reset()
                            } else {
                                // Aint it boi
                                found_chars.push(maybe);
                            }
                        } else {
                            // We couldn't find the closing curly bracket on the same line as the starting bracket
                        }
                    }
                }
                continue;
            }

            // Keep track of the chars, so we make sure that there is only pure void (blank spaces or nothing)
            if struct_def {
            }

            // If we find an opening curly bracket while we have a struct definition that means we are defining the struct itself
            // We should have nothing between this curly bracket and the name of the struct (before the curly bracket)
            if struct_def && line.contains("{")  && 
        }
    }
}