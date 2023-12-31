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
