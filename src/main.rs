use std::ffi::{CString, CStr};

use ash::{self, vk};
use ash_window;
use winit::{self};
use std::ptr;
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};

// Constants
const WINDOW_TITLE: &'static str = "Hello, Vulkan!";
const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 600;

struct VulkanApp {
    _entry: ash::Entry,
    instance: ash::Instance,
}

impl VulkanApp {
    pub fn new(window: &winit::window::Window) -> VulkanApp {
        // Init vulkan stuff
        let entry = unsafe {
            ash::Entry::load()
                .expect("Unable to load ash entry")
        };
        let instance = VulkanApp::create_instance(&entry, window);

        VulkanApp {
            _entry: entry,
            instance: instance
        }
    }

    fn init_window(event_loop: &winit::event_loop::EventLoop<()>) -> winit::window::Window {
        winit::window::WindowBuilder::new()
            .with_title(WINDOW_TITLE)
            .with_inner_size(winit::dpi::LogicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT))
            .build(event_loop)
            .expect("Failed to create window")
    }

    fn create_instance(entry: &ash::Entry, window: &winit::window::Window) -> ash::Instance {
        let app_name = CString::new(WINDOW_TITLE).unwrap();
        let engine_name = CString::new("Vulkan Engine").unwrap();
        let app_info = vk::ApplicationInfo {
            s_type: vk::StructureType::APPLICATION_INFO,
            p_next: ptr::null(),
            p_application_name: app_name.as_ptr(),
            application_version: vk::make_api_version(0, 1, 0, 0),
            p_engine_name: engine_name.as_ptr(),
            engine_version: vk::make_api_version(0, 1, 0, 0),
            api_version: vk::API_VERSION_1_3,
        };

        // Extensions
        let mut extensions_names = ash_window::enumerate_required_extensions(window.raw_display_handle()).unwrap().to_vec();
        extensions_names.push(ash::extensions::ext::DebugUtils::name().as_ptr());

        let create_info = vk::InstanceCreateInfo {
            s_type: vk::StructureType::INSTANCE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::InstanceCreateFlags::empty(),
            p_application_info: &app_info,
            pp_enabled_extension_names: ptr::null(),
            enabled_layer_count: 0,
            pp_enabled_layer_names: extensions_names.as_ptr(),
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
