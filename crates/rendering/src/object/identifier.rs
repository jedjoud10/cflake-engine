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
    phantom: PhantomData<T>,
    pub(crate) command_id: CommandID,
    pipeline_object_id: Arc<AtomicPtr<PipelineObjectID>>,
}

impl<T> others::Watchable for AsyncPipelineObjectID<T>
    where T: PipelineObject
{
    fn get_id(&self) -> CommandID {
        self.command_id
    }
}

impl<T> others::ExternalID<PipelineObjectID> for AsyncPipelineObjectID<T>
    where T: PipelineObject 
{
    fn ptr(&self) -> &Arc<AtomicPtr<PipelineObjectID>> {
        &self.pipeline_object_id
    }
}

// A simple ptr to the actual PipelineObjectID
// We can detect whenever the actual Pipeline Task finished using the others::Watchable trait
pub struct AsyncPipelineTaskID {
    pub(crate) command_id: CommandID,
    executed_command: Arc<AtomicPtr<bool>>,
}

impl others::Watchable for AsyncPipelineTaskID
{
    fn get_id(&self) -> CommandID {
        self.command_id
    }

    fn is_valid(&self) -> bool {
        self.is_valid()
    }
}

impl others::ExternalID<bool> for AsyncPipelineTaskID
{
    fn ptr(&self) -> &Arc<AtomicPtr<bool>> {
        &self.executed_command
    }
}