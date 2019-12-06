pub mod core;
pub mod rendering;
pub use crate::core::*;
pub use crate::rendering::*;

#[macro_use]
extern crate bitflags;
extern crate crossbeam_channel;

pub mod capabilities {
    use crate::shader;

    pub fn spirv_supported() -> bool {
        shader::check_spirv_support()
    }
}
