use gl;
use glm;

use gl::types::*;

pub enum MinificationFilter {
    Nearest = gl::NEAREST,
    Linear = gl::LINEAR,
    NearestMipmapNearest = gl::NEAREST_MIPMAP_NEAREST,
    LinearMipmapNearest = gl::LINEAR_MIPMAP_NEAREST,
    LinearMipmapLinear = gl::LINEAR_MIPMAP_LINEAR
}

pub enum MagnificationFilter {
    Nearest = gl::NEAREST,
    Linear = gl::LINEAR
}

pub enum WrappingMode {
    Repeat = gl::REPEAT,
    ClampToEdge = gl::CLAMP_TO_EDGE,
    MirroredRepeat = gl::MIRRORED_REPEAT
}

pub struct Sampler {
    pub id: GLuint
}

impl Sampler {

    pub fn new(min_filter: MinificationFilter, mag_filter: MagnificationFilter,
               wrap_s: WrappingMode, wrap_t: WrappingMode, wrap_r: WrappingMode,
               border_color: glm::Vec4f) -> Sampler {
        gl::ClearColor();
        let id = gl::CreateSamplers(1);
    }

}
