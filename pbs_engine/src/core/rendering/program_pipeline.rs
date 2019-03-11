use pbs_gl as gl;
use gl::types::GLuint;

use super::shader::Shader;
use crate::core::rendering::shader::ShaderStage;

pub struct ProgramPipeline {
    id: GLuint,
    shaders: [Shader; 6],
    shader_programs: [GLuint; 6]
}

impl ProgramPipeline {

    pub fn new(shaders: &[Shader]) -> ProgramPipeline {
        assert!(!shaders.is_empty(), "Cannot create program pipeline. No shaders provided");

        let mut id: GLuint = 0;

        unsafe {
            gl::CreateProgramPipelines(1, &mut id);
            gl::BindProgramPipeline(id);

            //TODO
        }

        ProgramPipeline {
            id,
            shaders: [Shader{ id: 0, stage: ShaderStage::Vertex }; 6],
            shader_programs: [0; 6]
        }
    }

}
