use crate::lv;
use ash::vk;
use std::sync::Arc;

#[derive(Clone)]
pub struct Surface {
    pub loader: ash::extensions::khr::Surface,
    pub handle: vk::SurfaceKHR,
}

impl Surface {
    pub fn new(
        lv: &lv::Instance,
        loader: ash::extensions::khr::Surface,
        display_handle: raw_window_handle::RawDisplayHandle,
        window_handle: raw_window_handle::RawWindowHandle,
    ) -> Arc<Surface> {
        let surface = unsafe {
            ash_window::create_surface(&lv.entry, &lv.instance, display_handle, window_handle, None)
                .unwrap()
        };
        Arc::new(Surface {
            loader,
            handle: surface,
        })
    }
}

impl Drop for Surface {
    fn drop(&mut self) {
        unsafe {
            self.loader.destroy_surface(self.handle, None);
        }
    }
}
