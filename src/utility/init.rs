use ash::vk;

pub fn image_subresource_range(aspect_mask: vk::ImageAspectFlags) -> vk::ImageSubresourceRange {
    let sub_image = vk::ImageSubresourceRange {
        aspect_mask,
        base_mip_level: 0,
        level_count: vk::REMAINING_MIP_LEVELS,
        base_array_layer: 0,
        layer_count: vk::REMAINING_ARRAY_LAYERS,
    };

    sub_image
}

pub fn attachment_info(
    image_view: vk::ImageView,
    clear: Option<vk::ClearValue>,
    layout: vk::ImageLayout,
) -> vk::RenderingAttachmentInfo {
    let color_attachment = vk::RenderingAttachmentInfo {
        s_type: vk::StructureType::RENDERING_ATTACHMENT_INFO,
        image_view,
        image_layout: layout,
        load_op: if clear.is_some() {
            vk::AttachmentLoadOp::CLEAR
        } else {
            vk::AttachmentLoadOp::LOAD
        },
        store_op: vk::AttachmentStoreOp::STORE,
        clear_value: clear.unwrap_or_default(),
        ..Default::default()
    };

    color_attachment
}

pub fn image_create_info(
    format: vk::Format,
    usage_flags: vk::ImageUsageFlags,
    extent: vk::Extent3D,
) -> vk::ImageCreateInfo {
    let image_ci = vk::ImageCreateInfo {
        s_type: vk::StructureType::IMAGE_CREATE_INFO,
        image_type: vk::ImageType::TYPE_2D,

        format,
        extent,

        mip_levels: 1,
        array_layers: 1,

        samples: vk::SampleCountFlags::TYPE_1,

        // GPU manage how the image is stored
        tiling: vk::ImageTiling::OPTIMAL,
        usage: usage_flags,

        ..Default::default()
    };
    image_ci
}

pub fn image_view_create_info(
    format: vk::Format,
    image: vk::Image,
    aspect_flags: vk::ImageAspectFlags,
) -> vk::ImageViewCreateInfo {
    vk::ImageViewCreateInfo {
        s_type: vk::StructureType::IMAGE_VIEW_CREATE_INFO,

        view_type: vk::ImageViewType::TYPE_2D,
        image,
        format,
        subresource_range: image_subresource_range(aspect_flags),

        ..Default::default()
    }
}

pub fn command_buffer_begin_info(flags: vk::CommandBufferUsageFlags) -> vk::CommandBufferBeginInfo {
    vk::CommandBufferBeginInfo {
        s_type: vk::StructureType::COMMAND_BUFFER_BEGIN_INFO,
        flags,
        ..Default::default()
    }
}
