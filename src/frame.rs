use crate::lv;

pub struct FrameData {
    pub pool: lv::CommandPool,
    pub main_command_buffer: lv::CommandBuffer,

    // Sync
    pub swapchain_semaphore: lv::Semaphore, // Indicate when image has been acquired
    pub render_semaphore: lv::Semaphore,    // Indicated when render of queue is done for GPU
    pub render_fence: lv::Fence,            // Indicate when render of queue is done for CPU
}
