use std::path::PathBuf;

use super::{Processed, Stage};
use ahash::AHashMap;
use arrayvec::ArrayVec;
use assets::Assets;
use snailquote::unescape;

// A shader code constant. This value will be replaced at shader compile time (aka runtime)
pub struct Constant<T: ToString>(T);

// A processor is something that will take some raw GLSL text and expand/process each directive
pub struct Processor<'a> {
    // This is the asset loader that we will use to load include files
    loader: &'a Assets,

    // A hashmap containing the constant values that we must replace
    // #const [name]
    constants: AHashMap<String, String>,

    // A hashmap containing the snippets that we must replace
    // #snip [snip_name]
    snippets: AHashMap<String, String>,
}

impl<'a> From<&'a Assets> for Processor<'a> {
    fn from(loader: &'a Assets) -> Self {
        Self::new(loader)
    }
}

impl<'a> Processor<'a> {
    // Create a processor from an asset loader
    pub fn new(loader: &'a Assets) -> Self {
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
        let mut lines = source.lines().map(str::to_string).collect::<Vec<String>>();
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
                    let words = trimmed.split_whitespace().collect::<Vec<&str>>();

                    // Get the name and value (this assumes that there are no special character after the name of the directive)
                    let name = words[2];
                    let ty = words[1];
                    dbg!(name);
                    let loaded = self.constants.get(name).unwrap().clone();

                    // Create a whole new line with the proper params
                    let line = format!("const {} {} = {};", ty, name, loaded);

                    // Overwrite output
                    output = line;
                } else if trimmed.starts_with("#snip") {
                    let words = trimmed.split_whitespace().collect::<Vec<&str>>();

                    // Try to get the snippet
                    let snippet = self.snippets.get(words[1]).cloned().unwrap();
                    output = snippet;
                } else if trimmed.starts_with("#include") {
                    // Split into words, and classify path
                    let words = trimmed.split_whitespace().collect::<ArrayVec<&str, 3>>();
                    let path = unescape(words[1]).unwrap();

                    // Make sure the path is something we can load (.func file)
                    let pathbuf = PathBuf::try_from(path).unwrap();
                    let name = pathbuf.file_name().unwrap().to_str().unwrap();

                    // Load the path from the asset manager
                    let path = pathbuf.as_os_str().to_str().unwrap();
                    let raw = unsafe {
                        self.loader
                            .load_unchecked::<String>(path)
                            .expect(&format!("File '{name}' could not be loaded in"))
                    };
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
