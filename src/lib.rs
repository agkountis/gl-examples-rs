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

pub mod capabilities {
    use crate::shader;

    pub fn spirv_supported() -> bool {
        shader::check_spirv_support()
    }
}

pub mod color {
    use crate::core::math::{Vec3, Vec4};
    use nalgebra_glm::pow;

    pub fn srgb_to_linear(value: f32) -> f32 {
        value.powf(2.2)
    }

    pub fn srgb_to_linear4f(color: &Vec4) -> Vec4 {
        pow(&color, &Vec4::new(2.2, 2.2, 2.2, 2.2))
    }

    pub fn srgb_to_linear3f(color: &Vec3) -> Vec3 {
        pow(&color, &Vec3::new(2.2, 2.2, 2.2))
    }
}
