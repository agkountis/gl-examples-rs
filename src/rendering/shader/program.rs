use gl::types::*;
use gl_bindings as gl;
use std::ffi::CString;
use std::ptr;

use crate::rendering::shader::module::ShaderModule;
use crate::rendering::shader::ShaderStage;

#[derive(Debug)]
pub struct ShaderProgram {
    id: GLuint,
}

impl ShaderProgram {
    pub fn id(&self) -> GLuint {
        self.id
    }
}

pub struct ShaderProgramBuilder<'a> {
    modules: [Option<&'a ShaderModule>; 5],
}

impl Default for ShaderProgramBuilder<'_> {
    fn default() -> Self {
        Self { modules: [None; 5] }
    }
}

impl<'a> ShaderProgramBuilder<'a> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_shader_module(mut self, shader: &'a ShaderModule) -> Self {
        let idx = Self::shader_stage_to_array_index(shader.stage());

        if self.modules[idx].is_some() {
            println!(
                "Shader module of type {:?} already exists in the program pipeline... Replacing...",
                shader.stage()
            )
        }

        self.modules[idx] = Some(shader);

        self
    }

    pub fn build(self) -> Result<ShaderProgram, String> {
        let mut pipeline_id: GLuint = 0;

        unsafe {
            gl::CreateProgramPipelines(1, &mut pipeline_id);
        }

        for &module in self.modules.iter().flatten() {
            let shader = module.id();
            let stage = module.stage();

            unsafe {
                let program_id = gl::CreateProgram();

                //must be called before linking
                gl::ProgramParameteri(program_id, gl::PROGRAM_SEPARABLE, gl::TRUE as i32);

                gl::AttachShader(program_id, shader);

                gl::LinkProgram(program_id);

                let mut link_status: GLint = 0;
                gl::GetProgramiv(program_id, gl::LINK_STATUS, &mut link_status);

                if link_status != gl::TRUE as i32 {
                    let mut message_size = 0;

                    gl::GetProgramiv(program_id, gl::INFO_LOG_LENGTH, &mut message_size);

                    //+1 for nul termination
                    let mut buffer = Vec::with_capacity(message_size as usize + 1);

                    buffer.extend([b' '].iter().cycle().take(message_size as usize));

                    let message = CString::from_vec_unchecked(buffer);

                    gl::GetProgramInfoLog(
                        program_id,
                        message_size as i32,
                        ptr::null_mut(),
                        message.as_ptr() as *mut GLchar,
                    );

                    return Err(message.to_string_lossy().into_owned());
                }

                // let idx = Self::shader_stage_to_array_index(stage);
                // self.shader_programs[idx] = Some(program_id);

                gl::UseProgramStages(
                    pipeline_id,
                    Self::shader_stage_to_gl_bitfield(stage),
                    program_id,
                )
            }
        }

        Ok(ShaderProgram { id: pipeline_id })
    }

    fn shader_stage_to_array_index(shader_type: ShaderStage) -> usize {
        match shader_type {
            ShaderStage::Vertex => 0,
            ShaderStage::TesselationControl => 1,
            ShaderStage::TesselationEvaluation => 2,
            ShaderStage::Geometry => 3,
            ShaderStage::Fragment => 4,
        }
    }

    fn shader_stage_to_gl_bitfield(stage: ShaderStage) -> GLbitfield {
        match stage {
            ShaderStage::Vertex => gl::VERTEX_SHADER_BIT,
            ShaderStage::TesselationControl => gl::TESS_CONTROL_SHADER_BIT,
            ShaderStage::TesselationEvaluation => gl::TESS_EVALUATION_SHADER_BIT,
            ShaderStage::Geometry => gl::GEOMETRY_SHADER_BIT,
            ShaderStage::Fragment => gl::FRAGMENT_SHADER_BIT,
        }
    }
}

impl Drop for ShaderProgram {
    fn drop(&mut self) {
        unsafe { gl::DeleteProgramPipelines(1, &self.id) }
    }
}
