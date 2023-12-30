use crate::lv;
use ash::vk;

pub struct CommandBuffer {
    handle: vk::CommandBuffer,
}

impl CommandBuffer {
    pub fn new(
        command_pool: &lv::CommandPool,
        level: vk::CommandBufferLevel,
        device: &lv::Device,
    ) -> Self {
        let command_buffer_ai = vk::CommandBufferAllocateInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
            command_pool: command_pool.get_handle(),
            level,
            command_buffer_count: 1,
            ..Default::default()
        };
        let handle = unsafe {
            device
                .handle
                .allocate_command_buffers(&command_buffer_ai)
                .unwrap()
                .pop()
                .unwrap()
        };
        CommandBuffer { handle }
    }

    pub fn get_handle(&self) -> vk::CommandBuffer {
        self.handle
    }
}
