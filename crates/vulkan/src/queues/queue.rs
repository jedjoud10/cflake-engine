use crate::{Device, Recorder, Adapter};
use ash::vk;
use parking_lot::Mutex;

// Queue that we can submit work to
pub struct Queue {
    pub(crate) raw: vk::Queue,
}

impl Queue {
    // Create a new queue wrapper
    pub(super) unsafe fn new(
        qfi: u32,
        device: &Device,
    ) -> Self {
        let raw = device.device.get_device_queue(qfi, 0);
        Self {
            raw,
        }
    }

    // Submit multiples recorders to this queue
    // This will actually submit the recorders for execution
    pub unsafe fn submit<'d>(
        &self,
        device: &Device,
        signal_semaphores: &[vk::Semaphore],
        wait_semaphores: &[vk::Semaphore],
        wait_dst_stage_mask: &[vk::PipelineStageFlags],
        recorders: impl Iterator<Item = Recorder<'d>>,
        fence: Option<vk::Fence>,
    ) {
        // Submit the command buffers
        let cmds = recorders.map(|r| r.finish()).collect::<Vec<_>>();
        let submit_info = vk::SubmitInfo::builder()
            .signal_semaphores(signal_semaphores)
            .wait_semaphores(wait_semaphores)
            .wait_dst_stage_mask(wait_dst_stage_mask)
            .command_buffers(cmds.as_slice());
        log::debug!("Submitting {} command buffers to queue {:?}", cmds.len(), self.raw);

        // Convert to null if needed
        let fence = fence.unwrap_or_else(|| vk::Fence::null());

        // Submit the commands with the fence
        device
            .device
            .queue_submit(self.raw, &[*submit_info], fence)
            .unwrap();
    }
}
