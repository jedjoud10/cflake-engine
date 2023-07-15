use crate::Source;


// Buffer a source by caching it
pub struct Buffered<T: Source>(T, Vec<f32>);
