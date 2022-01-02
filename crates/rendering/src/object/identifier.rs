use std::{marker::PhantomData, sync::{Arc, atomic::AtomicPtr}};
use others::{CommandID, ExternalID};

use super::PipelineObject;

// An ID for the PipelineObject
pub struct PipelineObjectID {
    pub(crate) id: u16, // The OrderedVec's ID for this pipeline object
}
// A simple ptr to the actual GPUObjectID
// We can detect whenever the actual Pipeline command finished generating / updating the Object specified by the ID
pub struct AsyncPipelineObjectID<T>
    where T: PipelineObject
{
}

impl<T> others::Watchable for AsyncPipelineObjectID<T>
    where T: PipelineObject
{
    fn get_id(&self) -> CommandID {
        self.command_id
    }
}

// A simple ptr to the actual PipelineObjectID
// We can detect whenever the actual Pipeline Task finished using the others::Watchable trait
// AsyncPipelineTaskID