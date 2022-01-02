use std::{marker::PhantomData, sync::{Arc, atomic::AtomicPtr}};
use others::CommandID;
use super::GPUObject;

// An ID for the PipelineObject
pub struct PipelineObjectID {
    pub(crate) id: u16, // The OrderedVec's ID for this pipeline object
}




// A simple ptr to the actual GPUObjectID
// We can detect whenever the actual Pipeline command finished generating / updating the Object specified by the ID
pub struct AsyncPipelineObjectID<T>
    where T: GPUObject

{
    phantom: PhantomData<T>,
    pub(crate) command_id: CommandID,
    pipeline_object_id: Arc<AtomicPtr<PipelineObjectID>>,
}

impl<T> others::Watchable for AsyncPipelineObjectID<T>
    where T: GPUObject
{
    fn get_id(&self) -> CommandID {
        self.command_id
    }
}

impl<T> others::ExternalID<PipelineObjectID> for AsyncPipelineObjectID<T>
    where T: GPUObject 
{
    fn ptr(&self) -> &Arc<AtomicPtr<PipelineObjectID>> {
        &self.pipeline_object_id
    }
}

// A simple ptr to the actual PipelineObjectID
// We can detect whenever the actual Pipeline command finished using the others::Watchable trait
pub struct AsyncPipelineCommand {
    pub(crate) command_id: CommandID,
}