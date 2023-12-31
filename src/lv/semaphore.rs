use crate::lv;
use ash::vk;
use std::sync::Arc;

pub struct Semaphore {
    device: Arc<lv::Device>,
    handle: vk::Semaphore,
}

impl Semaphore {
    pub fn new(device: Arc<lv::Device>) -> Self {
        let semaphore_ci = vk::SemaphoreCreateInfo {
            s_type: vk::StructureType::SEMAPHORE_CREATE_INFO,
            flags: Default::default(),
            ..Default::default()
        };
        let handle = unsafe { device.handle.create_semaphore(&semaphore_ci, None).unwrap() };

        Semaphore { device, handle }
    }

    pub fn get_handle(&self) -> vk::Semaphore {
        self.handle
    }
}

impl Drop for Semaphore {
	fn drop(&mut self) {
		unsafe {
			self.device.handle.destroy_semaphore(self.handle, None);
		};
	}
}