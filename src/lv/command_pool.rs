use crate::lv;
use ash::vk;
use std::sync::Arc;

pub struct CommandPool {
    handle: vk::CommandPool,
    device: Arc<lv::Device>,
}

impl CommandPool {
    pub fn new(
        flags: vk::CommandPoolCreateFlags,
        queue: &lv::Queue,
        device: Arc<lv::Device>,
    ) -> Self {
        let pool_ci = vk::CommandPoolCreateInfo {
            s_type: vk::StructureType::COMMAND_POOL_CREATE_INFO,
            flags,
            queue_family_index: queue.index,
            ..Default::default()
        };
        let pool = unsafe { device.handle.create_command_pool(&pool_ci, None).unwrap() };
        CommandPool {
            handle: pool,
            device,
        }
    }
	
	pub fn get_handle(&self) -> vk::CommandPool {
		self.handle
	}
}

impl Drop for CommandPool {
    fn drop(&mut self) {
        unsafe {
            self.device.handle.destroy_command_pool(self.handle, None);
        };
    }
}
