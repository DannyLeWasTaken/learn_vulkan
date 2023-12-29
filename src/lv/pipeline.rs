use crate::lv;
use ash::vk;
use std::ptr;
use std::sync::Arc;

pub struct PipelineBuilder {
    pub dynamic_states: vk::PipelineDynamicStateCreateInfo,
    vertex_input: vk::PipelineVertexInputStateCreateInfo,
    pub input_assembly: vk::PipelineInputAssemblyStateCreateInfo,
    pub viewport_state: vk::PipelineViewportStateCreateInfo,
    pub rasterizer: vk::PipelineRasterizationStateCreateInfo,
    pub multisampling: vk::PipelineMultisampleStateCreateInfo,
    pub color_blending: vk::PipelineColorBlendStateCreateInfo,
    pub pipeline_layout: vk::PipelineLayoutCreateInfo,
    viewports: Vec<vk::Viewport>,
    scissors: Vec<vk::Rect2D>,
}

impl PipelineBuilder {
    pub fn new() -> Self {
        let color_blending: vk::PipelineColorBlendAttachmentState =
            vk::PipelineColorBlendAttachmentState {
                color_write_mask: vk::ColorComponentFlags::R
                    | vk::ColorComponentFlags::G
                    | vk::ColorComponentFlags::B
                    | vk::ColorComponentFlags::A,
                blend_enable: vk::FALSE,
                src_color_blend_factor: vk::BlendFactor::ONE,
                dst_color_blend_factor: vk::BlendFactor::ONE_MINUS_SRC_ALPHA,
                color_blend_op: vk::BlendOp::ADD,
                src_alpha_blend_factor: vk::BlendFactor::ONE,
                dst_alpha_blend_factor: vk::BlendFactor::ZERO,
                alpha_blend_op: vk::BlendOp::ADD,
                ..Default::default()
            };
        PipelineBuilder {
            dynamic_states: vk::PipelineDynamicStateCreateInfo {
                ..Default::default()
            },
            vertex_input: vk::PipelineVertexInputStateCreateInfo {
                s_type: vk::StructureType::PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO,
                vertex_binding_description_count: 0,
                p_vertex_binding_descriptions: ptr::null(),
                vertex_attribute_description_count: 0,
                p_vertex_attribute_descriptions: ptr::null(),
                ..Default::default()
            },
            input_assembly: vk::PipelineInputAssemblyStateCreateInfo {
                s_type: vk::StructureType::PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO,
                topology: vk::PrimitiveTopology::TRIANGLE_LIST,
                primitive_restart_enable: vk::FALSE,
                ..Default::default()
            },
            viewport_state: vk::PipelineViewportStateCreateInfo {
                s_type: vk::StructureType::PIPELINE_VIEWPORT_STATE_CREATE_INFO,
                p_scissors: ptr::null(),
                scissor_count: 0,
                p_viewports: ptr::null(),
                viewport_count: 0,
                ..Default::default()
            },
            rasterizer: vk::PipelineRasterizationStateCreateInfo {
                s_type: vk::StructureType::PIPELINE_RASTERIZATION_STATE_CREATE_INFO,
                rasterizer_discard_enable: vk::FALSE,
                polygon_mode: vk::PolygonMode::FILL,
                line_width: 1.0f32,
                cull_mode: vk::CullModeFlags::BACK,
                front_face: vk::FrontFace::CLOCKWISE,
                depth_bias_enable: vk::FALSE,
                depth_bias_constant_factor: 0.0f32,
                depth_bias_clamp: 0.0f32,
                depth_bias_slope_factor: 0.0f32,
                ..Default::default()
            },
            multisampling: vk::PipelineMultisampleStateCreateInfo {
                s_type: vk::StructureType::PIPELINE_MULTISAMPLE_STATE_CREATE_INFO,
                sample_shading_enable: vk::TRUE,
                rasterization_samples: vk::SampleCountFlags::TYPE_1,
                min_sample_shading: 1.0f32,
                p_sample_mask: ptr::null(),
                alpha_to_coverage_enable: vk::FALSE,
                alpha_to_one_enable: vk::FALSE,
                ..Default::default()
            },
            color_blending: vk::PipelineColorBlendStateCreateInfo {
                s_type: vk::StructureType::PIPELINE_COLOR_BLEND_STATE_CREATE_INFO,
                logic_op_enable: vk::FALSE,
                logic_op: vk::LogicOp::COPY,
                attachment_count: 1,
                p_attachments: &color_blending,
                blend_constants: [0.0f32, 0.0f32, 0.0f32, 0.0f32],
                ..Default::default()
            },
            pipeline_layout: vk::PipelineLayoutCreateInfo {
                s_type: vk::StructureType::PIPELINE_LAYOUT_CREATE_INFO,
                set_layout_count: 0,
                p_set_layouts: ptr::null(),
                push_constant_range_count: 0,
                p_push_constant_ranges: ptr::null(),
                ..Default::default()
            },
            viewports: Vec::new(),
            scissors: Vec::new(),
        }
    }

    /// Sets viewports and automatically deals with pointers and size
    pub fn set_viewports(
        mut self,
        viewports: Option<Vec<vk::Viewport>>,
        scissors: Option<Vec<vk::Rect2D>>,
    ) -> Self {
        if let Some(viewports) = viewports {
            self.viewports = viewports;
            self.viewport_state.viewport_count = self.viewports.len() as u32;
            self.viewport_state.p_viewports = self.viewports.as_ptr();
        }
        if let Some(scissors) = scissors {
            self.scissors = scissors;
            self.viewport_state.scissor_count = self.scissors.len() as u32;
            self.viewport_state.p_scissors = self.scissors.as_ptr();
        }
        self
    }

    /// Sets only viewport counts, but will clear out pointers
    pub fn set_viewport_counts(mut self, viewport_count: u32, scissors_count: u32) -> Self {
        self.viewport_state.viewport_count = viewport_count;
        self.viewport_state.scissor_count = scissors_count;
        self.viewports.clear();
        self.scissors.clear();
        self.viewport_state.p_viewports = ptr::null();
        self.viewport_state.p_scissors = ptr::null();
        self
    }

    pub fn build(self, device: Arc<lv::Device>) -> Pipeline {
        Pipeline::from_builder(self, device)
    }
}

pub struct Pipeline {
    layout: vk::PipelineLayout,

    // Reference-counting
    device: Arc<lv::Device>,
}

/// Configuration of the lv Pipeline object
pub struct PipelineConfiguration {}

impl Pipeline {
    fn from_builder(builder: PipelineBuilder, device: Arc<lv::Device>) -> Self {
        let layout = unsafe {
            device
                .handle
                .create_pipeline_layout(&builder.pipeline_layout, None)
                .unwrap()
        };
        Pipeline { layout, device }
    }

    pub fn attach_shader(shader: lv::Shader) {}
}

impl Drop for Pipeline {
    fn drop(&mut self) {
        unsafe {
            self.device
                .handle
                .destroy_pipeline_layout(self.layout, None);
        }
    }
}
