pub mod core;

#[macro_use]
extern crate bitflags;

pub struct Capabilities;

impl Capabilities {
    pub fn spirv_supported() -> bool {
        use crate::core::rendering::shader;

        shader::check_spirv_support()
    }
}
