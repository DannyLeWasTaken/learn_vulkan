use crate::lv;
use crate::utility::tools::vk_to_string;
use ash::vk;
use ash::vk::TaggedStructure;
use std::collections::{HashMap, HashSet};
use std::ffi::{c_char, c_void, CString};
use std::sync::Arc;

#[derive(Clone, Copy)]
pub struct QueueFamilyIndices {
    pub graphics_family: Option<u32>,
    pub present_family: Option<u32>,
}

pub struct PhysicalDevice {
    pub handle: vk::PhysicalDevice,
    pub properties: vk::PhysicalDeviceProperties2,
    pub features: vk::PhysicalDeviceFeatures2,
    pub features_1_3: vk::PhysicalDeviceVulkan13Features,
    pub features_1_2: vk::PhysicalDeviceVulkan12Features,
    pub features_1_1: vk::PhysicalDeviceVulkan11Features,
    pub queue_families: QueueFamilyIndices,
    pub swapchain_support: Option<lv::SwapchainSupportDetails>,

    // Reference-counting
    instance: Arc<lv::Instance>,
}

impl PhysicalDevice {
    pub fn new(vk_device: vk::PhysicalDevice, lv: Arc<lv::Instance>) -> PhysicalDevice {
        let mut physical_device_properties: vk::PhysicalDeviceProperties2 = Default::default();
        unsafe {
            lv.instance
                .get_physical_device_properties2(vk_device, &mut physical_device_properties);
        };
        let mut physical_device_features: vk::PhysicalDeviceFeatures2 = Default::default();
        let mut features_1_3 = vk::PhysicalDeviceVulkan13Features::default();
        let mut features_1_2 = vk::PhysicalDeviceVulkan12Features::default();
        let mut features_1_1 = vk::PhysicalDeviceVulkan11Features::default();
        physical_device_features.p_next = &mut features_1_3 as *mut _ as *mut c_void;
        features_1_3.p_next = &mut features_1_2 as *mut _ as *mut c_void;
        features_1_2.p_next = &mut features_1_1 as *mut _ as *mut c_void;
        unsafe {
            lv.instance
                .get_physical_device_features2(vk_device, &mut physical_device_features);
        };

        PhysicalDevice {
            handle: vk_device,
            instance: lv,
            properties: physical_device_properties,
            features: physical_device_features,
            features_1_3,
            features_1_2,
            features_1_1,
            queue_families: QueueFamilyIndices {
                graphics_family: None,
                present_family: None,
            },
            swapchain_support: None,
        }
    }

    pub fn find_queue_families(
        &mut self,
        surface_loader: &ash::extensions::khr::Surface,
        surface: vk::SurfaceKHR,
    ) {
        let queue_family_properties = unsafe {
            self.instance
                .instance
                .get_physical_device_queue_family_properties(self.handle)
        };
        for (index, queue_family) in queue_family_properties.iter().enumerate() {
            if queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                self.queue_families.graphics_family = Some(index as u32);
            }
            if unsafe {
                surface_loader
                    .get_physical_device_surface_support(self.handle, index as u32, surface)
                    .unwrap()
            } {
                self.queue_families.present_family = Some(index as u32);
            }
        }
    }
    pub fn has_extensions(&self, extensions: &[String]) -> bool {
        let available_extensions = unsafe {
            self.instance
                .instance
                .enumerate_device_extension_properties(self.handle)
                .unwrap()
        };

        let mut available_extensions_names: Vec<String> = vec![];
        for extension in available_extensions.iter() {
            let extension_name = vk_to_string(&extension.extension_name);
            available_extensions_names.push(extension_name);
        }

        let mut required_extensions = HashSet::new();
        for extension in extensions.iter() {
            required_extensions.insert(extension.to_string());
        }

        for extension_name in available_extensions_names.iter() {
            required_extensions.remove(extension_name);
        }

        required_extensions.is_empty()
    }

    pub fn get_swapchain_support(
        &self,
        surface_loader: &ash::extensions::khr::Surface,
        surface: vk::SurfaceKHR,
    ) -> lv::SwapchainSupportDetails {
        let support = lv::SwapchainSupportDetails {
            capabilities: unsafe {
                surface_loader
                    .get_physical_device_surface_capabilities(self.handle, surface)
                    .unwrap()
            },
            formats: unsafe {
                surface_loader
                    .get_physical_device_surface_formats(self.handle, surface)
                    .unwrap()
            },
            present_modes: unsafe {
                surface_loader
                    .get_physical_device_surface_present_modes(self.handle, surface)
                    .unwrap()
            },
        };
        support
    }
}

pub struct Device {
    pub handle: ash::Device,
    pub queues: HashMap<u32, lv::Queue>,

    // Reference-count
    instance: Arc<lv::Instance>,
    physical_device: Arc<PhysicalDevice>,
}

impl Device {
    pub fn new(
        physical_device: Arc<PhysicalDevice>,
        required_extensions: Option<Vec<String>>,
        instance: Arc<lv::Instance>,
    ) -> Arc<Device> {
        // Determine which queue family to use
        let queue_families: QueueFamilyIndices = physical_device.queue_families;
        // TODO: deal with multiple queues
        let unique_queue_families = vec![
            queue_families.graphics_family.unwrap(),
            queue_families.present_family.unwrap(),
        ];
        let mut queue_cis: Vec<vk::DeviceQueueCreateInfo> =
            Vec::with_capacity(unique_queue_families.len());
        for unique_queue in unique_queue_families {
            let queue_ci = vk::DeviceQueueCreateInfo {
                s_type: vk::StructureType::DEVICE_QUEUE_CREATE_INFO,
                queue_family_index: unique_queue,
                queue_count: 1,
                p_queue_priorities: &1.0,
                ..vk::DeviceQueueCreateInfo::default()
            };

            queue_cis.push(queue_ci);
        }
        let cstring_ext_names: Vec<CString> = required_extensions
            .unwrap_or_default()
            .iter()
            .map(|s| CString::new(s.clone()).unwrap())
            .collect();
        let c_str_ptrs: Vec<*const c_char> = cstring_ext_names.iter().map(|s| s.as_ptr()).collect();

        let mut physical_device_features = physical_device.features;
        // Dynamic rendering is a feature expected at the very minimum
        let mut dynamic_rendering_feature = vk::PhysicalDeviceDynamicRenderingFeatures {
            s_type: vk::StructureType::PHYSICAL_DEVICE_DYNAMIC_RENDERING_FEATURES,
            dynamic_rendering: vk::TRUE,
            ..Default::default()
        };
        // Same with sync2
        let mut sync_2 = vk::PhysicalDeviceSynchronization2Features {
            s_type: vk::StructureType::PHYSICAL_DEVICE_SYNCHRONIZATION_2_FEATURES,
            synchronization2: vk::TRUE,
            ..Default::default()
        };
        let mut bda = vk::PhysicalDeviceBufferDeviceAddressFeatures {
            s_type: vk::StructureType::PHYSICAL_DEVICE_BUFFER_DEVICE_ADDRESS_FEATURES,
            buffer_device_address: vk::TRUE,
            buffer_device_address_capture_replay: vk::TRUE,
            ..Default::default()
        };
        let mut descriptor_indexing = vk::PhysicalDeviceDescriptorIndexingFeatures {
            s_type: vk::PhysicalDeviceDescriptorIndexingFeatures::STRUCTURE_TYPE,
            shader_sampled_image_array_non_uniform_indexing: vk::TRUE,
            shader_storage_image_array_non_uniform_indexing: vk::TRUE,
            descriptor_binding_sampled_image_update_after_bind: vk::TRUE,
            descriptor_binding_storage_image_update_after_bind: vk::TRUE,
            descriptor_binding_update_unused_while_pending: vk::TRUE,
            descriptor_binding_partially_bound: vk::TRUE,
            ..Default::default()
        };
        dynamic_rendering_feature.p_next = &mut sync_2 as *mut _ as *mut c_void;
        physical_device_features.p_next = &mut dynamic_rendering_feature as *mut _ as *mut c_void;
        sync_2.p_next = &mut bda as *mut _ as *mut c_void;
        bda.p_next = &mut descriptor_indexing as *mut _ as *mut c_void;

        let device_ci = vk::DeviceCreateInfo {
            s_type: vk::StructureType::DEVICE_CREATE_INFO,
            p_next: &physical_device_features as *const _ as *const c_void,
            p_queue_create_infos: queue_cis.as_ptr(),
            queue_create_info_count: queue_cis.len() as u32,
            enabled_extension_count: c_str_ptrs.len() as u32,
            pp_enabled_extension_names: c_str_ptrs.as_ptr(),
            ..vk::DeviceCreateInfo::default()
        };
        let device = unsafe {
            instance
                .instance
                .create_device(physical_device.handle, &device_ci, None)
                .unwrap()
        };
        let mut queues = HashMap::new();
        queues.insert(
            queue_families.graphics_family.unwrap(),
            lv::Queue::new(queue_families.graphics_family.unwrap(), &device),
        );
        queues.insert(
            queue_families.present_family.unwrap(),
            lv::Queue::new(queue_families.present_family.unwrap(), &device),
        );

        Arc::new(Device {
            handle: device,
            queues,
            instance: instance.clone(),
            physical_device: physical_device.clone(),
        })
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe { self.handle.destroy_device(None) };
    }
}
