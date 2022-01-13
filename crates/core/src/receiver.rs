use crate::task::WorldTask;


// A receiver that we can use to receive tasks from other threads
pub struct WorldTaskReceiver {
    // An internal receiver
    rx: std::sync::mpsc::Receiver<WorldTask>,
    // Buffered tasks that we can run at a later time
    tasks: Vec<WorldTask>,
}

impl WorldTaskReceiver {
    // Create a new world task receiver
    // Also setup the global channel
    pub fn new() -> Self {
        let (tx, rx) = std::sync::mpsc::channel::<WorldTask>();
        crate::sender::set_global_sender(tx);
        Self {
            rx,
            tasks: Vec::new(),
        }
    }
    // We will buffer the tasks that are stored in the receiver
    pub fn buffer(&mut self) {
        self.tasks.extend(self.rx.try_iter());
    }
    // We will flush the tasks, and execute them
    // This is called at the end of each system execution, since some tasks might need to execute earlier than others
    pub fn flush(&mut self) {
        let taken = std::mem::take(&mut self.tasks);
        for task in taken {
            // We will execute the tasks
            match task {
                WorldTask::AddEntity(entity, id, group) => {
                    // We will add the entity to the world
                },
                WorldTask::RemoveEntity(id) => {
                    // We will remove the entity from the world
                },
                WorldTask::DirectAddComponent(_, _) => todo!(),
            }
        }
    }
}