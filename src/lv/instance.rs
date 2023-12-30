use std::ffi::CString;
use std::ptr;
use crate::{utility, VALIDATION, VulkanApp, WINDOW_TITLE};
use std::sync::{Arc, RwLock};
use ash::vk;

const validation_layer_name: &str = "VK_LAYER_KHRONOS_validation";

pub struct Instance {
    pub entry: ash::Entry,
    pub instance: ash::Instance,
}

impl Instance {
    
    pub fn new(required_extensions: Vec<*const i8>, validation_layers: bool) -> Self {
        let entry = ash::Entry::linked();
        if !validation_layers || !Instance::check_validation_layer_support(&entry, &vec![validation_layer_name.to_string()]) {
            panic!("Validation layer is enabled, but could not find validation layers!");
        }
        
        // Create instance
        let app_name = CString::new(WINDOW_TITLE).unwrap();
        let engine_name = CString::new("Vulkan Engine").unwrap();
        let app_info = vk::ApplicationInfo {
            s_type: vk::StructureType::APPLICATION_INFO,
            p_next: ptr::null(),
            p_application_name: app_name.as_ptr(),
            application_version: 0,
            p_engine_name: engine_name.as_ptr(),
            engine_version: 0,
            api_version: vk::make_api_version(0, 1, 3, 0),
        };

        let required_extension_layer_names: Vec<CString> = VALIDATION
            .required_validation_layers
            .iter()
            .map(|layer_name| CString::new(*layer_name).unwrap())
            .collect();

        let enabled_layer_names: Vec<*const i8> = required_extension_layer_names
            .iter()
            .map(|layer_name| layer_name.as_ptr())
            .collect();

        let create_info = vk::InstanceCreateInfo {
            s_type: vk::StructureType::INSTANCE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::InstanceCreateFlags::empty(),
            p_application_info: &app_info,
            enabled_layer_count: if VALIDATION.is_enabled {
                enabled_layer_names.len()
            } else {
                0
            } as u32,
            pp_enabled_layer_names: if VALIDATION.is_enabled {
                enabled_layer_names.as_ptr()
            } else {
                ptr::null()
            },
            pp_enabled_extension_names: required_extensions.as_ptr(),
            enabled_extension_count: required_extensions.len() as u32,
        };

        let instance: ash::Instance = unsafe {
            entry
                .create_instance(&create_info, None)
                .expect("Failed to create instance")
        };

        Self {
            entry,
            instance
        }
    }
    
    pub fn check_validation_layer_support(entry: &ash::Entry, required_layers: &[String]) -> bool {
        let layer_properties = entry
            .enumerate_instance_layer_properties()
            .expect("Failed to enumerate instance layer properties");

        if layer_properties.is_empty() {
            eprintln!("No available layers.");
            return false;
        } else {
            for required_layer_name in required_layers {
                let mut is_layer_found = false;
                for layer_property in layer_properties.iter() {
                    let test_layer_name = utility::tools::vk_to_string(&layer_property.layer_name);
                    if (*required_layer_name) == test_layer_name {
                        is_layer_found = true;
                        break;
                    }
                }

                if !is_layer_found {
                    return false;
                }
            }
        }

        true
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        unsafe {
            self.instance.destroy_instance(None);
        }
    }
}
