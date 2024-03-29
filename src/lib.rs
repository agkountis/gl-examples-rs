#[macro_use]
extern crate memoffset;

#[macro_export]
macro_rules! impl_as_any {
    ($t: ty) => {
        impl AsAny for $t {
            fn as_any(&self) -> &dyn Any {
                self
            }
        }

        impl AsAnyMut for $t {
            fn as_any_mut(&mut self) -> &mut dyn Any {
                self
            }
        }
    };
}

pub mod core;
pub mod imgui;
pub mod rendering;

pub use crate::core::*;
pub use crate::rendering::*;

#[macro_use]
extern crate bitflags;

#[macro_use]
extern crate lazy_static;

pub mod capabilities {
    use crate::shader;

    pub fn spirv_supported() -> bool {
        shader::check_spirv_support()
    }
}
