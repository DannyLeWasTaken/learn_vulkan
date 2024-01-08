use crate::frame::FrameData;
use ash::vk::TaggedStructure;
use ash::{self, vk};
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use std::ffi::CString;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use winit::{self};

mod frame;
mod lv;
mod utility;
mod vk_descriptors;

// Constants
const WINDOW_TITLE: &str = "Hello, Vulkan!";
const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 600;
const FRAME_OVERLAP: u32 = 2;

#[derive()]
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
    allocator: Arc<Mutex<gpu_allocator::vulkan::Allocator>>,
    swapchain: lv::Swapchain,

    draw_extent: vk::Extent2D,
    draw_image_index: u32,
    frames: Vec<FrameData>,
    frame_count: u64,

    gpu_resource_table: lv::descriptors::ShaRT,

    gradient_pipeline: Rc<lv::ComputePipeline>,
    triangle_pipeline: Rc<lv::Pipeline>,
}

const VALIDATION: ValidationInfo = ValidationInfo {
    is_enabled: true,
    required_validation_layers: ["VK_LAYER_KHRONOS_validation"],
};

#[repr(C)]
struct ComputePushConstants {
    data1: [f32; 4],
    data2: [f32; 4],
    data3: [f32; 4],
    data4: [f32; 4],
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
            /*
            ash::extensions::khr::DeferredHostOperations::name()
                .to_string_lossy()
                .into_owned(),
            ash::extensions::khr::AccelerationStructure::name()
                .to_string_lossy()
                .into_owned(),
            ash::extensions::khr::RayTracingPipeline::name()
                .to_string_lossy()
                .into_owned(),

             */
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
        let allocator = Arc::new(Mutex::new(
            gpu_allocator::vulkan::Allocator::new(&gpu_allocator::vulkan::AllocatorCreateDesc {
                instance: instance.instance.clone(),
                device: logical_device.handle.clone(),
                physical_device: physical_device.handle,
                debug_settings: Default::default(),
                buffer_device_address: true,
                allocation_sizes: Default::default(),
            })
            .unwrap(),
        ));

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
        // create image that is rendered to
        let draw_extent = swapchain.extent;
        let draw_image_extent = vk::Extent3D {
            height: swapchain.extent.height,
            width: swapchain.extent.width,
            depth: 1,
        };
        let draw_image = lv::AllocatedImage::new(
            utility::init::image_create_info(
                vk::Format::R16G16B16A16_SFLOAT,
                vk::ImageUsageFlags::TRANSFER_SRC
                    | vk::ImageUsageFlags::TRANSFER_DST
                    | vk::ImageUsageFlags::STORAGE
                    | vk::ImageUsageFlags::COLOR_ATTACHMENT,
                draw_image_extent,
            ),
            vk::ImageAspectFlags::COLOR,
            logical_device.clone(),
            allocator.clone(),
        );

        let triangle_pipeline = VulkanApp::create_triangle_pipeline(
            logical_device.clone(),
            &swapchain_support,
            draw_image.get_format(),
        );
        let mut frames: Vec<FrameData> = Vec::with_capacity(FRAME_OVERLAP as usize);
        for _ in 0..FRAME_OVERLAP {
            let pool = lv::CommandPool::new(
                vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER,
                logical_device
                    .queues
                    .get(&physical_device.queue_families.graphics_family.unwrap())
                    .unwrap(),
                logical_device.clone(),
            );
            let main_command_buffer =
                lv::CommandBuffer::new(&pool, vk::CommandBufferLevel::PRIMARY, &logical_device);
            let render_semaphore = lv::Semaphore::new(logical_device.clone(), None);
            let swapchain_semaphore = lv::Semaphore::new(logical_device.clone(), None);
            let render_fence =
                lv::Fence::new(logical_device.clone(), Some(vk::FenceCreateFlags::SIGNALED));

            frames.push(FrameData {
                pool,
                main_command_buffer,
                render_semaphore,
                render_fence,
                swapchain_semaphore,
            })
        }
        let (gpu_resource_table, draw_image_index) =
            VulkanApp::init_descriptors(logical_device.clone(), draw_image);
        let gradient_pipeline = VulkanApp::init_background_pipelines(
            logical_device.clone(),
            *gpu_resource_table.get_layout(),
        );
        let gradient_pipeline = Rc::new(gradient_pipeline);

        VulkanApp {
            handle: instance,
            debug_messenger,
            physical_device,
            logical_device,
            allocator,
            surface,
            swapchain,
            frames,
            draw_image_index,
            draw_extent,
            frame_count: 0,

            gpu_resource_table,

            gradient_pipeline,
            triangle_pipeline,
        }
    }

    fn init_background_pipelines(
        device: Arc<lv::Device>,
        descriptor_set_layout: vk::DescriptorSetLayout,
    ) -> lv::ComputePipeline {
        let draw_shader = lv::Shader::new(
            std::path::Path::new("./shaders/gradient.comp.spv"),
            device.clone(),
        );
        let shader_entry_point = CString::new("main").unwrap();
        let shader_stage_ci = vk::PipelineShaderStageCreateInfo {
            s_type: vk::PipelineShaderStageCreateInfo::STRUCTURE_TYPE,
            stage: vk::ShaderStageFlags::COMPUTE,
            module: draw_shader.handle,
            p_name: shader_entry_point.as_ptr(),
            ..Default::default()
        };
        let push_constant = vk::PushConstantRange {
            stage_flags: vk::ShaderStageFlags::COMPUTE,
            offset: 0,
            size: std::mem::size_of::<ComputePushConstants>() as u32,
        };
        println!(
            "Size: {:?}",
            std::mem::size_of::<ComputePushConstants>() as u32
        );
        let pipeline_builder = lv::ComputePipelineBuilder::new()
            .attach_stages(shader_stage_ci)
            .set_layouts(vec![descriptor_set_layout])
            .attach_push_constant(push_constant);
        let pipeline = lv::ComputePipeline::from_builder(pipeline_builder, device.clone());
        pipeline
    }

    fn init_descriptors(
        device: Arc<lv::Device>,
        image: lv::AllocatedImage,
    ) -> (lv::descriptors::ShaRT, u32) {
        let mut gpu_resource_table = lv::descriptors::ShaRT::new(device.clone());
        let id = gpu_resource_table.allocate_storage_image(image);
        gpu_resource_table.update();

        (gpu_resource_table, id)
    }

    fn get_current_frame(&self) -> &FrameData {
        self.frames
            .get((self.frame_count % self.frames.len() as u64) as usize)
            .unwrap()
    }

    fn draw_geometry(&self) {
        let command_buffer = self.get_current_frame().main_command_buffer.get_handle();
        let draw_image = self
            .gpu_resource_table
            .get_storage_image(self.draw_image_index as usize)
            .as_ref()
            .unwrap();
        let draw_image_attachment =
            utility::init::attachment_info(draw_image.get_view(), None, vk::ImageLayout::GENERAL);
        let rendering_info = vk::RenderingInfo {
            s_type: vk::RenderingInfo::STRUCTURE_TYPE,
            flags: Default::default(),
            render_area: vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: vk::Extent2D {
                    width: self.draw_extent.width,
                    height: self.draw_extent.height,
                },
            },
            layer_count: 1,
            view_mask: 0,
            color_attachment_count: 1,
            p_color_attachments: [draw_image_attachment].as_ptr(),
            ..Default::default()
        };
        unsafe {
            self.logical_device
                .handle
                .cmd_begin_rendering(command_buffer, &rendering_info);
            self.logical_device.handle.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.triangle_pipeline.get_handle(),
            )
        }
        let viewport = vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: self.draw_extent.width as f32,
            height: self.draw_extent.height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        };
        unsafe {
            self.logical_device
                .handle
                .cmd_set_viewport(command_buffer, 0, &[viewport]);
        }
        let scissor: vk::Rect2D = vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: vk::Extent2D::into(self.draw_extent),
        };
        unsafe {
            self.logical_device
                .handle
                .cmd_set_scissor(command_buffer, 0, &[scissor]);

            self.logical_device
                .handle
                .cmd_draw(command_buffer, 3, 1, 0, 0);

            self.logical_device.handle.cmd_end_rendering(command_buffer);
        }
    }
    fn draw_background(&self) {
        let command_buffer = self.get_current_frame().main_command_buffer.get_handle();
        unsafe {
            self.logical_device.handle.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::COMPUTE,
                self.gradient_pipeline.get_handle(),
            );
            self.logical_device.handle.cmd_bind_descriptor_sets(
                command_buffer,
                vk::PipelineBindPoint::COMPUTE,
                self.gradient_pipeline.get_layout(),
                0,
                &[*self.gpu_resource_table.get_descriptor()],
                &[],
            );
            let pc = ComputePushConstants {
                data1: glam::Vec4::new(1.0, 0.0, 0.0, 1.0).to_array(),
                data2: glam::Vec4::new(0.0, 0.0, 1.0, 1.0).to_array(),
                data3: glam::Vec4::ZERO.to_array(),
                data4: glam::Vec4::ZERO.to_array(),
            };
            self.logical_device.handle.cmd_push_constants(
                command_buffer,
                self.gradient_pipeline.get_layout(),
                vk::ShaderStageFlags::COMPUTE,
                0,
                std::slice::from_raw_parts(
                    &pc as *const _ as *const u8,
                    std::mem::size_of::<ComputePushConstants>(),
                ),
            );
            self.logical_device.handle.cmd_dispatch(
                command_buffer,
                (self.swapchain.extent.width as f32 / 16.0f32).ceil() as u32,
                (self.swapchain.extent.height as f32 / 16.0f32).ceil() as u32,
                1,
            );
        }
    }

    fn record_commands(&mut self, index: usize) {
        let command_buffer = self.get_current_frame().main_command_buffer.get_handle();
        self.draw_extent = self.swapchain.extent;

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

        let draw_image = self
            .gpu_resource_table
            .get_storage_image(self.draw_image_index as usize)
            .as_ref()
            .unwrap();

        // transition image from swapchain to be rendered into
        utility::transition_image(
            &self.logical_device.handle,
            command_buffer,
            draw_image.get_handle(),
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::GENERAL,
            vk::QUEUE_FAMILY_IGNORED,
            vk::QUEUE_FAMILY_IGNORED,
        );
        self.draw_background();

        utility::transition_image(
            &self.logical_device.handle,
            command_buffer,
            draw_image.get_handle(),
            vk::ImageLayout::GENERAL,
            vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            vk::QUEUE_FAMILY_IGNORED,
            vk::QUEUE_FAMILY_IGNORED,
        );
        self.draw_geometry();

        // transition image into their connect transfer layout
        utility::transition_image(
            &self.logical_device.handle,
            command_buffer,
            draw_image.get_handle(),
            vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
            vk::QUEUE_FAMILY_IGNORED,
            vk::QUEUE_FAMILY_IGNORED,
        );
        utility::transition_image(
            &self.logical_device.handle,
            command_buffer,
            *self.swapchain.images.get(index).unwrap(),
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            vk::QUEUE_FAMILY_IGNORED,
            vk::QUEUE_FAMILY_IGNORED,
        );

        // execute a copy from drawn image to present image
        utility::copy_image_to_image(
            command_buffer,
            &self.logical_device,
            draw_image.get_handle(),
            *self.swapchain.images.get(index).unwrap(),
            self.draw_extent,
            self.swapchain.extent,
        );

        // set swapchain image layout to Present so we can show it on the screen
        utility::transition_image(
            &self.logical_device.handle,
            command_buffer,
            *self.swapchain.images.get(index).unwrap(),
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            vk::ImageLayout::PRESENT_SRC_KHR,
            vk::QUEUE_FAMILY_IGNORED,
            vk::QUEUE_FAMILY_IGNORED,
        );

        unsafe {
            self.logical_device
                .handle
                .end_command_buffer(command_buffer)
                .unwrap()
        }
    }
    fn create_triangle_pipeline(
        device: Arc<lv::Device>,
        swapchain_support: &lv::SwapchainSupportDetails,
        color_format: vk::Format,
    ) -> Rc<lv::Pipeline> {
        let vertex_shader = lv::Shader::new(
            std::path::Path::new("./shaders/colored_triangle.vert.spv"),
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
            std::path::Path::new("./shaders/colored_triangle.frag.spv"),
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
        /*
        let formats: Vec<vk::Format> = swapchain_support
            .formats
            .iter()
            .map(|format| format.format)
            .collect();
         */
        let formats = vec![color_format];
        let builder = lv::PipelineBuilder::new()
            .dynamic_states(vec![vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR])
            .attach_shaders_stages(shader_stages)
            .color_attachments(formats.len() as u32, formats)
            .set_input_topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .set_polygon_mode(vk::PolygonMode::FILL)
            .set_cull_mode(vk::CullModeFlags::NONE, vk::FrontFace::CLOCKWISE)
            .set_multisampling_none()
            .disable_blending()
            .disable_depthtest();
        let pipeline = Rc::new(lv::Pipeline::from_builder(builder, device.clone()));
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
            && physical_device.features_1_3.synchronization2 == vk::TRUE
            && physical_device.features_1_3.dynamic_rendering == vk::TRUE
            && physical_device.features_1_2.buffer_device_address == vk::TRUE
            && physical_device.features_1_2.descriptor_indexing == vk::TRUE
            && physical_device.features_1_2.runtime_descriptor_array == vk::TRUE
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
        let render_fences = [self.get_current_frame().render_fence.get_handle()];
        unsafe {
            self.logical_device
                .handle
                .wait_for_fences(&render_fences, true, u64::MAX)
                .unwrap();
        };

        let (index, _) = unsafe {
            self.swapchain
                .get_loader()
                .acquire_next_image(
                    self.swapchain.handle,
                    u64::MAX,
                    self.get_current_frame().swapchain_semaphore.get_handle(),
                    vk::Fence::null(),
                )
                .unwrap()
        };
        let index = index as usize;
        let wait_semaphores = [self.get_current_frame().swapchain_semaphore.get_handle()];
        let signal_semaphore = [self.get_current_frame().render_semaphore.get_handle()];

        // reset command buffer to be recorded into
        unsafe {
            self.logical_device
                .handle
                .reset_command_buffer(
                    self.get_current_frame().main_command_buffer.get_handle(),
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
            p_command_buffers: [self.get_current_frame().main_command_buffer.get_handle()].as_ptr(),

            // Indicate which semaphore is to be signaled after the queue is finished
            signal_semaphore_count: signal_semaphore.len() as u32,
            p_signal_semaphores: signal_semaphore.as_ptr(),
            ..Default::default()
        };
        unsafe {
            self.logical_device
                .handle
                .reset_fences(&render_fences)
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
                    *render_fences.get(0).unwrap(),
                )
                .unwrap();
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
        self.frame_count += 1;
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
