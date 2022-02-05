use crate::pipeline::{Pipeline, PipelineRenderer};

// Some callback types
pub enum Callback {
    EndOfFrame(Box<dyn Fn(&mut Pipeline, &mut PipelineRenderer) + Sync + Send>),
    EndOfFrameOnce(Box<dyn FnOnce(&mut Pipeline, &mut PipelineRenderer) + Sync + Send>),
}
