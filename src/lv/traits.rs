use crate::lv;

pub trait Resource {
	fn get_descriptor(&self) -> lv::descriptors::DescriptorInfo;
}