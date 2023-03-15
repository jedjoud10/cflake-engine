// The possible states that a fence can be in
pub enum FenceState {
    NotSubmitted,
    Executing,
    Finished,
}

// A fence can be used to wait for the GPU to complete a specific command
// They can also be used to *check* if the GPU has finished said command
// FIXME: Implement dis
pub struct Fence {}

impl Fence {
    // Check the fence's tasks' states
    pub fn state(&self) -> FenceState {
        todo!()
    }


    // Force the GPU to complete the fence's task (and every task before it), but don't wait
    pub fn flush(self) {
        todo!()
    }

    // Force the GPU to complete the tasks, and wait for it's completions
    pub fn wait(self) {
        todo!()
    }
}