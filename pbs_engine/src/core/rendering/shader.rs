use pbs_gl as gl;
use gl::types::GLuint;

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum ShaderStage {
    Vertex = gl::VERTEX_SHADER_BIT,
    TesselationControl,
    TesselationEvaluation,
    Geometry,
    Fragment,
    Compute
}

#[derive(Debug, Clone, Copy)]
pub struct Shader {
    pub id: GLuint,
    pub stage: ShaderStage
}

impl Shader {


}
