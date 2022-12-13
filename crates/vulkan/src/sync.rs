use ahash::AHashMap;
use ash::vk;
use crate::{Access, BufferAccess, State, CompletedState, Barrier};

// proto-barrier for mut access of buffer2
// proto-barrier for ref access of buffer1
// vkCopyBuffer(src = buffer1, dst = buffer2)
// vkDispatch
// vkCopyImageToBuffer(src = image1, dst = buffer 3)

// proto-barrier for mut access of buffer1
// proto-barrier for ref access of buffer2
// vkCopyBuffer(src = buffer2, dst = buffer1)

// Create a prototype barrier for a specific access BEFORE the command
fn prototype(access: &Access, states: &mut AHashMap::<vk::Buffer, (vk::AccessFlags2, vk::PipelineStageFlags2)>) -> Barrier {
    match access {
        Access::Buffer(buffer) => {
            // Get the old state and access of the buffer
            //let (old_access_flags, old_stage_flags) = states.get(&buffer.buffer).unwrap();
            let BufferAccess { flags, stage, buffer, mutable, size, offset } = buffer;

            let barrier = vk::BufferMemoryBarrier2::builder()
                .buffer(*buffer)
                .dst_access_mask(*flags)
                .dst_stage_mask(*stage)
                .size(*size)
                .offset(*offset);

            let barrier = if let Some((flags, state)) = states.get(&buffer) {
                barrier.src_access_mask(*flags).src_stage_mask(*state)
            } else {
                barrier
            };

            dbg!(&*barrier);
            states.insert(*buffer, (*flags, *stage));

            Barrier {
                dependency_flags: vk::DependencyFlags::empty(),
                memory_barriers: Default::default(),
                buffer_memory_barriers: vec![*barrier],
                image_memory_barriers: Default::default(),
            }
        },
    }
}

// Convert the locally stored command to local groups that automatically place barriers within them
pub(super) fn complete(state: State) -> CompletedState {
    // Keep track of what buffers have mutable access and what stage needs mutable access
    let mut exclusive_buffer_access = AHashMap::<vk::Buffer, (vk::AccessFlags2, vk::PipelineStageFlags2)>::new();

    // Pipeline barrier that must be placed before commands
    let mut commands_test = AHashMap::<usize, Barrier>::new();

    // Create a prototype barrier for each access
    for (access, command) in &state.access {
        let other = prototype(access, &mut exclusive_buffer_access);

        match commands_test.entry(*command) {
            std::collections::hash_map::Entry::Occupied(mut current) => {
                current.get_mut().combine(other); 
            },
            std::collections::hash_map::Entry::Vacant(empty) => { empty.insert(other); },
        }
    }

    let commands = state
        .commands
        .into_iter()
        .enumerate()
        .map(|(i, command)| (vec![command], Some(commands_test.remove(&i).unwrap())));

        


    CompletedState {
        groups: commands.collect(),
    }
}