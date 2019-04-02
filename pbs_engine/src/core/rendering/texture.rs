use pbs_gl as gl;
use gl::types::GLuint;

pub struct Texture {
    id: GLuint
}

impl Texture {
    pub fn get_id(&self) -> GLuint {
        self.id
    }
}
