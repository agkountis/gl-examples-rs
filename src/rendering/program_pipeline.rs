use gl::types::*;
use gl_bindings as gl;
use std::ffi::CString;
use std::ptr;

use crate::rendering::{
    sampler::Sampler,
    shader::{Shader, ShaderStage},
    texture::{Texture2D, TextureCube},
};

pub struct ProgramPipeline {
    id: GLuint,
    shaders: [Option<(ShaderStage, GLuint)>; 6],
    shader_programs: [Option<GLuint>; 6],
}

impl ProgramPipeline {
    pub fn new() -> ProgramPipeline {
        let mut id: GLuint = 0;

        unsafe {
            gl::CreateProgramPipelines(1, &mut id);
        }

        ProgramPipeline {
            id,
            shaders: [None; 6],
            shader_programs: [None; 6],
        }
    }

    pub fn add_shader(mut self, shader: &Shader) -> Self {
        let idx = Self::shader_stage_to_array_index(shader.get_stage());

        if let Some(ref sdr) = self.shaders[idx] {
            println!(
                "Shader of type {:?} already exists in the program pipeline... Replacing...",
                shader.get_stage()
            )
        }

        self.shaders[idx] = Some((shader.get_stage(), shader.get_id()));

        self
    }

    pub fn build(mut self) -> Result<Self, String> {
        unsafe {
            for option in self.shaders.iter() {
                if let Some(shader) = option {
                    let program_id = gl::CreateProgram();

                    //must be called before linking
                    gl::ProgramParameteri(program_id, gl::PROGRAM_SEPARABLE, gl::TRUE as i32);

                    gl::AttachShader(program_id, shader.1);

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

                    let idx = Self::shader_stage_to_array_index(shader.0);
                    self.shader_programs[idx] = Some(program_id);

                    gl::UseProgramStages(
                        self.id,
                        Self::shader_stage_to_gl_bitfield(shader.0),
                        program_id,
                    )
                }
            }
        }

        Ok(self)
    }

    pub fn set_texture_2d(&self, location: u32, texture: &Texture2D, sampler: &Sampler) -> &Self {
        unsafe {
            gl::BindTextureUnit(location as GLuint, texture.get_id());
            gl::BindSampler(location as GLuint, sampler.id)
        }

        self
    }

    pub fn set_texture_2d_with_id(
        &self,
        binding_location: u32,
        texture_id: u32,
        sampler: &Sampler,
    ) -> &Self {
        unsafe {
            gl::BindTextureUnit(binding_location as GLuint, texture_id);
            gl::BindSampler(binding_location as GLuint, sampler.id)
        }

        self
    }

    pub fn set_texture_cube(
        &self,
        binding_location: u32,
        texture: &TextureCube,
        sampler: &Sampler,
    ) -> &Self {
        unsafe {
            gl::BindTextureUnit(binding_location as GLuint, texture.get_id());
            gl::BindSampler(binding_location as GLuint, sampler.id)
        }

        self
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindProgramPipeline(self.id);
        }
    }

    pub fn unbind(&self) {
        unsafe {
            gl::BindProgramPipeline(0);
        }
    }

    fn shader_stage_to_array_index(shader_type: ShaderStage) -> usize {
        match shader_type {
            ShaderStage::Vertex => 0,
            ShaderStage::TesselationControl => 1,
            ShaderStage::TesselationEvaluation => 2,
            ShaderStage::Geometry => 3,
            ShaderStage::Fragment => 4,
            ShaderStage::Compute => 5,
        }
    }

    fn shader_stage_to_gl_bitfield(stage: ShaderStage) -> GLbitfield {
        match stage {
            ShaderStage::Vertex => gl::VERTEX_SHADER_BIT,
            ShaderStage::TesselationControl => gl::TESS_CONTROL_SHADER_BIT,
            ShaderStage::TesselationEvaluation => gl::TESS_EVALUATION_SHADER_BIT,
            ShaderStage::Geometry => gl::GEOMETRY_SHADER_BIT,
            ShaderStage::Fragment => gl::FRAGMENT_SHADER_BIT,
            ShaderStage::Compute => gl::COMPUTE_SHADER_BIT,
        }
    }
}

impl Drop for ProgramPipeline {
    fn drop(&mut self) {
        unsafe { gl::DeleteProgramPipelines(1, &self.id) }
    }
}
