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
