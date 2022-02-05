use crate::pipeline::Pipeline;

// Some callback types
pub enum Callback {
    EndOfFrame(Box<dyn Fn(&mut Pipeline) + Sync + Send>),
    EndOfFrameOnce(Box<dyn FnOnce(&mut Pipeline) + Sync + Send>),
}