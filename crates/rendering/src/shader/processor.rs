// This is a processor directive constant that can be replaced using the #const directive
pub trait ConstDirective: ToString {
}

// A processor is something that will take some raw GLSL text and expand/process each directive
pub struct Processor<'a> {
    // The path of the base file
    path: &'a str,

    // The contents of the base file
    base: String,

    // The given directives that we must replace at runtime
    // These are the ones currently supported:
    // #const [name]
    // #include [path]
    // #loadsnip [snip_name]
    // #custom [id]
    replace: AHashMap<String, String>,
}