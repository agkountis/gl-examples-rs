use pbs_gl as gl;
use gl::types::GLuint;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32
}

impl Color {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Color {
        Color { r, g, b, a }
    }
}

pub struct Texture2D<'a> {
    id: GLuint,
    pixels: &'a [u8]
}

impl<'a> Texture2D<'a> {
    pub fn get_id(&self) -> GLuint {
        self.id
    }
}

pub struct TextureCube {
    id: GLuint
}

impl TextureCube {

}

