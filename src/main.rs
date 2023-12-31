use std::ffi::{c_char, CString};
use std::ptr;

use ash::{self, vk};
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use std::rc::Rc;
use std::sync::Arc;
use winit::{self};

mod lv;
mod utility;

// Constants
const WINDOW_TITLE: &str = "Hello, Vulkan!";
const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 600;
const MAX_FRAIMES_IN_FLIGHT: u32 = 2;

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
    swapchain: lv::Swapchain,
    pipeline: Rc<lv::Pipeline>,
    command_pool: lv::CommandPool,
    command_buffer: lv::CommandBuffer,

    // Synchronization primitives (i don't like this but lol)
    image_available_semaphore: lv::Semaphore,
    render_finished_semaphore: lv::Semaphore,
    in_flight_fence: lv::Fence,
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
            ash::extensions::khr::Synchronization2::name()
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
            surface.clone(),
            lv::SwapchainPreferred {
                swapchain_support_details: swapchain_support.clone(),
                preferred_format: &[vk::Format::R8G8B8_SRGB],
                preferred_present_modes: &[vk::PresentModeKHR::MAILBOX],
            },
            window,
        );
        let pipeline =
            VulkanApp::create_graphics_pipeline(logical_device.clone(), &swapchain_support);
        let command_pool = lv::CommandPool::new(
            vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER,
            logical_device
                .queues
                .get(&physical_device.queue_families.graphics_family.unwrap())
                .unwrap(),
            logical_device.clone(),
        );
        let command_buffer = lv::CommandBuffer::new(
            &command_pool,
            vk::CommandBufferLevel::PRIMARY,
            &logical_device,
        );
        let image_available_semaphore = lv::Semaphore::new(logical_device.clone());
        let render_finished_semaphore = lv::Semaphore::new(logical_device.clone());
        let in_flight_fence =
            lv::Fence::new(logical_device.clone(), Some(vk::FenceCreateFlags::SIGNALED));

        VulkanApp {
            handle: instance,
            debug_messenger,
            physical_device,
            logical_device,
            surface,
            swapchain,
            pipeline,
            command_pool,
            command_buffer,
            image_available_semaphore,
            render_finished_semaphore,
            in_flight_fence,
        }
    }

    fn record_commands(&self, index: usize) {
        let present_queue_index = self.physical_device.queue_families.present_family.unwrap();
        let graphics_queue_index = self.physical_device.queue_families.graphics_family.unwrap();

        let command_buffer = self.command_buffer.get_handle();
        let color_attachment = utility::init::attachment_info(
            *self.swapchain.image_views.get(index).unwrap(),
            Some(vk::ClearValue::default()),
            vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        );
        unsafe {
            let command_buffer_bi = vk::CommandBufferBeginInfo {
                s_type: vk::StructureType::COMMAND_BUFFER_BEGIN_INFO,
                flags: vk::CommandBufferUsageFlags::SIMULTANEOUS_USE,
                ..Default::default()
            };
            self.logical_device
                .handle
                .begin_command_buffer(command_buffer, &command_buffer_bi)
                .unwrap();
        }
        // Transition into writing
        utility::transition_image(
            &self.logical_device.handle,
            command_buffer,
            *self.swapchain.images.get(index).unwrap(),
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            vk::QUEUE_FAMILY_IGNORED,
            vk::QUEUE_FAMILY_IGNORED,
        );

        let rendering_info = vk::RenderingInfo {
            s_type: vk::StructureType::RENDERING_INFO,
            flags: vk::RenderingFlags::empty(),
            layer_count: 1,
            view_mask: 0,
            color_attachment_count: 1,
            render_area: vk::Rect2D {
                extent: self.swapchain.extent,
                offset: vk::Offset2D::default(),
            },
            p_color_attachments: &color_attachment,
            ..Default::default()
        };
        unsafe {
            self.logical_device
                .handle
                .cmd_begin_rendering(command_buffer, &rendering_info);
            self.logical_device.handle.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline.get_handle(),
            );
        }

        // Recall we have set viewport + scissor as dynamic and not fixed
        let viewports = [vk::Viewport {
            x: 0.0f32,
            y: 0.0f32,
            width: self.swapchain.extent.width as f32,
            height: self.swapchain.extent.height as f32,
            min_depth: 0.0,
            max_depth: 0.0,
        }];
        unsafe {
            self.logical_device
                .handle
                .cmd_set_viewport(command_buffer, 0, &viewports);
        }

        let scissors = [vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: self.swapchain.extent,
        }];
        unsafe {
            self.logical_device
                .handle
                .cmd_set_scissor(command_buffer, 0, &scissors);
        };

        // Draw
        unsafe {
            self.logical_device
                .handle
                .cmd_draw(command_buffer, 3, 1, 0, 0);
        };

        unsafe {
            self.logical_device.handle.cmd_end_rendering(command_buffer);

            utility::transition_image(
                &self.logical_device.handle,
                command_buffer,
                *self.swapchain.images.get(index).unwrap(),
                vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                vk::ImageLayout::PRESENT_SRC_KHR,
                vk::QUEUE_FAMILY_IGNORED,
                vk::QUEUE_FAMILY_IGNORED,
            );

            self.logical_device
                .handle
                .end_command_buffer(command_buffer)
                .unwrap();
        };
    }
    fn create_graphics_pipeline(
        device: Arc<lv::Device>,
        swapchain_support: &lv::SwapchainSupportDetails,
    ) -> Rc<lv::Pipeline> {
        let vertex_shader = lv::Shader::new(
            std::path::Path::new("./shaders/triangle.vert.spv"),
            device.clone(),
        );
        let shader_entry_point = CString::new("main").unwrap();
        let vert_shader_stage_info = vk::PipelineShaderStageCreateInfo {
            s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
            stage: vk::ShaderStageFlags::VERTEX,
            module: vertex_shader.handle,
            p_name: shader_entry_point.as_ptr(),
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
            p_name: shader_entry_point.as_ptr(),
            ..Default::default()
        };
        let shader_stages = vec![vert_shader_stage_info, fragment_shader_stage_info];
        let pipeline = Rc::new(
            lv::PipelineBuilder::new()
                .set_viewport_counts(1, 1)
                .dynamic_states(vec![vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR])
                .attach_shaders_stages(shader_stages)
                .color_attachments(
                    1,
                    swapchain_support
                        .formats
                        .iter()
                        .map(|format| format.format)
                        .collect(),
                )
                .build(device.clone()),
        );
        pipeline
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

    fn draw_frame(&mut self) {
        let wait_fences = [self.in_flight_fence.get_handle()];
        unsafe {
            self.logical_device
                .handle
                .wait_for_fences(&wait_fences, true, u64::MAX)
                .unwrap();
        };

        let (index, _) = unsafe {
            self.swapchain
                .get_loader()
                .acquire_next_image(
                    self.swapchain.handle,
                    u64::MAX,
                    self.image_available_semaphore.get_handle(),
                    vk::Fence::null(),
                )
                .unwrap()
        };
        let index = index as usize;
        let wait_semaphores = [self.image_available_semaphore.get_handle()];
        let signal_semaphore = [self.render_finished_semaphore.get_handle()];

        // reset command buffer to be recorded into
        unsafe {
            self.logical_device
                .handle
                .reset_command_buffer(
                    self.command_buffer.get_handle(),
                    vk::CommandBufferResetFlags::empty(),
                )
                .unwrap()
        }
        self.record_commands(index);

        // submitting the commands
        let submit_info = vk::SubmitInfo {
            s_type: vk::StructureType::SUBMIT_INFO,
            wait_semaphore_count: wait_semaphores.len() as u32,
            p_wait_semaphores: wait_semaphores.as_ptr(), // wait for this semaphore before
            // executing
            p_wait_dst_stage_mask: [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT].as_ptr(), // wait stage to wait
            // at. i.e. continue up until color attachment and wait for the semaphore to be signaled
            command_buffer_count: 1,
            p_command_buffers: [self.command_buffer.get_handle()].as_ptr(),

            // Indicate which semaphore is to be signaled after the queue is finished
            signal_semaphore_count: signal_semaphore.len() as u32,
            p_signal_semaphores: signal_semaphore.as_ptr(),
            ..Default::default()
        };
        unsafe {
            self.logical_device
                .handle
                .reset_fences(&wait_fences)
                .unwrap();

            self.logical_device
                .handle
                .queue_submit(
                    self.logical_device
                        .queues
                        .get(&self.physical_device.queue_families.graphics_family.unwrap())
                        .unwrap()
                        .handle,
                    &[submit_info],
                    self.in_flight_fence.get_handle(),
                )
                .unwrap()
        };

        let swapchains = [self.swapchain.handle];
        let present_info = vk::PresentInfoKHR {
            s_type: vk::StructureType::PRESENT_INFO_KHR,
            wait_semaphore_count: 1,
            p_wait_semaphores: signal_semaphore.as_ptr(),
            swapchain_count: 1,
            p_swapchains: swapchains.as_ptr(),
            p_image_indices: &index as *const _ as *const u32,
            ..Default::default()
        };

        unsafe {
            self.swapchain
                .get_loader()
                .queue_present(
                    self.logical_device
                        .queues
                        .get(&self.physical_device.queue_families.present_family.unwrap())
                        .unwrap()
                        .handle,
                    &present_info,
                )
                .unwrap();
        }
    }

    pub fn main_loop(
        &mut self,
        event_loop: winit::event_loop::EventLoop<()>,
        window: winit::window::Window,
    ) {
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
        event_loop
            .run(move |event, elwt| match event {
                winit::event::Event::WindowEvent { window_id, event } => match event {
                    winit::event::WindowEvent::CloseRequested => {
                        println!("Exiting application!");
                        unsafe {
                            self.logical_device.handle.device_wait_idle().unwrap();
                        };
                        elwt.exit();
                    }
                    winit::event::WindowEvent::RedrawRequested => {
                        self.draw_frame();
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
    let mut vulkan_app = VulkanApp::new(&window);
    vulkan_app.main_loop(event_loop, window);
}
