use crate::{lv, utility};
use ash::vk;
use ash::vk::ImageAspectFlags;
use std::cell::RefCell;
use std::sync::{Arc, Mutex};

pub struct AllocatedImage {
    handle: vk::Image,
    view: vk::ImageView,
    allocation: gpu_allocator::vulkan::Allocation,
    extent: vk::Extent3D,
    format: vk::Format,

    device: Arc<lv::Device>,
    allocator: Arc<Mutex<gpu_allocator::vulkan::Allocator>>,
}

impl AllocatedImage {
    pub fn new(
        image_ci: vk::ImageCreateInfo,
        image_aspect_flags: ImageAspectFlags,
        device: Arc<lv::Device>,
        allocator: Arc<Mutex<gpu_allocator::vulkan::Allocator>>,
    ) -> Self {
        let handle = unsafe { device.handle.create_image(&image_ci, None).unwrap() };
        let requirements = unsafe { device.handle.get_image_memory_requirements(handle) };
        // Allocate image into gpu memory
        let mut allocation = None;
        {
            let mut allocator_lock = allocator.lock().unwrap();
            allocation = Some(
                allocator_lock
                    .allocate(&gpu_allocator::vulkan::AllocationCreateDesc {
                        name: "Image",
                        requirements,
                        location: gpu_allocator::MemoryLocation::GpuOnly,
                        linear: true,
                        allocation_scheme:
                            gpu_allocator::vulkan::AllocationScheme::GpuAllocatorManaged,
                    })
                    .unwrap(),
            );
            let allocation = allocation.as_ref().unwrap();
            unsafe {
                device
                    .handle
                    .bind_image_memory(handle, allocation.memory(), allocation.offset())
                    .unwrap()
            };
        }

        let view_ci =
            utility::init::image_view_create_info(image_ci.format, handle, image_aspect_flags);
        let view = unsafe { device.handle.create_image_view(&view_ci, None).unwrap() };
        let extent = image_ci.extent;
        let format = image_ci.format;

        AllocatedImage {
            handle,
            view,
            allocation: allocation.unwrap(),
            extent,
            format,

            device,
            allocator,
        }
    }

    pub fn get_handle(&self) -> vk::Image {
        self.handle
    }
}

impl Drop for AllocatedImage {
    fn drop(&mut self) {
        unsafe {
            self.device.handle.destroy_image_view(self.view, None);
            self.device.handle.destroy_image(self.handle, None);
            let mut allocator = self.allocator.lock().unwrap();
            allocator
                .free(std::mem::take(&mut self.allocation))
                .unwrap();
        };
    }
}
