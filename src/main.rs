use std::ffi::{c_char, CString};
use std::ops::Deref;

use ash::{self, vk};
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use std::ptr;
use std::sync::{Arc, RwLock};
use winit::{self};

mod lv;
mod utility;

// Constants
const WINDOW_TITLE: &str = "Hello, Vulkan!";
const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 600;

struct ValidationInfo {
    pub is_enabled: bool,
    pub required_validation_layers: [&'static str; 1],
}

struct VulkanApp {
    handle: Arc<lv::Instance>,
    debug_messenger: Arc<lv::DebugMessenger>,
    surface: Arc<lv::Surface>,
    physical_device: Arc<lv::PhysicalDevice>,
    logical_device: Arc<lv::Device>,
}

const VALIDATION: ValidationInfo = ValidationInfo {
    is_enabled: true,
    required_validation_layers: ["VK_LAYER_KHRONOS_validation"],
};

const DEVICE_EXTENSIONS: [&str; 1] = ["VK_KHR_swapchain"];
fn convert_static_str_to_string(vec_str: [&'static str; 1]) -> Vec<String> {
    vec_str.into_iter().map(|s| s.to_string()).collect()
}

impl VulkanApp {
    pub fn new(window: &winit::window::Window) -> VulkanApp {
        // Init vulkan stuff
        let entry = ash::Entry::linked();
        let instance = VulkanApp::create_instance(&entry, window);
        let entry = Arc::new(lv::Instance { instance, entry });

        // debug messenger
        let debug_messenger = lv::DebugMessenger::new(entry.clone());
        let surface_loader = ash::extensions::khr::Surface::new(&entry.entry, &entry.instance);
        let surface = lv::Surface::new(
            &entry,
            surface_loader.clone(),
            window.raw_display_handle(),
            window.raw_window_handle(),
        );

        let physical_device =
            VulkanApp::pick_physical_device(entry.clone(), &surface_loader, surface.handle)
                .unwrap();
        let logical_device = lv::Device::new(
            physical_device.clone(),
            Some(convert_static_str_to_string(DEVICE_EXTENSIONS)),
            entry.clone(),
        );

        let swapchain_loader =
            ash::extensions::khr::Swapchain::new(&entry.instance, &logical_device.handle);
        let swapchain_support =
            physical_device.get_swapchain_support(&surface_loader, surface.handle);
        let swapchain = lv::Swapchain::new(
            swapchain_loader,
            &physical_device,
            logical_device.clone(),
            surface.handle,
            lv::SwapchainPreferred {
                swapchain_support_details: swapchain_support,
                preferred_format: &[vk::Format::R8G8B8_SRGB],
                preferred_present_modes: &[vk::PresentModeKHR::MAILBOX],
            },
            window,
        );

        VulkanApp {
            handle: entry,
            debug_messenger,
            physical_device,
            logical_device,
            surface,
        }
    }

    fn create_graphics_pipeline(entry: &ash::Entry, device: Arc<lv::Device>) {
        let vertex_shader = lv::Shader::new(
            std::path::Path::new("./shaders/triangle.vert.spv"),
            device.clone(),
        );
        let vert_shader_stage_info = vk::PipelineShaderStageCreateInfo {
            s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
            stage: vk::ShaderStageFlags::FRAGMENT,
            module: vertex_shader.handle,
            p_name: "main" as *const _ as *const c_char,
            ..Default::default()
        };
        let fragment_shader = lv::Shader::new(
            std::path::Path::new("./shaders/triangle.frag.spv"),
            device.clone(),
        );
        let fragment_shader_stage_info = vk::PipelineShaderStageCreateInfo {
            s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
            stage: vk::ShaderStageFlags::FRAGMENT,
            module: fragment_shader.handle,
            p_name: "main" as *const _ as *const c_char,
            ..Default::default()
        };
        let shader_stages = vec![vert_shader_stage_info, fragment_shader_stage_info];
    }

    fn check_validation_layer_support(entry: &ash::Entry) -> bool {
        let layer_properties = entry
            .enumerate_instance_layer_properties()
            .expect("Failed to enumerate instance layer properties");

        if layer_properties.is_empty() {
            eprintln!("No available layers.");
            return false;
        } else {
            for required_layer_name in VALIDATION.required_validation_layers.iter() {
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

    fn init_window(event_loop: &winit::event_loop::EventLoop<()>) -> winit::window::Window {
        winit::window::WindowBuilder::new()
            .with_title(WINDOW_TITLE)
            .with_inner_size(winit::dpi::LogicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT))
            .build(event_loop)
            .expect("Failed to create window")
    }

    fn create_instance(entry: &ash::Entry, window: &winit::window::Window) -> ash::Instance {
        if VALIDATION.is_enabled && !VulkanApp::check_validation_layer_support(entry) {
            panic!("Validation layers were requested, but not found");
        }

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

        // Extensions
        let extensions = VulkanApp::get_required_extensions(entry, window);

        let reqiured_validation_layer_raw_names: Vec<CString> = VALIDATION
            .required_validation_layers
            .iter()
            .map(|layer_name| CString::new(*layer_name).unwrap())
            .collect();

        let enabled_layer_names: Vec<*const i8> = reqiured_validation_layer_raw_names
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
            pp_enabled_extension_names: extensions.as_ptr(),
            enabled_extension_count: extensions.len() as u32,
        };

        let instance: ash::Instance = unsafe {
            entry
                .create_instance(&create_info, None)
                .expect("Failed to create instance")
        };

        instance
    }

    fn is_device_suitable(
        physical_device: &mut lv::PhysicalDevice,
        surface_loader: &ash::extensions::khr::Surface,
        surface: vk::SurfaceKHR,
    ) -> bool {
        physical_device.find_queue_families(surface_loader, surface);
        let swapchain_support = physical_device.get_swapchain_support(surface_loader, surface);

        if physical_device.properties.device_type == vk::PhysicalDeviceType::DISCRETE_GPU
            && physical_device.features.geometry_shader == vk::TRUE
            && physical_device.has_extensions(convert_static_str_to_string(DEVICE_EXTENSIONS))
            && !swapchain_support.formats.is_empty()
            && !swapchain_support.present_modes.is_empty()
            && physical_device.queue_families.graphics_family.is_some()
            && physical_device.queue_families.present_family.is_some()
        {
            return true;
        }
        false
    }

    fn pick_physical_device(
        instance: Arc<lv::Instance>,
        surface_loader: &ash::extensions::khr::Surface,
        surface: vk::SurfaceKHR,
    ) -> Option<Arc<lv::PhysicalDevice>> {
        let physical_devices = unsafe { instance.instance.enumerate_physical_devices().unwrap() };
        for physical_device in physical_devices {
            let mut lv_device = lv::PhysicalDevice::new(physical_device, instance.clone());
            if VulkanApp::is_device_suitable(&mut lv_device, surface_loader, surface) {
                return Some(Arc::new(lv_device));
            }
        }
        None
    }

    fn get_required_extensions(
        entry: &ash::Entry,
        window: &winit::window::Window,
    ) -> Vec<*const i8> {
        // Extensions
        let mut extensions_names =
            ash_window::enumerate_required_extensions(window.raw_display_handle())
                .unwrap()
                .to_vec();

        if VALIDATION.is_enabled {
            extensions_names.push(ash::extensions::ext::DebugUtils::name().as_ptr());
        }

        extensions_names
    }

    fn draw_frame(&mut self) {}

    pub fn main_loop(
        self,
        event_loop: winit::event_loop::EventLoop<()>,
        window: winit::window::Window,
    ) {
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
        event_loop
            .run(move |event, elwt| match event {
                winit::event::Event::WindowEvent { window_id, event } => match event {
                    winit::event::WindowEvent::CloseRequested => {
                        println!("Exiting application!");
                        elwt.exit();
                    }
                    winit::event::WindowEvent::RedrawRequested => {
                        window.request_redraw();
                    }
                    _ => (),
                },
                winit::event::Event::AboutToWait => {
                    window.request_redraw();
                }
                _ => (),
            })
            .expect("TODO: panic message");
    }
}

fn main() {
    let event_loop = winit::event_loop::EventLoop::new().expect("Failed to make event loop");
    let window = VulkanApp::init_window(&event_loop);
    let vulkan_app = VulkanApp::new(&window);
    vulkan_app.main_loop(event_loop, window);
}
