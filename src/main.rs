use std::ffi::{CString, CStr};

use ash::{self, vk};
use utility::tools;
use winit::{self};
use std::ptr;
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};

mod utility;

// Constants
const WINDOW_TITLE: &'static str = "Hello, Vulkan!";
const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 600;

struct ValidationInfo {
    pub is_enabled: bool,
    pub required_validation_layers: [&'static str; 1],
}

struct VulkanApp {
    _entry: ash::Entry,
    instance: ash::Instance,
}

const VALIDATION: ValidationInfo = ValidationInfo {
    is_enabled: true,
    required_validation_layers: ["VK_LAYER_KHRONOS_validation"],
};

impl VulkanApp {
    pub fn new(window: &winit::window::Window) -> VulkanApp {
        // Init vulkan stuff
        let entry = ash::Entry::linked();
        let instance = VulkanApp::create_instance(&entry, window);

        VulkanApp {
            _entry: entry,
            instance: instance
        }
    }

    fn check_validation_layer_support(entry: &ash::Entry) -> bool {
        let layer_count: u32;
        let layer_properties = entry
            .enumerate_instance_layer_properties()
            .expect("Failed to enumerate instance layer properties");
        
        if layer_properties.len() <= 0 {
            eprintln!("No avaliable layers.");
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
        let mut extensions_names = ash_window::enumerate_required_extensions(window.raw_display_handle()).unwrap().to_vec();
        extensions_names.push(ash::extensions::ext::DebugUtils::name().as_ptr());

        let create_info = vk::InstanceCreateInfo {
            s_type: vk::StructureType::INSTANCE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::InstanceCreateFlags::empty(),
            p_application_info: &app_info,
            enabled_layer_count: 0,
            pp_enabled_layer_names: ptr::null(),
            pp_enabled_extension_names: extensions_names.as_ptr(),
            enabled_extension_count: extensions_names.len() as u32,
        };

        let instance: ash::Instance = unsafe {
            entry
                .create_instance(&create_info, None)
                .expect("Failed to create instance")
        };

        instance
    }

    fn draw_frame(&mut self) {

    }

    pub fn main_loop(mut self, event_loop: winit::event_loop::EventLoop<()>, window: winit::window::Window) {
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
        event_loop.run(move |event, elwt|  {
            match event {
                winit::event::Event::WindowEvent { window_id, event } => {
                    match event {
                        winit::event::WindowEvent::CloseRequested => {
                            println!("Exiting application!");
                            elwt.exit();
                        },
                        winit::event::WindowEvent::RedrawRequested => {
                            window.request_redraw();
                        },
                        _ => (),
                    }
                },
                winit::event::Event::AboutToWait => {
                    window.request_redraw();
                },
                _ => ()
            }
        });
    }
}

impl Drop for VulkanApp {
    fn drop(&mut self) {
        unsafe {
            self.instance.destroy_instance(None);
        }
    }
}

fn main() {
    let event_loop = winit::event_loop::EventLoop::new().expect("Failed to make event loop");
    let window = VulkanApp::init_window(&event_loop);
    let vulkan_app = VulkanApp::new(&window);
    vulkan_app.main_loop(event_loop, window);
}
