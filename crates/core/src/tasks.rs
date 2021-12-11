use crate::communication::*;
use crate::system::{IS_MAIN_THREAD, SYSTEM_GROUP_THREAD_DATA};

// Some world tasks
pub enum Task {
    // Entity
    EntityAdd(ecs::Entity, ecs::ComponentLinkingGroup),
    EntityRemove(usize),
    // This is only valid if the entity is also valid
    ComponentLinkDirect(usize, usize),
    ComponentUnlinkDirect(usize, usize),
    // UI
    AddRoot(String, ui::Root),
    SetRootVisibility(bool),
    // Main
    CreateConfigFile(),
}
// And their corresponding output
pub enum TaskReturn {
    // Entity
    CreateEntity(usize),
    DestroyEntity(Option<()>),
}
// The return type for a world task, we can wait for the return or just not care lol
pub struct WaitableTask {
    pub id: u64,
    pub val: Option<TaskReturn>,
    pub thread_id: std::thread::ThreadId,
}

impl WaitableTask {
    // Wait for the main thread to finish this specific task
    pub fn wait(self) -> TaskReturn {
        /* #region We are already on the main thread */
        // No need to do any of this multithreading shit if we're already on the main thread
        let x = IS_MAIN_THREAD.with(|x| x.get());
        if x {
            return self.val.unwrap();
        }
        /* #endregion */
        /* #region Wait for the main thread to send a return task */
        // Wait for the main thread to send back the return task
        let _sender = SENDER.lock().unwrap();
        let rx = SYSTEM_GROUP_THREAD_DATA.with(|x| {
            let system_group_thread_data = x.borrow();
            let rx = system_group_thread_data.rx.as_ref().unwrap().clone();
            rx
        });
        let _thread_id = std::thread::current().id();
        let id = self.id;
        loop {
            // Receive infinitely until we get the valid return task value
            match rx.try_recv() {
                Ok(x) => {
                    // Either add this to the buffer and continue the loop or return early
                    if x.id == id {
                        // The same ID, we can exit early
                        return x.val.unwrap();
                    } else {
                        // Add it to the buffer
                        let id = x.id;
                        SYSTEM_GROUP_THREAD_DATA.with(|data| {
                            let mut data = data.borrow_mut();
                            data.buffer.insert(id, x);
                        })
                    }
                }
                Err(_) => {
                    // Handle error
                }
            }
            let x: Option<TaskReturn> = SYSTEM_GROUP_THREAD_DATA.with(|data| {
                // Always check if the current group thread data contains our answer
                let mut data = data.borrow_mut();
                if data.buffer.contains_key(&id) {
                    // We found our answer!
                    let x = data.buffer.remove(&id).unwrap();
                    Some(x.val.unwrap())
                } else {
                    None
                }
            });
            match x {
                Some(x) => return x, /* The buffer does indeed contain the result */
                None => todo!(),
            }
        }
        /* #endregion */
    }
}

// Excecute a specific task and give back it's result
pub fn excecute_task(t: Task, _world: &mut crate::world::World) -> TaskReturn {
    match t {
        Task::EntityAdd(_, _) => todo!(),
        Task::EntityRemove(_) => todo!(),
        Task::ComponentLinkDirect(_, _) => todo!(),
        Task::ComponentUnlinkDirect(_, _) => todo!(),
        Task::AddRoot(_, _) => todo!(),
        Task::SetRootVisibility(_) => todo!(),
        Task::CreateConfigFile() => todo!(),
    }
}
