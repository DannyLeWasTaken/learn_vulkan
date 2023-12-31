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
