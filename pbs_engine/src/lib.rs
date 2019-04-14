pub mod core;

#[macro_use]
extern crate bitflags;

pub mod capabilities {
    pub fn spirv_supported() -> bool {
        use crate::core::rendering::shader;

        shader::check_spirv_support()
    }
}
