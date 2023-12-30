use std::ffi::{c_char, CString};
use std::mem::swap;
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
fn convert_static_str_to_string(vec_str: &'static str) -> String {
    vec_str.to_string()
}

fn convert_i8_to_string(extensions: Vec<*const i8>) -> Vec<String> {
    extensions
        .iter()
        .map(|&ext| unsafe { std::ffi::CStr::from_ptr(ext).to_string_lossy().to_string() })
        .collect()
}

impl VulkanApp {
    pub fn new(window: &winit::window::Window) -> VulkanApp {
        // Init vulkan stuff
        let instance = Arc::new(lv::Instance::new(
            VulkanApp::get_required_extensions(window),
            true,
        ));

        let required_extensions = vec![
            ash::extensions::khr::Swapchain::name()
                .to_string_lossy()
                .into_owned(),
            ash::extensions::khr::DynamicRendering::name()
                .to_string_lossy()
                .into_owned(),
            // BDAs
            ash::extensions::khr::BufferDeviceAddress::name()
                .to_string_lossy()
                .into_owned(),
            // RT extensions
            ash::extensions::khr::DeferredHostOperations::name()
                .to_string_lossy()
                .into_owned(),
            ash::extensions::khr::AccelerationStructure::name()
                .to_string_lossy()
                .into_owned(),
            ash::extensions::khr::RayTracingPipeline::name()
                .to_string_lossy()
                .into_owned(),
        ];

        let surface_loader =
            ash::extensions::khr::Surface::new(&instance.entry, &instance.instance);
        let surface = lv::Surface::new(
            &instance,
            surface_loader.clone(),
            window.raw_display_handle(),
            window.raw_window_handle(),
        );
        let physical_device = VulkanApp::pick_physical_devices(
            instance.clone(),
            &surface_loader,
            surface.handle,
            required_extensions.clone(),
        )
        .unwrap();
        let logical_device = lv::Device::new(
            physical_device.clone(),
            Some(required_extensions),
            instance.clone(),
        );
        let debug_messenger = lv::DebugMessenger::new(instance.clone());

        let swapchain_loader =
            ash::extensions::khr::Swapchain::new(&instance.instance, &logical_device.handle);
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
            handle: instance,
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

    fn init_window(event_loop: &winit::event_loop::EventLoop<()>) -> winit::window::Window {
        winit::window::WindowBuilder::new()
            .with_title(WINDOW_TITLE)
            .with_inner_size(winit::dpi::LogicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT))
            .build(event_loop)
            .expect("Failed to create window")
    }

    fn is_device_suitable(
        physical_device: &mut lv::PhysicalDevice,
        surface_loader: &ash::extensions::khr::Surface,
        surface: vk::SurfaceKHR,
        required_extensions: &[String],
    ) -> bool {
        if physical_device.properties.properties.device_type == vk::PhysicalDeviceType::DISCRETE_GPU
            && physical_device.features.features.geometry_shader == vk::TRUE
            && physical_device.has_extensions(required_extensions)
        {
            // check surface support now
            physical_device.find_queue_families(surface_loader, surface);
            let swapchain_support = physical_device.get_swapchain_support(surface_loader, surface);
            if !swapchain_support.formats.is_empty()
                && !swapchain_support.present_modes.is_empty()
                && physical_device.queue_families.graphics_family.is_some()
                && physical_device.queue_families.present_family.is_some()
            {
                return true;
            }
        }
        false
    }

    fn pick_physical_devices(
        instance: Arc<lv::Instance>,
        surface_loader: &ash::extensions::khr::Surface,
        surface: vk::SurfaceKHR,
        required_extensions: Vec<String>,
    ) -> Option<Arc<lv::PhysicalDevice>> {
        let physical_devices = unsafe { instance.instance.enumerate_physical_devices().unwrap() };
        for physical_device in physical_devices {
            let mut lv_device = lv::PhysicalDevice::new(physical_device, instance.clone());
            if VulkanApp::is_device_suitable(
                &mut lv_device,
                surface_loader,
                surface,
                &required_extensions,
            ) {
                return Some(Arc::new(lv_device));
            }
        }
        None
    }

    fn get_required_extensions(window: &winit::window::Window) -> Vec<*const i8> {
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
