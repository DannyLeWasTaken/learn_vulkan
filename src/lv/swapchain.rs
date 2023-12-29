use crate::lv;
use ash::vk;
use std::ptr;
use std::sync::Arc;
use winit::window;

#[derive(Clone)]
pub struct SwapchainSupportDetails {
    pub capabilities: vk::SurfaceCapabilitiesKHR,
    pub formats: Vec<vk::SurfaceFormatKHR>,
    pub present_modes: Vec<vk::PresentModeKHR>,
}

impl SwapchainSupportDetails {
    pub fn choose_format(&self, preferred_formats: &[vk::Format]) -> vk::SurfaceFormatKHR {
        for preferred_format in preferred_formats.iter() {
            for available_format in self.formats.iter() {
                if *preferred_format == available_format.format {
                    return available_format.clone();
                }
            }
        }

        self.formats.get(0).unwrap().clone()
    }

    pub fn choose_presentation_mode(
        &self,
        preferred_present_modes: &[vk::PresentModeKHR],
    ) -> vk::PresentModeKHR {
        for preferred_mode in preferred_present_modes.iter() {
            for available_mode in self.present_modes.iter() {
                if preferred_mode == available_mode {
                    return *preferred_mode;
                }
            }
        }

        vk::PresentModeKHR::FIFO
    }

    pub fn choose_extent(&self, window: &window::Window) -> vk::Extent2D {
        if self.capabilities.current_extent.width != u32::MAX {
            return self.capabilities.current_extent;
        }
        let extent: vk::Extent2D = vk::Extent2D {
            width: window.inner_size().width.clamp(
                self.capabilities.min_image_extent.width,
                self.capabilities.max_image_extent.width,
            ),
            height: window.inner_size().height.clamp(
                self.capabilities.min_image_extent.height,
                self.capabilities.max_image_extent.height,
            ),
        };

        extent
    }
}

pub struct Swapchain {
    pub handle: vk::SwapchainKHR,
    pub details: SwapchainSupportDetails,
    pub images: Vec<vk::Image>,
    pub image_views: Vec<vk::ImageView>,
    pub extent: vk::Extent2D,
    pub surface_format: vk::SurfaceFormatKHR,
    loader: ash::extensions::khr::Swapchain,

    // Reference-counting
    device: Arc<lv::Device>,
}

pub struct SwapchainPreferred<'a> {
    pub preferred_format: &'a [vk::Format],
    pub preferred_present_modes: &'a [vk::PresentModeKHR],
    pub swapchain_support_details: SwapchainSupportDetails,
}

impl Swapchain {
    pub fn new(
        swapchain_loader: ash::extensions::khr::Swapchain,
        physical_device: &lv::PhysicalDevice,
        device: Arc<lv::Device>,
        surface: vk::SurfaceKHR,
        preferred: SwapchainPreferred,
        window: &window::Window,
    ) -> Swapchain {
        let swapchain_support_details = preferred.swapchain_support_details;
        let surface_format = swapchain_support_details.choose_format(preferred.preferred_format);
        let present_mode =
            swapchain_support_details.choose_presentation_mode(preferred.preferred_present_modes);
        let extent = swapchain_support_details.choose_extent(window);
        let image_count: u32 = (swapchain_support_details.capabilities.min_image_count + 3)
            .min(swapchain_support_details.capabilities.max_image_count);

        let family_queues = physical_device.queue_families;
        let swapchain_ci = vk::SwapchainCreateInfoKHR {
            s_type: vk::StructureType::SWAPCHAIN_CREATE_INFO_KHR,
            surface,
            min_image_count: image_count,
            image_format: surface_format.format,
            image_color_space: surface_format.color_space,
            image_extent: extent,
            image_array_layers: 1,
            image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT,
            image_sharing_mode: if family_queues.graphics_family.unwrap()
                != family_queues.present_family.unwrap()
            {
                vk::SharingMode::CONCURRENT
            } else {
                vk::SharingMode::EXCLUSIVE
            },
            queue_family_index_count: if family_queues.graphics_family.unwrap()
                != family_queues.present_family.unwrap()
            {
                2
            } else {
                0
            },
            p_queue_family_indices: if family_queues.graphics_family.unwrap()
                != family_queues.present_family.unwrap()
            {
                vec![
                    family_queues.graphics_family.unwrap(),
                    family_queues.present_family.unwrap(),
                ]
                .as_ptr()
            } else {
                ptr::null()
            },
            pre_transform: swapchain_support_details.capabilities.current_transform,
            composite_alpha: vk::CompositeAlphaFlagsKHR::OPAQUE,
            present_mode,
            clipped: vk::TRUE,
            old_swapchain: vk::SwapchainKHR::null(),
            ..vk::SwapchainCreateInfoKHR::default()
        };
        let swapchain = unsafe {
            swapchain_loader
                .create_swapchain(&swapchain_ci, None)
                .unwrap()
        };

        // Retrieve swapchain images and views
        let images = unsafe { swapchain_loader.get_swapchain_images(swapchain).unwrap() };
        let mut image_views = Vec::<vk::ImageView>::with_capacity(images.len());
        for (index, mut _image_view) in image_views.iter_mut().enumerate() {
            let image_view_ci = vk::ImageViewCreateInfo {
                s_type: vk::StructureType::IMAGE_VIEW_CREATE_INFO,
                image: images[index],
                view_type: vk::ImageViewType::TYPE_2D,
                format: surface_format.format,
                components: vk::ComponentMapping {
                    r: vk::ComponentSwizzle::IDENTITY,
                    g: vk::ComponentSwizzle::IDENTITY,
                    b: vk::ComponentSwizzle::IDENTITY,
                    a: vk::ComponentSwizzle::IDENTITY,
                },
                subresource_range: vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                },
                ..vk::ImageViewCreateInfo::default()
            };

            _image_view = &mut unsafe {
                device
                    .handle
                    .create_image_view(&image_view_ci, None)
                    .unwrap()
            };
        }

        Swapchain {
            handle: swapchain,
            details: swapchain_support_details,
            loader: swapchain_loader,
            images,
            image_views,
            surface_format,
            extent,
            device,
        }
    }
}

impl Swapchain {}

impl Drop for Swapchain {
    fn drop(&mut self) {
        unsafe {
            self.image_views.clear();
            self.loader.destroy_swapchain(self.handle, None);
        };
    }
}
