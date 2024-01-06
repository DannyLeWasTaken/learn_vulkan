use crate::lv;
use crate::lv::traits::Resource;
use ash::vk;
use ash::vk::TaggedStructure;
use lv::descriptors::DescriptorTable;
use std::ffi::c_void;
use std::ptr;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub enum DescriptorInfo {
    Image(vk::DescriptorImageInfo),
    Buffer(vk::DescriptorBufferInfo),
}

#[derive(Clone, Debug)]
struct DescriptorWrite {
    info: DescriptorInfo,
    id: u64,
}

pub struct GPUResourceTable {
    handle: vk::DescriptorSet,
    pool: vk::DescriptorPool,
    layout: vk::DescriptorSetLayout,

    storage_image: DescriptorTable<lv::AllocatedImage>,

    device: Arc<lv::Device>,
}

impl GPUResourceTable {
    pub fn new(device: Arc<lv::Device>) -> Self {
        let types = [vk::DescriptorType::STORAGE_IMAGE];
        let descriptor_flags = [
            vk::DescriptorBindingFlags::UPDATE_AFTER_BIND,
            vk::DescriptorBindingFlags::PARTIALLY_BOUND,
            vk::DescriptorBindingFlags::UPDATE_UNUSED_WHILE_PENDING,
        ];

        let pool_sizes: Vec<vk::DescriptorPoolSize> = types
            .iter()
            .map(|ty| vk::DescriptorPoolSize {
                ty: *ty,
                descriptor_count: u16::MAX as u32,
            })
            .collect();
        let pool = unsafe {
            device
                .handle
                .create_descriptor_pool(
                    &vk::DescriptorPoolCreateInfo {
                        s_type: vk::DescriptorPoolCreateInfo::STRUCTURE_TYPE,
                        flags: vk::DescriptorPoolCreateFlags::UPDATE_AFTER_BIND
                            | vk::DescriptorPoolCreateFlags::FREE_DESCRIPTOR_SET,
                        max_sets: u16::MAX as u32,
                        pool_size_count: pool_sizes.len() as u32,
                        p_pool_sizes: pool_sizes.as_ptr(),
                        ..Default::default()
                    },
                    None,
                )
                .unwrap()
        };

        let binding_flags = vk::DescriptorSetLayoutBindingFlagsCreateInfo {
            s_type: vk::DescriptorSetLayoutBindingFlagsCreateInfo::STRUCTURE_TYPE,
            p_next: ptr::null(),
            binding_count: types.len() as u32,
            p_binding_flags: descriptor_flags.as_ptr(),
        };
        let binding_flags: Vec<vk::DescriptorSetLayoutBindingFlagsCreateInfo> = (0..4)
            .map(|_| vk::DescriptorSetLayoutBindingFlagsCreateInfo {
                s_type: vk::DescriptorSetLayoutBindingFlagsCreateInfo::STRUCTURE_TYPE,
                p_next: ptr::null(),
                binding_count: types.len() as u32,
                p_binding_flags: descriptor_flags.as_ptr(),
            })
            .collect();
        let descriptor_bindings: Vec<vk::DescriptorSetLayoutBinding> = types
            .iter()
            .enumerate()
            .map(|(index, ty)| vk::DescriptorSetLayoutBinding {
                binding: index as u32,
                descriptor_type: *ty,
                descriptor_count: u16::MAX as u32,
                stage_flags: vk::ShaderStageFlags::ALL,
                ..Default::default()
            })
            .collect();
        let layout_ci = vk::DescriptorSetLayoutCreateInfo {
            s_type: vk::DescriptorSetLayoutCreateInfo::STRUCTURE_TYPE,
            p_next: binding_flags.as_ptr() as *const _ as *const c_void,
            binding_count: descriptor_bindings.len() as u32,
            p_bindings: descriptor_bindings.as_ptr(),
            ..Default::default()
        };

        let layout = unsafe {
            device
                .handle
                .create_descriptor_set_layout(&layout_ci, None)
                .unwrap()
        };

        let handle = unsafe {
            device
                .handle
                .allocate_descriptor_sets(&vk::DescriptorSetAllocateInfo {
                    s_type: vk::DescriptorSetAllocateInfo::STRUCTURE_TYPE,
                    descriptor_pool: pool,
                    descriptor_set_count: 1,
                    p_set_layouts: &layout,
                    ..Default::default()
                })
                .unwrap()
                .pop()
                .unwrap()
        };

        Self {
            handle,
            pool,
            layout,

            storage_image: DescriptorTable::new(),

            device,
        }
    }

    pub fn get_pool(&self) -> &vk::DescriptorPool {
        &self.pool
    }

    pub fn get_layout(&self) -> &vk::DescriptorSetLayout {
        &self.layout
    }

    pub fn get_descriptor(&self) -> &vk::DescriptorSet {
        &self.handle
    }

    pub fn allocate_storage_image(&mut self, resource: lv::AllocatedImage) -> u32 {
        self.storage_image.allocate_resource(resource)
    }

    pub fn free_storage_image(&mut self, index: u32) {
        self.storage_image.free_resource(index)
    }

    pub fn get_storage_image(&self, index: usize) -> &Option<lv::AllocatedImage> {
        self.storage_image.get_resource(index)
    }

    pub fn update(&mut self) {
        let mut image_writes: Vec<vk::WriteDescriptorSet> =
            Vec::with_capacity(self.storage_image.get_resources().len());
        let mut image_infos: Vec<vk::DescriptorImageInfo> = Vec::new();
        let resources = self.storage_image.get_resources();
        for write_index in self.storage_image.get_writes() {
            let write = resources.get(*write_index as usize).unwrap();
            if let Some(write) = write {
                let info = match write.get_descriptor() {
                    DescriptorInfo::Image(info) => info,
                    _ => panic!("Storage image expected a storage image descriptor info, but got otherwise.")
                };
                image_writes.push(vk::WriteDescriptorSet {
                    s_type: vk::WriteDescriptorSet::STRUCTURE_TYPE,
                    dst_set: self.handle,
                    dst_binding: 0,
                    dst_array_element: *write_index,
                    descriptor_count: 1,
                    descriptor_type: vk::DescriptorType::STORAGE_IMAGE,
                    p_image_info: &info,
                    p_buffer_info: ptr::null(),
                    p_texel_buffer_view: ptr::null(),
                    ..Default::default()
                });
            }
        }

        // allocate writes
        unsafe {
            self.device
                .handle
                .update_descriptor_sets(&image_writes, &[]);
        }

        self.storage_image.clear_writes();
    }
}

impl Drop for GPUResourceTable {
    fn drop(&mut self) {
        unsafe {
            self.device.handle.destroy_descriptor_pool(self.pool, None);
            self.device
                .handle
                .destroy_descriptor_set_layout(self.layout, None);
        }
    }
}
