use std::hint::spin_loop;
use crate::lv;
use ash::vk;

pub mod init;
pub mod tools;

pub fn transition_image(
    device: &ash::Device,
    command_buffer: vk::CommandBuffer,
    image: vk::Image,
    old_layout: vk::ImageLayout,
    new_layout: vk::ImageLayout,
    src_queue_family_index: u32,
    dst_queue_family_index: u32,
) {
    let image_barrier = vk::ImageMemoryBarrier2 {
        s_type: vk::StructureType::IMAGE_MEMORY_BARRIER_2,

        src_queue_family_index,
        dst_queue_family_index,

        src_stage_mask: vk::PipelineStageFlags2::ALL_COMMANDS,
        src_access_mask: vk::AccessFlags2::MEMORY_WRITE,

        dst_stage_mask: vk::PipelineStageFlags2::ALL_COMMANDS,
        dst_access_mask: vk::AccessFlags2::MEMORY_READ | vk::AccessFlags2::MEMORY_WRITE,

        old_layout,
        new_layout,
        image,

        subresource_range: init::image_subresource_range(vk::ImageAspectFlags::COLOR),

        ..Default::default()
    };
    let dep_info = vk::DependencyInfo {
        s_type: vk::StructureType::DEPENDENCY_INFO,
        image_memory_barrier_count: 1,
        p_image_memory_barriers: [image_barrier].as_ptr(),
        ..Default::default()
    };
    unsafe {
        device.cmd_pipeline_barrier2(command_buffer, &dep_info);
    }
}

pub fn copy_image_to_image(
    command_buffer: vk::CommandBuffer,
    device: &lv::Device,
    source: vk::Image,
    destination: vk::Image,
    src_size: vk::Extent2D,
    dst_size: vk::Extent2D,
) {
    let blit_region = vk::ImageBlit2 {
        s_type: vk::StructureType::IMAGE_BLIT_2,
        src_subresource: vk::ImageSubresourceLayers {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            mip_level: 0,
            base_array_layer: 0,
            layer_count: 1,
        },
        src_offsets: [
            vk::Offset3D {
                x: 0,
                y: 0,
                z: 0,
            },
            vk::Offset3D {
                x: src_size.width as i32,
                y: src_size.height as i32,
                z: 1,
            },
        ],
        dst_subresource: vk::ImageSubresourceLayers {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            mip_level: 0,
            base_array_layer: 0,
            layer_count: 1,
        },
        dst_offsets: [
            vk::Offset3D {
                x: 0,
                y: 0,
                z: 0,
            },
            vk::Offset3D {
                x: dst_size.width as i32,
                y: dst_size.height as i32,
                z: 1,
            },
        ],
        ..Default::default()
    };
    let blit_info = vk::BlitImageInfo2 {
        s_type: vk::StructureType::BLIT_IMAGE_INFO_2,
        dst_image: destination,
        dst_image_layout: vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        src_image: source,
        src_image_layout: vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
        filter: vk::Filter::LINEAR,
        region_count: 1,
        p_regions: &blit_region,
        ..Default::default()
    };
    unsafe {
        device.handle
            .cmd_blit_image2(command_buffer, &blit_info)
    };
}
