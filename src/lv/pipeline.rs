use crate::lv;
use ash::vk;
use ash::vk::TaggedStructure;
use std::cmp::max_by;
use std::ffi::{c_char, c_void};
use std::future::poll_fn;
use std::ptr;
use std::sync::Arc;

pub struct PipelineBuilder {
    pub input_assembly: vk::PipelineInputAssemblyStateCreateInfo,
    pub rasterizer: vk::PipelineRasterizationStateCreateInfo,
    pub color_blend_attachment: vk::PipelineColorBlendAttachmentState,
    pub multisampling: vk::PipelineMultisampleStateCreateInfo,
    pub pipeline_layout: vk::PipelineLayout,
    pub depth_stencil: vk::PipelineDepthStencilStateCreateInfo,
    pub render_info: vk::PipelineRenderingCreateInfo,
    viewports: Vec<vk::Viewport>,
    scissors: Vec<vk::Rect2D>,
    dynamic_states_vector: Vec<vk::DynamicState>,

    shader_stages: Vec<vk::PipelineShaderStageCreateInfo>,
    depth_formats: Vec<vk::Format>,
    color_formats: Vec<vk::Format>,
}

impl PipelineBuilder {
    pub fn new() -> Self {
        PipelineBuilder {
            input_assembly: vk::PipelineInputAssemblyStateCreateInfo {
                s_type: vk::StructureType::PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO,
                ..Default::default()
            },
            rasterizer: vk::PipelineRasterizationStateCreateInfo {
                s_type: vk::StructureType::PIPELINE_RASTERIZATION_STATE_CREATE_INFO,
                ..Default::default()
            },
            color_blend_attachment: vk::PipelineColorBlendAttachmentState {
                ..Default::default()
            },
            multisampling: vk::PipelineMultisampleStateCreateInfo {
                s_type: vk::StructureType::PIPELINE_MULTISAMPLE_STATE_CREATE_INFO,
                ..Default::default()
            },
            pipeline_layout: Default::default(),
            depth_stencil: vk::PipelineDepthStencilStateCreateInfo {
                s_type: vk::PipelineDepthStencilStateCreateInfo::STRUCTURE_TYPE,
                ..Default::default()
            },
            render_info: vk::PipelineRenderingCreateInfo {
                s_type: vk::PipelineRenderingCreateInfo::STRUCTURE_TYPE,
                ..Default::default()
            },
            viewports: Vec::new(),
            scissors: Vec::new(),
            dynamic_states_vector: Vec::new(),

            shader_stages: Vec::new(),
            color_formats: Vec::new(),
            depth_formats: Vec::new(),
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
        }
        if let Some(scissors) = scissors {
            self.scissors = scissors;
        }
        self
    }

    pub fn dynamic_states(mut self, states: Vec<vk::DynamicState>) -> Self {
        self.dynamic_states_vector = states;
        self
    }

    pub fn color_attachments(mut self, count: u32, formats: Vec<vk::Format>) -> Self {
        self.color_formats = formats;
        self
    }

    pub fn attach_shaders_stages(
        mut self,
        mut stages: Vec<vk::PipelineShaderStageCreateInfo>,
    ) -> Self {
        self.shader_stages.append(&mut stages);
        self
    }

    pub fn set_input_topology(mut self, topology: vk::PrimitiveTopology) -> Self {
        self.input_assembly.topology = topology;
        self.input_assembly.primitive_restart_enable = vk::FALSE;
        self
    }

    pub fn set_polygon_mode(mut self, mode: vk::PolygonMode) -> Self {
        self.rasterizer.polygon_mode = mode;
        self.rasterizer.line_width = 1.0f32;
        self
    }

    pub fn set_cull_mode(
        mut self,
        cull_mode: vk::CullModeFlags,
        front_face: vk::FrontFace,
    ) -> Self {
        self.rasterizer.cull_mode = cull_mode;
        self.rasterizer.front_face = front_face;
        self
    }

    pub fn set_multisampling_none(mut self) -> Self {
        self.multisampling.sample_shading_enable = vk::FALSE;
        self.multisampling.rasterization_samples = vk::SampleCountFlags::TYPE_1;
        self.multisampling.min_sample_shading = 1.0f32;
        self.multisampling.p_sample_mask = ptr::null();

        self.multisampling.alpha_to_one_enable = vk::FALSE;
        self.multisampling.alpha_to_one_enable = vk::FALSE;
        self
    }

    pub fn disable_blending(mut self) -> Self {
        self.color_blend_attachment.color_write_mask = vk::ColorComponentFlags::R
            | vk::ColorComponentFlags::G
            | vk::ColorComponentFlags::B
            | vk::ColorComponentFlags::A;
        self.color_blend_attachment.blend_enable = vk::FALSE;
        self
    }

    pub fn set_depth_format(mut self, format: vk::Format) -> Self {
        self.depth_formats.push(format);
        self
    }

    pub fn disable_depthtest(mut self) -> Self {
        self.depth_stencil.depth_test_enable = vk::FALSE;
        self.depth_stencil.depth_write_enable = vk::FALSE;
        self.depth_stencil.depth_compare_op = vk::CompareOp::NEVER;
        self.depth_stencil.depth_bounds_test_enable = vk::FALSE;
        self.depth_stencil.front = vk::StencilOpState::default();
        self.depth_stencil.back = vk::StencilOpState::default();
        self.depth_stencil.min_depth_bounds = 0.0f32;
        self.depth_stencil.max_depth_bounds = 1.0f32;
        self
    }
}

pub struct Pipeline {
    handle: vk::Pipeline,
    layout: vk::PipelineLayout,
    // Reference-counting
    device: Arc<lv::Device>,
}

impl Pipeline {
    pub fn from_builder(mut builder: PipelineBuilder, device: Arc<lv::Device>) -> Self {
        let layout_ci = vk::PipelineLayoutCreateInfo {
            s_type: vk::PipelineLayoutCreateInfo::STRUCTURE_TYPE,
            flags: vk::PipelineLayoutCreateFlags::empty(),
            set_layout_count: 0,
            p_set_layouts: ptr::null(),
            push_constant_range_count: 0,
            p_push_constant_ranges: ptr::null(),
            ..Default::default()
        };
        builder.pipeline_layout = unsafe {
            device
                .handle
                .create_pipeline_layout(&layout_ci, None)
                .unwrap()
        };
        let viewport_ci = vk::PipelineViewportStateCreateInfo {
            s_type: vk::PipelineViewportStateCreateInfo::STRUCTURE_TYPE,
            flags: vk::PipelineViewportStateCreateFlags::empty(),
            viewport_count: 1,
            p_viewports: ptr::null(),
            scissor_count: 1,
            p_scissors: ptr::null(),
            ..Default::default()
        };
        let color_blending = vk::PipelineColorBlendStateCreateInfo {
            s_type: vk::PipelineColorBlendStateCreateInfo::STRUCTURE_TYPE,
            logic_op: vk::LogicOp::COPY,
            attachment_count: 1,
            p_attachments: &builder.color_blend_attachment,
            ..Default::default()
        };
        let vertex_info = vk::PipelineVertexInputStateCreateInfo {
            s_type: vk::PipelineVertexInputStateCreateInfo::STRUCTURE_TYPE,
            ..Default::default()
        };
        let dynamic_state = vk::PipelineDynamicStateCreateInfo {
            s_type: vk::PipelineDynamicStateCreateInfo::STRUCTURE_TYPE,
            flags: vk::PipelineDynamicStateCreateFlags::empty(),
            dynamic_state_count: builder.dynamic_states_vector.len() as u32,
            p_dynamic_states: builder.dynamic_states_vector.as_ptr(),
            ..Default::default()
        };
        let pipeline_ci = vk::GraphicsPipelineCreateInfo {
            s_type: vk::GraphicsPipelineCreateInfo::STRUCTURE_TYPE,
            flags: vk::PipelineCreateFlags::empty(),
            stage_count: builder.shader_stages.len() as u32,
            p_stages: builder.shader_stages.as_ptr(),
            p_vertex_input_state: &vertex_info,
            p_input_assembly_state: &builder.input_assembly,
            p_tessellation_state: ptr::null(),
            p_viewport_state: &viewport_ci,
            p_rasterization_state: &builder.rasterizer,
            p_multisample_state: &builder.multisampling,
            p_depth_stencil_state: &builder.depth_stencil,
            p_color_blend_state: &color_blending,
            p_dynamic_state: &dynamic_state,
            layout: builder.pipeline_layout,
            render_pass: vk::RenderPass::null(),
            subpass: 0,
            base_pipeline_handle: vk::Pipeline::null(),
            base_pipeline_index: -1,
            ..Default::default()
        };
        let handle = unsafe {
            device
                .handle
                .create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_ci], None)
                .unwrap()
        }
        .pop()
        .unwrap();

        Pipeline {
            handle,
            layout: builder.pipeline_layout,
            device,
        }
    }

    pub fn get_handle(&self) -> vk::Pipeline {
        self.handle
    }
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

pub struct ComputePipeline {
    handle: vk::Pipeline,
    layout: vk::PipelineLayout,

    // ref-counts
    device: Arc<lv::Device>,
}

pub struct ComputePipelineBuilder {
    handle: vk::ComputePipelineCreateInfo,
    pipeline_layout: vk::PipelineLayoutCreateInfo,
    layouts: Vec<vk::DescriptorSetLayout>,
    shader_stage: vk::PipelineShaderStageCreateInfo,
    push_constant_range: Vec<vk::PushConstantRange>,
}

impl ComputePipelineBuilder {
    pub fn new() -> Self {
        ComputePipelineBuilder {
            handle: vk::ComputePipelineCreateInfo {
                s_type: vk::ComputePipelineCreateInfo::STRUCTURE_TYPE,
                ..Default::default()
            },
            pipeline_layout: vk::PipelineLayoutCreateInfo {
                s_type: vk::PipelineLayoutCreateInfo::STRUCTURE_TYPE,
                ..Default::default()
            },
            layouts: Vec::new(),
            shader_stage: vk::PipelineShaderStageCreateInfo::default(),
            push_constant_range: Vec::new(),
        }
    }

    pub fn set_layouts(mut self, layouts: Vec<vk::DescriptorSetLayout>) -> ComputePipelineBuilder {
        self.layouts = layouts;
        self.pipeline_layout.set_layout_count = self.layouts.len() as u32;
        self.pipeline_layout.p_set_layouts = self.layouts.as_ptr();
        self
    }

    pub fn attach_stages(mut self, stages: vk::PipelineShaderStageCreateInfo) -> Self {
        self.shader_stage = stages;
        self.handle.stage = self.shader_stage;
        self
    }

    pub fn attach_push_constant(mut self, push_constant_range: vk::PushConstantRange) -> Self {
        self.push_constant_range.push(push_constant_range);
        self.pipeline_layout.p_push_constant_ranges = self.push_constant_range.as_ptr();
        self.pipeline_layout.push_constant_range_count = self.push_constant_range.len() as u32;
        self
    }
}

impl ComputePipeline {
    pub fn from_builder(mut builder: ComputePipelineBuilder, device: Arc<lv::Device>) -> Self {
        println!(
            "Push constant range count: {:?}",
            builder.pipeline_layout.push_constant_range_count
        );
        let layout = unsafe {
            device
                .handle
                .create_pipeline_layout(&builder.pipeline_layout, None)
                .unwrap()
        };
        builder.handle.layout = layout;
        let pipeline = unsafe {
            device
                .handle
                .create_compute_pipelines(vk::PipelineCache::null(), &[builder.handle], None)
                .unwrap()
        }
        .pop()
        .unwrap();
        Self {
            handle: pipeline,
            layout,
            device,
        }
    }

    pub fn get_handle(&self) -> vk::Pipeline {
        self.handle
    }

    pub fn get_layout(&self) -> vk::PipelineLayout {
        self.layout
    }
}

impl Drop for ComputePipeline {
    fn drop(&mut self) {
        unsafe {
            self.device
                .handle
                .destroy_pipeline_layout(self.layout, None);
            self.device.handle.destroy_pipeline(self.handle, None)
        }
    }
}
