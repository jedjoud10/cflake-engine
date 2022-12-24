use crate::{FunctionModule, Graphics, ShaderModule, ModuleKind};
use ahash::AHashMap;
use assets::Assets;
use std::{marker::PhantomData, path::PathBuf};

// A processor is something that will take some raw GLSL text and expand/process each directive
pub struct Processor<'a, M: ShaderModule> {
    assets: &'a Assets,
    kind: ModuleKind,
    source: String,
    file_name: String,
    snippets: AHashMap<String, String>,
    _phantom: PhantomData<M>,
}

impl<'a, M: ShaderModule> Processor<'a, M> {
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
            _phantom: PhantomData,
            snippets: Default::default(),
        }
    }

    // Include a constant directive that will replace specialization constants (stored internally until compile time)
    pub fn define_constant(
        &mut self,
        name: impl ToString,
        value: impl ToString,
    ) {
        // Somehow make this work 
        // https://github.com/gwihlidal/spirv-reflect-rs
        todo!()
    }

    // Include a snippet directive that will replace ``#include`` lines that don't refer to a file
    pub fn define_snippet(
        &mut self,
        name: impl ToString,
        value: impl ToString,
    ) {
        let name = name.to_string();
        log::debug!("Defined snippet '{}' for processor '{}'", &name, &self.file_name);
        self.snippets.insert(name, value.to_string());
    }

    // Process the internally stored module and convert it to "Processed<M>"
    pub fn process(self) -> Processed<M> {
        let output = process(self.source, self.assets, self.snippets);

        Processed {
            kind: self.kind,
            file_name: self.file_name,
            source: output,
            _phantom: PhantomData,
        }
    }
}

// Main function that will process the shader directives outside of shaders
fn process(source: String, assets: &Assets, snippets: AHashMap<String, String>) -> String {
    // We must filter repeatedly until we find no more directives
    let mut lines =
        source.lines().map(str::to_string).collect::<Vec<String>>();
    loop {
        // Simply iterate through each line, and check if it starts with a directive that we must replace (whitespaces ignored)
        let mut should_stop = true;
        for line in lines.iter_mut() {
            let trimmed = line.trim();
            let mut output = String::new();

            // Include statements work with files and snippets
            if trimmed.starts_with("#include") {
                handle_include(trimmed, assets, &mut output, &snippets);

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
fn handle_include(
    trimmed: &str,
    assets: &Assets,
    output: &mut String,
    snippets: &AHashMap<String, String>,
) {
    // Split into words, and classify macro value (either path or snippet name)
    let words = trimmed.split_whitespace().collect::<Vec<&str>>();
    let name = snailquote::unescape(words[1]).unwrap();

    // Either load it as an asset or a snippet
    if resembles_asset_path(&name) {
        load_function_module(name, assets, output);
    } else {
        load_snippet(name, output, snippets);
    }
}

// Load a function module and write it to the output line
fn load_function_module(path: String, assets: &Assets, output: &mut String) {
    // Make sure the path is something we can load (.glsl file)
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

// Load a snippet from the snippets and write it to the output line
fn load_snippet(name: String, output: &mut String, snippets: &AHashMap<String, String>) {
    let snippet = snippets
        .get(&name)
        .expect(&format!("Snippet of name '{name}' was not defined"));
    *output = snippet.clone(); 
}
 
// Check if an include directive resembles like an asset path instead of a snippet
fn resembles_asset_path(path: &str) -> bool {
    let value = || {
        let pathbuf = PathBuf::try_from(path).ok()?;
        let extension = pathbuf.extension()?.to_str()?;
        Some(extension == "glsl")
    };
    value().unwrap_or_default()
} 

// This is a successfully processed staged returned by the "Processor"
pub struct Processed<M: ShaderModule> {
    pub(crate) kind: ModuleKind,
    pub(crate) source: String,
    pub(crate) file_name: String,
    _phantom: PhantomData<M>,
}
