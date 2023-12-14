use crate::lv;
use ash::vk;
use std::sync::Arc;

pub struct Queue {
    handle: vk::Queue,
    device: Arc<lv::Device>,
}

impl Queue {
    pub fn new(queue_family_indices: lv::QueueFamilyIndices, device: Arc<lv::Device>) -> Queue {
        let queue = unsafe {
            device
                .handle
                .get_device_queue(queue_family_indices.graphics_family.unwrap(), 0)
        };

        Queue {
            handle: queue,
            device,
        }
    }
}
