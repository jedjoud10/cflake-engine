use super::{Processed, Stage};
use ahash::AHashMap;
use arrayvec::ArrayVec;
use assets::{Asset, Assets};

// A shader code constant. This value will be replaced at shader compile time (aka runtime)
pub struct Constant<T: ToString>(T);

// A processor is something that will take some raw GLSL text and expand/process each directive
pub struct Processor<'a> {
    // This is the asset loader that we will use to load include files
    loader: &'a mut Assets,

    // A hashmap containing the constant values that we must replace
    // #const [name]
    constants: AHashMap<String, String>,

    // A hashmap containing the snippets that we must replace
    // #snip [snip_name]
    snippets: AHashMap<String, String>,
}

impl<'a> From<&'a mut Assets> for Processor<'a> {
    fn from(loader: &'a mut Assets) -> Self {
        Self::new(loader)
    }
}

impl<'a> Processor<'a> {
    // Create a processor from an asset loader
    pub fn new(loader: &'a mut Assets) -> Self {
        Self {
            loader,
            constants: Default::default(),
            snippets: Default::default(),
        }
    }

    // Include a constant directive
    pub fn insert_constant(&mut self, name: impl ToString, value: impl ToString) {
        self.constants.insert(name.to_string(), value.to_string());
    }

    // Include a snippet directive
    pub fn insert_snippet(&mut self, name: impl ToString, value: impl ToString) {
        self.snippets.insert(name.to_string(), value.to_string());
    }

    // Filter and process a single stage
    pub(super) fn filter<S: Stage>(&mut self, stage: S) -> Processed<S> {
        // We must filter repeatedly until we find no more directives
        let (source, name) = stage.into_raw_parts();
        let mut lines = source
            .lines()
            .map(str::to_string)
            .collect::<Vec<String>>();
        loop {
            // Simply iterate through each line, and check if it starts with a directive that we must replace (whitespaces ignored)
            let mut skipped = 0usize;
            for line in lines.iter_mut() {
                // A bit of line trolling
                let trimmed = line.trim();

                // Output line
                let output;

                // Very funny indeed
                if trimmed.contains("#const") {
                    // Get the start location and end location of the directive bound
                    let directive = trimmed.split_whitespace().position(|n| n == "#const").unwrap();
                    let name = directive + 1;

                    // Split into words
                    let mut words = trimmed
                        .split_whitespace()
                        .collect::<Vec<&str>>();

                    // Get the name and value
                    let filtered = words[name].replace(&['(', ')', ',', '\"', '.', ';', ':', '\''], "");
                    let name = filtered.trim();
                    println!("{}", name);
                    let loaded = self.constants.get(name).unwrap();

                    // Remove the directive words and replace them with the proper values
                    words.remove(directive);
                    words.remove(directive);
                    words.insert(directive, loaded);

                    // Overwrite output
                    output = words.join(" ");
                    println!("{}", &output);
                } else if trimmed.starts_with("#snip") {
                    // Split into words, and classify name
                    let words = trimmed
                        .split("#snip")
                        .next()
                        .unwrap()
                        .split_whitespace()
                        .collect::<ArrayVec<&str, 3>>();
                    let name = words[0];

                    // Try to get the snippet
                    let snippet = self.snippets.get(name).cloned().unwrap();
                    output = snippet;
                } else if trimmed.starts_with("#include") {
                    // Split into words, and classify path
                    let words = trimmed.split_whitespace().collect::<ArrayVec<&str, 3>>();
                    let path = words[1];

                    // Make sure the path is something we can load (.func file)
                    if !path.ends_with(".func") {
                        panic!();
                    }

                    // Load the path from the asset manager
                    let raw = self.loader.load::<String>(path).unwrap();
                    output = raw;
                } else {
                    // Don't overwrite really, and skip to the next line
                    skipped += 1;
                    continue;
                }

                // Overwrite line with new output
                *line = output;
            }

            // Make sure we split the lines again
            lines = lines
                .join("\n")
                .lines()
                .map(str::to_string)
                .collect::<Vec<String>>();

            // If we skipped all the lines, it means that we did absolutely nothing, and we can exit
            if skipped == lines.len() {
                break;
            }
        }

        // Combine the lines into a string and return the new, filtered stage
        Processed(S::from_raw_parts(lines.join("\n"), name))
    }
}
