use crate::lv;
use ash::vk;
use ash::vk::BindIndexBufferIndirectCommandNV;
use std::ptr;
use std::sync::Arc;

#[derive(Clone)]
pub struct QueueFamilyIndices {
    pub graphics_family: Option<u32>,
}

pub struct PhysicalDevice {
    pub handle: vk::PhysicalDevice,
    pub properties: vk::PhysicalDeviceProperties,
    pub features: vk::PhysicalDeviceFeatures,
    pub queue_families: QueueFamilyIndices,
    _ash: Arc<lv::lv>,
}

impl PhysicalDevice {
    pub fn new(vk_device: vk::PhysicalDevice, _ash: Arc<lv::lv>) -> PhysicalDevice {
        let mut physical_device_properties = unsafe {
            _ash.instance
                .read()
                .unwrap()
                .get_physical_device_properties(vk_device)
        };
        let mut physical_device_features = unsafe {
            _ash.instance
                .read()
                .unwrap()
                .get_physical_device_features(vk_device)
        };

        PhysicalDevice {
            handle: vk_device,
            properties: physical_device_properties,
            features: physical_device_features,
            queue_families: QueueFamilyIndices {
                graphics_family: None,
            },
            _ash,
        }
    }

    pub fn find_queue_families(&mut self) {
        let queue_family_properties = unsafe {
            self._ash
                .instance
                .read()
                .unwrap()
                .get_physical_device_queue_family_properties(self.handle)
        };
        for (index, queue_family) in queue_family_properties.iter().enumerate() {
            if queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                self.queue_families.graphics_family = Some(index as u32);
                break;
            }
        }
    }
}

pub struct Device {
    pub handle: ash::Device,
    pub physical_device: Arc<PhysicalDevice>,
    _ash: Arc<lv::lv>,
}

impl Device {
    pub fn new(
        physical_device: Arc<PhysicalDevice>,
        in_queue_families: Option<QueueFamilyIndices>,
        lv: Arc<lv::lv>,
    ) -> Device {
        // Determine which queue family to use
        let queue_families: QueueFamilyIndices;
        if (in_queue_families.is_none()) {
            // use default queue families
            queue_families = physical_device.queue_families.clone();
        } else {
            queue_families = in_queue_families.unwrap();
        }

        // TODO: deal with multiple queues
        let mut queue_ci = vk::DeviceQueueCreateInfo::default();
        queue_ci.s_type = vk::StructureType::DEVICE_QUEUE_CREATE_INFO;
        queue_ci.queue_family_index = queue_families.graphics_family.unwrap();
        queue_ci.queue_count = 1;
        queue_ci.p_queue_priorities = &1.0;

        let physical_device_features = vk::PhysicalDeviceFeatures::default();
        let mut device_ci = vk::DeviceCreateInfo::default();
        device_ci.s_type = vk::StructureType::DEVICE_CREATE_INFO;
        device_ci.p_queue_create_infos = &queue_ci;
        device_ci.queue_create_info_count = 1;
        device_ci.p_enabled_features = &physical_device_features;

        let device = unsafe {
            lv.instance
                .read()
                .unwrap()
                .create_device(physical_device.handle, &device_ci, None)
                .unwrap()
        };

        Device {
            handle: device,
            physical_device,
            _ash: lv,
        }
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe { self.handle.destroy_device(None) };
    }
}
