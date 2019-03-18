use pbs_gl as gl;
use gl::types::GLuint;
use std::ptr;

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

    pub fn new(filename: &str) {

        let mut id: GLuint = 0;

        unsafe {
            id = gl::CreateShader(gl::VERTEX_SHADER);
            //TEST: Spirv seems to be supported.
            gl::ShaderBinary(1,
                             &id,
                             gl::SHADER_BINARY_FORMAT_SPIR_V_ARB,
                             ptr::null_mut(),
                             0);

            // Specify shader module entry point and specialization cosntants
            gl::SpecializeShaderARB(id, //obj id
                                  "main".as_ptr() as *const i8, //TEST THIS: Seems weird that this call expects a *const i8. The C call expects a GLchar*
                                  0, // no specialization constants
                                  ptr::null_mut(), // no specialization constants
                                  ptr::null_mut()); // no specialization constants
        }

    }

}
