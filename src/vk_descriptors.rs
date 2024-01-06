use crate::lv;
use ash::vk;
use std::sync::Arc;

pub struct DescriptorLayoutBuilder {
    bindings: Vec<vk::DescriptorSetLayoutBinding>,
}

#[derive(Default, Copy, Clone)]
pub struct PoolSizeRatio {
    pub descriptor_type: vk::DescriptorType,
    pub ratio: f32,
}

pub struct DescriptorAllocator {
    pool: vk::DescriptorPool,

    device: Arc<lv::Device>,
}

impl DescriptorLayoutBuilder {
    pub fn new() -> DescriptorLayoutBuilder {
        DescriptorLayoutBuilder {
            bindings: Vec::new(),
        }
    }

    pub fn add_binding(&mut self, binding: u32, descriptor_type: vk::DescriptorType) {
        let new_bind = vk::DescriptorSetLayoutBinding {
            binding,
            descriptor_count: 1,
            descriptor_type,
            stage_flags: vk::ShaderStageFlags::empty(),
            ..Default::default()
        };
        self.bindings.push(new_bind);
    }

    pub fn clear(&mut self) {}

    pub fn build(
        &mut self,
        device: &ash::Device,
        shader_stages: vk::ShaderStageFlags,
    ) -> vk::DescriptorSetLayout {
        for bindings in self.bindings.iter_mut() {
            bindings.stage_flags |= shader_stages;
        }

        let descriptor_set_ci = vk::DescriptorSetLayoutCreateInfo {
            s_type: vk::StructureType::DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
            p_bindings: self.bindings.as_ptr(),
            binding_count: self.bindings.len() as u32,
            flags: vk::DescriptorSetLayoutCreateFlags::empty(),
            ..Default::default()
        };
        let set = unsafe {
            device
                .create_descriptor_set_layout(&descriptor_set_ci, None)
                .unwrap()
        };
        set
    }
}

impl DescriptorAllocator {
    pub fn new(device: Arc<lv::Device>, max_sets: u32, pool_ratio: &[PoolSizeRatio]) -> Self {
        let mut pool_sizes: Vec<vk::DescriptorPoolSize> = Vec::with_capacity(pool_ratio.len());
        for ratio in pool_ratio.iter() {
            pool_sizes.push(vk::DescriptorPoolSize {
                ty: ratio.descriptor_type,
                descriptor_count: (ratio.ratio * max_sets as f32) as u32,
            });
        }

        let pool_ci = vk::DescriptorPoolCreateInfo {
            s_type: vk::StructureType::DESCRIPTOR_POOL_CREATE_INFO,
            flags: vk::DescriptorPoolCreateFlags::empty(),
            max_sets,
            pool_size_count: pool_sizes.len() as u32,
            p_pool_sizes: pool_sizes.as_ptr(),
            ..Default::default()
        };
        let handle = unsafe {
            device
                .handle
                .create_descriptor_pool(&pool_ci, None)
                .unwrap()
        };
        Self {
            pool: handle,

            device,
        }
    }

    pub fn clear_descriptors(&self) {
        unsafe {
            self.device
                .handle
                .reset_descriptor_pool(self.pool, vk::DescriptorPoolResetFlags::empty())
                .unwrap()
        }
    }

    pub fn allocate(&self, layout: vk::DescriptorSetLayout) -> Vec<vk::DescriptorSet> {
        let allocation_info = vk::DescriptorSetAllocateInfo {
            s_type: vk::StructureType::DESCRIPTOR_SET_ALLOCATE_INFO,
            descriptor_pool: self.pool,
            descriptor_set_count: 1,
            p_set_layouts: &layout,
            ..Default::default()
        };
        unsafe {
            self.device
                .handle
                .allocate_descriptor_sets(&allocation_info)
                .unwrap()
        }
    }
}

impl Drop for DescriptorAllocator {
    fn drop(&mut self) {
        unsafe { self.device.handle.destroy_descriptor_pool(self.pool, None) }
    }
}
