use super::math;

#[derive(Debug)]
#[repr(C)]
struct DirectionalLight { //TODO: non physically accurate.
    pub direction: Vec3f,
    pub diffuse: Vec3f,
    pub specular: Vec3f,
    pub ambient: Vec3f
}


