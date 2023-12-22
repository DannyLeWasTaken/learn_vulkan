use crate::lv;
use ash::vk;
use std::sync::Arc;

#[derive(Clone)]
pub struct Surface {
    pub loader: Arc<ash::extensions::khr::Surface>,
    pub handle: vk::SurfaceKHR,
    lv: Arc<lv::lv>,
}

impl Surface {
    pub fn new(
        lv: Arc<lv::lv>,
        loader: Arc<ash::extensions::khr::Surface>,
        display_handle: raw_window_handle::RawDisplayHandle,
        window_handle: raw_window_handle::RawWindowHandle,
    ) -> Surface {
        let surface = unsafe {
            ash_window::create_surface(
                lv.entry.read().as_ref().unwrap(),
                lv.instance.read().as_ref().unwrap(),
                display_handle,
                window_handle,
                None,
            )
            .unwrap()
        };
        Surface {
            loader,
            lv,
            handle: surface,
        }
    }
}

impl Drop for Surface {
    fn drop(&mut self) {
        unsafe {
            self.loader.destroy_surface(self.handle, None);
        }
    }
}
