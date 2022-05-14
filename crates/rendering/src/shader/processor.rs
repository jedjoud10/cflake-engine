use ahash::AHashMap;
use arrayvec::ArrayVec;
use assets::loader::AssetLoader;
use super::{Stage, RawText};

// A shader code constant. This value will be replaced at shader compile time (aka runtime)
pub struct Constant<T: ToString>(T);

// A processor is something that will take some raw GLSL text and expand/process each directive
pub struct Processor<'a> {
    // This is the asset loader that we will use to load include files
    loader: &'a mut AssetLoader,
    
    // A hashmap containing the constant values that we must replace
    // #const [name] [opt<default>]
    // #const [name]
    constants: AHashMap<String, String>,

    // A hashmap containing the snippets that we must replace
    // #snip [snip_name]
    snippets: AHashMap<String, String>,
}

impl<'a> Processor<'a> {
    // Include a constant directive
    pub fn insert_const(&mut self, name: impl ToString, value: impl ToString) {
        self.constants.insert(name.to_string(), value.to_string());
    }

    // Include a snippet directive
    pub fn insert_snippet(&mut self, name: impl ToString, value: impl ToString) {
        self.snippets.insert(name.to_string(), value.to_string());
    }

    // Filter and process a single stage
    pub fn filter<S: Stage>(&mut self, stage: S) -> S {
        // We must filter infinitely until we find no more directives
        let mut lines = stage.as_ref().to_string().lines().map(str::to_string).collect::<Vec<String>>();
        loop {            
            // Simply iterate through each line, and check if it starts with a directive that we must replace (whitespaces ignored)
            let mut skipped = 0usize;
            for line in lines.iter_mut() {
                // A bit of line trolling
                let trimmed = line.trim();

                // Output line
                let mut output = String::new();

                // Very funny indeed
                if trimmed.contains("#const") {
                    // Split into words, and classify name and default value
                    let words =  trimmed.split("#const").next().unwrap().split_whitespace().collect::<ArrayVec<&str, 3>>();
                    let name = words[0];
                    let default = words.get(1).cloned();

                    // Then try to load the value from the processor
                    let loaded = self.constants.get(name).map(String::as_str);
                    output = loaded.or(default).unwrap().to_string();
                } else if trimmed.starts_with("#snip") {
                } else if trimmed.starts_with("#include") {
                    // Split into words, and classify path
                    let words = trimmed.split_whitespace().collect::<ArrayVec<&str, 3>>();
                    let path = words[1];

                    // Load the path from the asset manager
                    let raw = self.loader.load::<RawText>(path).unwrap();
                    output = raw.0;
                } else {
                    // Don't overwrite really, and skip to the next line
                    skipped += 1;
                    continue;
                }           
                
                // Overwrite line with new output
                *line = output;
            }

            // Make sure we split the lines again
            lines = lines.join("\n").lines().map(str::to_string).collect::<Vec<String>>();
            
            // If we skipped all the lines, it means that we did absolutely nothing, and we can exit
            if skipped == lines.len() {
                break;
            }
        }

        // Combine the lines into a string and return the new, filtered stage
        S::from(lines.join("\n"))
    } 
}