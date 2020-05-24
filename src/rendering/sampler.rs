use crate::core::math::utilities;
use crate::core::math::Vec4;
use gl_bindings as gl;

use gl::types::GLuint;

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum MinificationFilter {
    Nearest = gl::NEAREST,
    Linear = gl::LINEAR,
    NearestMipmapNearest = gl::NEAREST_MIPMAP_NEAREST,
    LinearMipmapNearest = gl::LINEAR_MIPMAP_NEAREST,
    LinearMipmapLinear = gl::LINEAR_MIPMAP_LINEAR,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum MagnificationFilter {
    Nearest = gl::NEAREST,
    Linear = gl::LINEAR,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum WrappingMode {
    Repeat = gl::REPEAT,
    ClampToEdge = gl::CLAMP_TO_EDGE,
    MirroredRepeat = gl::MIRRORED_REPEAT,
}

#[derive(Debug)]
pub struct Sampler {
    pub id: GLuint,
    pub min_filter: MinificationFilter,
    pub mag_filter: MagnificationFilter,
    pub wrap_s: WrappingMode,
    pub wrap_t: WrappingMode,
    pub wrap_r: WrappingMode,
    pub border_color: Vec4,
}

impl Sampler {
    pub fn new(
        min_filter: MinificationFilter,
        mag_filter: MagnificationFilter,
        wrap_s: WrappingMode,
        wrap_t: WrappingMode,
        wrap_r: WrappingMode,
        border_color: Vec4,
    ) -> Sampler {
        let mut id: GLuint = 0;

        unsafe {
            gl::CreateSamplers(1, &mut id);

            gl::SamplerParameteri(id, gl::TEXTURE_MIN_FILTER, min_filter as i32);
            gl::SamplerParameteri(id, gl::TEXTURE_MAG_FILTER, mag_filter as i32);
            gl::SamplerParameteri(id, gl::TEXTURE_WRAP_S, wrap_s as i32);
            gl::SamplerParameteri(id, gl::TEXTURE_WRAP_T, wrap_t as i32);
            gl::SamplerParameteri(id, gl::TEXTURE_WRAP_R, wrap_r as i32);
            gl::SamplerParameterfv(
                id,
                gl::TEXTURE_BORDER_COLOR,
                utilities::value_ptr(&border_color),
            );
        }

        Sampler {
            id,
            min_filter,
            mag_filter,
            wrap_s,
            wrap_t,
            wrap_r,
            border_color,
        }
    }
}

impl Drop for Sampler {
    fn drop(&mut self) {
        unsafe { gl::DeleteSamplers(1, &self.id) }
    }
}
