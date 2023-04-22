use std::{any::Any, sync::Arc};

// Shared arc that represents a pointer tuple
pub(super) type BoxedPtrTuple = Arc<dyn Any + Send + Sync + 'static>;
pub(super) type ArcFn<'b> = Arc<dyn Fn(ThreadFuncEntry) + Send + Sync + 'b>;

// Data passed to each thread
pub(super) struct ThreadFuncEntry {
    pub(super) base: BoxedPtrTuple,
    pub(super) batch_length: usize,
    pub(super) batch_offset: usize,
}

// Shared arc that represents a shared function
pub(super) type BoxedFunction = Arc<dyn Fn(ThreadFuncEntry) + Send + Sync + 'static>;

// Represents a single task that will be executed by multiple threads
pub(super) enum ThreadedTask {
    // This is a single task that will be executed on a single thread
    Execute(Box<dyn FnOnce() + Send>),

    // Executed in multiple threads
    ForEachBatch {
        entry: ThreadFuncEntry,
        function: BoxedFunction,
    },
}

// Execute a threaded task
pub(super) fn execute(task: ThreadedTask) {
    match task {
        // Execute a single task in another thread
        ThreadedTask::Execute(f) => f(),

        // Execute the same function over and over again on the same slice, but at different indices (no overrun)
        ThreadedTask::ForEachBatch { entry, function } => function(entry),
    }
}
