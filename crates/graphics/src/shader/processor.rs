use std::{marker::PhantomData, path::PathBuf};
use assets::Assets;
use crate::{Module, Graphics, ModuleKind, FunctionModule};

// A processor is something that will take some raw GLSL text and expand/process each directive
pub struct Processor<'a, M: Module> {
    assets: &'a Assets,
    kind: ModuleKind,
    source: String,
    file_name: String,
    _phantom: PhantomData<M>,
}

impl<'a, M: Module> Processor<'a, M> {
    // Create a processor that will execute over the given module
    pub fn new(module: M, assets: &'a Assets) -> Self {
        let kind = module.kind();
        let (file_name, source) = module.into_raw_parts();
        log::debug!("Created a new processor for {}", file_name);

        Self {
            assets,
            kind,
            source,
            file_name,
            _phantom: PhantomData
        }
    }

    // Include a constant directive that will replace specialization constants (stored internally until compile time)
    pub fn define_constant(&mut self, name: impl ToString, value: impl ToString) {}

    // Include a snippet directive that will replace ``#include`` lines that don't refer to a file
    pub fn define_snippet(&mut self, name: impl ToString, value: impl ToString) {}

    // Process the internally stored module and convert it to "Processed<M>"
    pub fn process(self) -> Processed<M> {
        let output = process(self.source, self.assets);
    
        Processed {
            kind: self.kind,
            file_name: self.file_name,
            source: output,
            _phantom: PhantomData,
        }
    }
}

// Main function that will process the shader directives outside of shaders
fn process(source: String, assets: &Assets) -> String {
    // We must filter repeatedly until we find no more directives
    let mut lines = source.lines().map(str::to_string).collect::<Vec<String>>();
    loop {
        // Simply iterate through each line, and check if it starts with a directive that we must replace (whitespaces ignored)
        let mut should_stop = true;
        for line in lines.iter_mut() {
            let trimmed = line.trim();
            let mut output = String::new();

            // Include statements work with files and snippets
            if trimmed.starts_with("#include") {
                handle_include(trimmed, assets, &mut output);
                
                // Overwrite line with new output
                should_stop = false;
                *line = output;
            } else {
                continue;
            }
        }

        // Make sure we split the lines again
        lines = lines
            .join("\n")
            .lines()
            .map(str::to_string)
            .collect::<Vec<String>>();

        // Break if we must
        if should_stop {
            break;
        }
    }
    lines.join("\n")
}

// Handle dealing with the include directive (that works with asset paths and snippets)
fn handle_include(trimmed: &str, assets: &Assets, output: &mut String) {
    // Split into words, and classify path
    let words = trimmed.split_whitespace().collect::<Vec<&str>>();
    let path = snailquote::unescape(words[1]).unwrap();

    // Make sure the path is something we can load (.func file)
    let pathbuf = PathBuf::try_from(path).unwrap();
    let name = pathbuf.file_name().unwrap().to_str().unwrap();

    // Load the path from the asset manager
    let path = pathbuf.as_os_str().to_str().unwrap();
    let raw = assets
        .load::<FunctionModule>(path)
        .map(|x| x.source)
        .expect(&format!("File '{name}' could not be loaded in"));
    *output = raw;
}

// This is a successfully processed staged returned by the "Processor"
pub struct Processed<M: Module> {
    pub(crate) kind: ModuleKind,
    pub(crate) source: String,
    pub(crate) file_name: String,
    _phantom: PhantomData<M>,
}