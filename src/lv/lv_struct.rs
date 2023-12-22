use crate::utility;
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct lv {
    pub entry: Arc<RwLock<ash::Entry>>,
    pub instance: Arc<RwLock<ash::Instance>>,
}

impl lv {
    pub fn check_validation_layer_support(&self, required_layers: Vec<&str>) -> bool {
        let layer_properties = self
            .entry
            .read()
            .unwrap()
            .enumerate_instance_layer_properties()
            .expect("Failed to enumerate instance layer properties");

        if layer_properties.is_empty() {
            eprintln!("No available layers.");
            return false;
        } else {
            for required_layer_name in required_layers {
                let mut is_layer_found = false;
                for layer_property in layer_properties.iter() {
                    let test_layer_name = utility::tools::vk_to_string(&layer_property.layer_name);
                    if (*required_layer_name) == test_layer_name {
                        is_layer_found = true;
                        break;
                    }
                }

                if !is_layer_found {
                    return false;
                }
            }
        }

        true
    }
}

impl Drop for lv {
    fn drop(&mut self) {
        unsafe {
            self.instance.read().unwrap().destroy_instance(None);
        }
    }
}
