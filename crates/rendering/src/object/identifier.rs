use std::{marker::PhantomData, sync::{Arc, atomic::AtomicPtr}};
use others::{ExternalID};
use crate::SharedPipeline;
use super::{PipelineObject, PipelineTaskStatus};

// An ID for the PipelineObject
pub struct IPipelineObjectID {
    pub(crate) id: u16, // The OrderedVec's ID for this pipeline object
}
// A wrapper so we hide the generic type T so we can store this in the SharedPipeline
pub(crate) struct IAsyncPipelineObjectID {
    id: usize,
}

impl ExternalID<IPipelineObjectID> for IAsyncPipelineObjectID {
    fn new() -> Self {
        panic!()
    }

    fn id(&self) -> usize {
        self.id
    }
}

// A simple ptr to the actual PipelineObjectID
// We can detect whenever the actual Pipeline command finished generating / updating the Object specified by the ID
pub struct PipelineObjectID<T> 
    where T: PipelineObject
{
    phantom: PhantomData<*const T>,
    id: usize,
}

impl<T> ExternalID<IPipelineObjectID> for PipelineObjectID<T> 
    where T: PipelineObject 
{
    fn new() -> Self {
        Self {
            phantom: PhantomData::default(),
            id: Self::increment(),
        }
    }

    fn id(&self) -> usize {
        self.id
    }
}

impl<T> others::Watchable<SharedPipeline> for PipelineObjectID<T>
    where T: PipelineObject
{
    fn get_uid(&self) -> usize {
        self.id
    }

    fn is_valid(&self, pipeline: &SharedPipeline) -> bool {
        Self::try_get_id(self.id, &pipeline.buffer).is_some()
    }
}

// A simple ptr to the actual PipelineObjectID
// We can detect whenever the actual Pipeline Task finished using the others::Watchable trait
pub struct AsyncPipelineTaskID
{
    id: usize,
}

impl ExternalID<PipelineTaskStatus> for AsyncPipelineTaskID {
    fn new() -> Self {
        Self {
            id: Self::increment(),
        }
    }

    fn id(&self) -> usize {
        self.id
    }
}

impl others::Watchable<SharedPipeline> for AsyncPipelineTaskID {
    fn get_uid(&self) -> usize {
        self.id
    }

    fn is_valid(&self, pipeline: &SharedPipeline) -> bool {
        if let Some(status) = Self::try_get_id(self.id, &pipeline.task_buffer) {
            if let PipelineTaskStatus::Finished = status {
                true
            } else { false }
        } else { false }
    }
}