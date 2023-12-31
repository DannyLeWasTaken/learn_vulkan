use crate::lv;
use ash::vk;
use std::ffi::c_void;
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
    pub pipeline_rendering: vk::PipelineRenderingCreateInfo,
    viewports: Vec<vk::Viewport>,
    scissors: Vec<vk::Rect2D>,
    dynamic_states_vector: Vec<vk::DynamicState>,

    shader_stages: Vec<vk::PipelineShaderStageCreateInfo>,
    color_formats: Vec<vk::Format>,
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
                dst_color_blend_factor: vk::BlendFactor::ZERO,
                color_blend_op: vk::BlendOp::ADD,
                src_alpha_blend_factor: vk::BlendFactor::ONE,
                dst_alpha_blend_factor: vk::BlendFactor::ZERO,
                alpha_blend_op: vk::BlendOp::ADD,
            };
        PipelineBuilder {
            dynamic_states: vk::PipelineDynamicStateCreateInfo {
                s_type: vk::StructureType::PIPELINE_DYNAMIC_STATE_CREATE_INFO,
                dynamic_state_count: 0,
                p_dynamic_states: ptr::null(),
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
                depth_clamp_enable: vk::FALSE,
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
            pipeline_rendering: vk::PipelineRenderingCreateInfo {
                s_type: vk::StructureType::PIPELINE_RENDERING_CREATE_INFO,
                ..Default::default()
            },
            viewports: Vec::new(),
            scissors: Vec::new(),
            dynamic_states_vector: Vec::new(),

            shader_stages: Vec::new(),
            color_formats: Vec::new(),
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

    pub fn dynamic_states(mut self, states: Vec<vk::DynamicState>) -> Self {
        self.dynamic_states_vector = states;
        self.dynamic_states.dynamic_state_count = self.dynamic_states_vector.len() as u32;
        self.dynamic_states.p_dynamic_states = self.dynamic_states_vector.as_ptr();
        self
    }

    pub fn color_attachments(mut self, count: u32, formats: Vec<vk::Format>) -> Self {
        self.color_formats = formats;
        self.pipeline_rendering.p_color_attachment_formats = self.color_formats.as_ptr();
        self.pipeline_rendering.color_attachment_count = count;
        self
    }

    pub fn attach_shaders_stages(
        mut self,
        mut stages: Vec<vk::PipelineShaderStageCreateInfo>,
    ) -> Self {
        self.shader_stages.append(&mut stages);
        self
    }

    pub fn build(self, device: Arc<lv::Device>) -> Pipeline {
        Pipeline::from_builder(self, device)
    }
}

pub struct Pipeline {
    handle: vk::Pipeline,
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
        let graphics_pipeline_ci = vk::GraphicsPipelineCreateInfo {
            s_type: vk::StructureType::GRAPHICS_PIPELINE_CREATE_INFO,
            p_next: &builder.pipeline_rendering as *const _ as *const c_void,
            stage_count: builder.shader_stages.len() as u32,
            p_stages: builder.shader_stages.as_ptr(),
            p_vertex_input_state: &builder.vertex_input,
            p_input_assembly_state: &builder.input_assembly,
            p_viewport_state: &builder.viewport_state,
            p_rasterization_state: &builder.rasterizer,
            p_multisample_state: &builder.multisampling,
            p_depth_stencil_state: ptr::null(),
            p_color_blend_state: &builder.color_blending,
            p_dynamic_state: &builder.dynamic_states,
            layout,
            render_pass: vk::RenderPass::null(),
            subpass: 0,
            base_pipeline_handle: vk::Pipeline::null(),
            base_pipeline_index: -1,
            ..Default::default()
        };
        let handle = unsafe {
            device
                .handle
                .create_graphics_pipelines(vk::PipelineCache::null(), &[graphics_pipeline_ci], None)
                .unwrap()
                .pop()
                .unwrap()
        };
        Pipeline {
            handle,
            layout,
            device,
        }
    }

    pub fn get_handle(&self) -> vk::Pipeline {
        self.handle
    }

    pub fn attach_shader(shader: lv::Shader) {}
}

impl Drop for Pipeline {
    fn drop(&mut self) {
        unsafe {
            self.device
                .handle
                .destroy_pipeline_layout(self.layout, None);
            self.device.handle.destroy_pipeline(self.handle, None);
        }
    }
}
