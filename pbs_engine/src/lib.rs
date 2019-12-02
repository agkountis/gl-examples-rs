pub mod core;
pub use crate::core::*;

#[macro_use]
extern crate bitflags;
extern crate crossbeam_channel;

pub mod capabilities {
    pub fn spirv_supported() -> bool {
        use crate::core::rendering::shader;

        shader::check_spirv_support()
    }
}
