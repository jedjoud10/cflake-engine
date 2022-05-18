// This is a simple command stream that we can use to help facilitate flushing/executing some commands
// Everything within the command buffer will get flushed to the driver whenever we call flush()
pub struct CommandStream {
}


impl CommandStream {
    // Initialize a new command stream
    pub fn new(ctx: &mut Context, func: impl FnOnce(&mut Context)) -> Self {
        func(ctx)
    }
    
    // Flush the command stream and simply return a fence, telling us when all the tasks finish executing
    fn flush() -> Fence {

    }
}
