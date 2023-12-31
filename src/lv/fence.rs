use crate::lv;
use ash::vk;
use std::sync::Arc;

pub struct Fence {
    device: Arc<lv::Device>,
    handle: vk::Fence,
}

impl Fence {
    pub fn new(device: Arc<lv::Device>, flags: Option<vk::FenceCreateFlags>) -> Self {
        let fence_ci = vk::FenceCreateInfo {
            s_type: vk::StructureType::FENCE_CREATE_INFO,
            flags: flags.unwrap_or_default(),
            ..Default::default()
        };

        let handle = unsafe { device.handle.create_fence(&fence_ci, None).unwrap() };

        Fence { device, handle }
    }

    pub fn get_handle(&self) -> vk::Fence {
        self.handle
    }
}

impl Drop for Fence {
    fn drop(&mut self) {
        unsafe {
            self.device.handle.destroy_fence(self.handle, None);
        };
    }
}
