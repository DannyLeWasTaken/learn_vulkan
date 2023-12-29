use crate::lv;
use ash::vk;
use std::sync::Arc;

fn read_shader_code(shader_path: &std::path::Path) -> Vec<u8> {
    use std::fs::File;
    use std::io::Read;

    let spv_file = File::open(shader_path)
        .unwrap_or_else(|_| panic!("Failed to find spv file at {:?}", shader_path));
    let bytes_code: Vec<u8> = spv_file.bytes().filter_map(|byte| byte.ok()).collect();

    bytes_code
}

pub struct Shader {
    pub handle: vk::ShaderModule,
    device: Arc<lv::Device>,
}

impl Shader {
    pub fn new(path: &std::path::Path, device: Arc<lv::Device>) -> Shader {
        let shader_code = read_shader_code(path);
        let shader_ci = vk::ShaderModuleCreateInfo {
            s_type: vk::StructureType::SHADER_MODULE_CREATE_INFO,
            code_size: shader_code.len(),
            p_code: shader_code.as_ptr() as *const _ as *const u32,
            ..Default::default()
        };
        let shader = unsafe {
            device
                .handle
                .create_shader_module(&shader_ci, None)
                .unwrap()
        };
        Shader {
            handle: shader,
            device,
        }
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            self.device.handle.destroy_shader_module(self.handle, None);
        }
    }
}
