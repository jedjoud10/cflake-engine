use ahash::AHashMap;
use ash::vk;
use crate::{Access, BufferAccess, State, CompletedState, Barrier};

// Flags for either an image or a buffer
pub(crate) struct Flags {
    access: vk::AccessFlags2,
    pipeline: vk::PipelineStageFlags2,
}

// Buffer identifier for a specific buffer and a specific range
#[derive(Hash, PartialEq, Eq)]
pub(crate) struct BufferId {
    buffer: vk::Buffer,
    offset: u64,
    size: u64,
}

// Image identifier for a specific image and a specific range
#[derive(Hash, PartialEq, Eq)]
pub(crate) struct ImageId {
    image: vk::Image,
    range: vk::ImageSubresourceRange,
}

pub(crate) type BufferTrackers = AHashMap::<BufferId, Flags>;
pub(crate) type ImageTrackers = AHashMap::<ImageId, Flags>;
pub(crate) type OutBarriers = AHashMap::<usize, Vec<Barrier>>;

// Create a prototype barrier for a specific access BEFORE the command
pub(crate) fn prototype(access: &Access, trackers: &mut BufferTrackers) -> Option<Barrier> {
    match access {
        Access::Buffer(buffer) => {
            let BufferAccess { flags, stage, buffer, mutable, size, offset } = buffer;

            // This barrier will be placed BEFORE the command
            let barrier = vk::BufferMemoryBarrier2::builder()
                .buffer(*buffer)
                .dst_access_mask(*flags)
                .dst_stage_mask(*stage)
                .size(*size)
                .offset(*offset);

            // Update the barrier src fields if the buffer was already tracked
            let id = BufferId { buffer: *buffer, offset: *offset, size: *size  };
            let barrier = if let Some(Flags { access, pipeline }) = trackers.get(&id) {
                barrier.src_access_mask(*access).src_stage_mask(*pipeline)
            } else {
                barrier
            };

            // Update the tracked buffer values
            trackers.insert(id, Flags { access: *flags, pipeline: *stage });

            // If the buffer memory barrier is redundant, remove it
            if barrier.src_access_mask.is_empty() && barrier.src_stage_mask.is_empty() {
                return None;
            }

            // Return a proper pipeline barrier 
            Some(Barrier {
                dependency_flags: vk::DependencyFlags::empty(),
                memory_barriers: Default::default(),
                buffer_memory_barriers: vec![*barrier],
                image_memory_barriers: Default::default(),
            })
        },

        Access::Image(image) => {
        },
    }
}

// Convert the locally stored command to local groups that automatically place barriers within them
pub(super) fn complete(state: State) -> CompletedState {
    let mut buffers = BufferTrackers::new();
    let mut images = ImageTrackers::new();
    let mut barriers = OutBarriers::new();

    // Create a prototype barrier for each access
    for (access, command) in &state.access {
        let barrier = prototype(access, &mut buffers);

        // Skip adding the prototype barrier if it's not needed
        let barrier = if let Some(barrier) = barrier {
            barrier
        } else {
            continue;
        };

        match barriers.entry(*command) {
            std::collections::hash_map::Entry::Occupied(mut current) => {
                current.get_mut().push(barrier); 
            },
            std::collections::hash_map::Entry::Vacant(empty) => { empty.insert(vec![barrier]); },
        }
    }

    // Create the command groups and their barriers
    let groups = state
        .commands
        .into_iter()
        .enumerate()
        .map(|(i, command)| (vec![command], barriers.remove(&i).unwrap_or_default()))
        .collect::<Vec<_>>();
    

        


    CompletedState {
        groups,
    }
}