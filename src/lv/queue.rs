use ash::vk;

pub struct Queue {
    pub handle: vk::Queue,
}

impl Queue {
    pub fn new(queue_family_index: u32, device: &ash::Device) -> Queue {
        let queue = unsafe { device.get_device_queue(queue_family_index, 0) };
        Queue { handle: queue }
    }
}
