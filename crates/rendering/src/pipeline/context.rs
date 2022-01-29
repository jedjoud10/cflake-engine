use std::sync::{Arc, RwLock, RwLockReadGuard, Mutex};

use super::{Pipeline, PipelineHandler};

// A pipeline context that we can share to the main thread
#[derive(Clone)]
pub struct PipelineContext {
    // The pipeline
    pub(crate) pipeline: Arc<RwLock<Pipeline>>,
    // The pipeline's handler that allows us to call specific pipeline tasks from the main thread, like flush
    pub handler: Arc<Mutex<PipelineHandler>>,
}

impl PipelineContext {
    // Read
    pub fn read<'a>(&'a self) -> ReadPipelineContext<'a> {
        ReadPipelineContext {
            pipeline: self.pipeline.read().unwrap(),
        }
    }
}

// A readable pipeline context
pub struct ReadPipelineContext<'a> {
    pipeline: RwLockReadGuard<'a, Pipeline>,
}

impl<'a> std::ops::Deref for ReadPipelineContext<'a> {
    type Target = Pipeline;

    fn deref(&self) -> &Self::Target {
        &*self.pipeline
    }
}