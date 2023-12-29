use crate::lv;
use ash::vk;
use std::ffi::{c_void, CStr};
use std::ptr;
use std::sync::Arc;

unsafe extern "system" fn vulkan_debug_utils_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _p_user_data: *mut c_void,
) -> vk::Bool32 {
    let severity = match message_severity {
        vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE => "[Verbose]",
        vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => "[Warning]",
        vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => "[Error]",
        vk::DebugUtilsMessageSeverityFlagsEXT::INFO => "[Info]",
        _ => "[Unknown]",
    };
    let types = match message_type {
        vk::DebugUtilsMessageTypeFlagsEXT::GENERAL => "[General]",
        vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE => "[Performance]",
        vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION => "[Validation]",
        _ => "[Unknown]",
    };
    let message = CStr::from_ptr((*p_callback_data).p_message);
    println!("[Debug]{}{} {:?}", severity, types, message);

    vk::FALSE
}

pub struct DebugMessenger {
    loader: ash::extensions::ext::DebugUtils,
    handle: vk::DebugUtilsMessengerEXT,

    // Reference-counting
    instance: Arc<lv::Instance>,
}

impl DebugMessenger {
    fn get_debug_create_info() -> vk::DebugUtilsMessengerCreateInfoEXT {
        vk::DebugUtilsMessengerCreateInfoEXT {
            s_type: vk::StructureType::DEBUG_UTILS_MESSENGER_CREATE_INFO_EXT,
            p_next: ptr::null(),
            flags: vk::DebugUtilsMessengerCreateFlagsEXT::empty(),
            message_severity: vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
            message_type: vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
                | vk::DebugUtilsMessageTypeFlagsEXT::DEVICE_ADDRESS_BINDING,
            pfn_user_callback: Some(vulkan_debug_utils_callback),
            p_user_data: ptr::null_mut(),
        }
    }

    pub fn new(instance: Arc<lv::Instance>) -> Arc<DebugMessenger> {
        // Check if validation layers support
        if !instance.check_validation_layer_support(vec!["VK_LAYER_KHRONOS_validation"]) {
            panic!("Validation layers were requested, but not found");
        }

        // Create debug messenger
        let debug_utils_loader =
            ash::extensions::ext::DebugUtils::new(&instance.entry, &instance.instance);

        let create_info = DebugMessenger::get_debug_create_info();
        let utils_messenger = unsafe {
            debug_utils_loader
                .create_debug_utils_messenger(&create_info, None)
                .expect("Debug utils could not be made")
        };

        Arc::new(DebugMessenger {
            loader: debug_utils_loader,
            handle: utils_messenger,
            instance: instance.clone(),
        })
    }
}

impl Drop for DebugMessenger {
    fn drop(&mut self) {
        unsafe {
            self.loader.destroy_debug_utils_messenger(self.handle, None);
        };
    }
}
