use crate::math::Vec3;

#[repr(C)]
#[derive(Debug)]
pub enum Light {
    Directional { direction: Vec3, temperature: u32 },
    Point,
    Spotlight,
}
