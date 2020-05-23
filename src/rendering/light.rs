use crate::math::Vec3;

#[derive(Debug)]
#[repr(C)]
struct DirectionalLight { //TODO: non physically accurate.
    pub direction: Vec3,
    pub diffuse: Vec3,
    pub specular: Vec3,
    pub ambient: Vec3
}


