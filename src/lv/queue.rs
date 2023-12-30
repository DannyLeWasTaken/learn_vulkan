use ash::vk;

#[derive(Clone)]
pub struct Queue {
    pub handle: vk::Queue,
    pub index: u32,
}

impl Queue {
    pub fn new(queue_family_index: u32, device: &ash::Device) -> Queue {
        let queue = unsafe { device.get_device_queue(queue_family_index, 0) };
        Queue {
            handle: queue,
            index: queue_family_index,
        }
    }
}
