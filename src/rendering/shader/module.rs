use std::fmt::Debug;
use std::{ffi::CString, ptr};

use gl::types::*;
use gl_bindings as gl;
use shaderc::CompilationArtifact;

use crate::shader::ShaderStage;

#[derive(Debug)]
pub(crate) struct ShaderModule {
    id: GLuint,
    stage: ShaderStage,
}

impl ShaderModule {
    pub fn new(
        stage: ShaderStage,
        compilation_artifact: &CompilationArtifact,
    ) -> Result<ShaderModule, String> {
        if cfg!(feature = "use-spirv") {
            let spirv = compilation_artifact.as_binary_u8();
            Self::new_from_spirv(stage, spirv)
        } else {
            let text_source = compilation_artifact.as_text();
            Self::new_from_text(stage, &text_source)
        }
    }

    fn new_from_spirv(stage: ShaderStage, spir_v: &[u8]) -> Result<ShaderModule, String> {
        let id: GLuint;

        unsafe {
            id = gl::CreateShader(stage as u32);

            gl::ShaderBinary(
                1,
                &id,
                gl::SHADER_BINARY_FORMAT_SPIR_V_ARB,
                spir_v.as_ptr() as *const GLvoid,
                spir_v.len() as i32,
            );

            let cstr = CString::new("main").unwrap();

            // Specify shader module entry point and specialization constants
            gl::SpecializeShaderARB(
                id, //obj id
                cstr.as_ptr() as *const GLchar,
                0,               // no specialization constants
                ptr::null_mut(), // no specialization constants
                ptr::null_mut(),
            ); // no specialization constants

            let mut compilation_status: GLint = 0;

            gl::GetShaderiv(id, gl::COMPILE_STATUS, &mut compilation_status);

            if compilation_status != gl::TRUE as i32 {
                let mut message_size = 0;

                gl::GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut message_size);

                let mut buffer = Vec::with_capacity(message_size as usize + 1); //+1 for nul termination

                buffer.extend([b' '].iter().cycle().take(message_size as usize));

                let message = CString::from_vec_unchecked(buffer);

                gl::GetShaderInfoLog(
                    id,
                    message_size as i32,
                    ptr::null_mut(),
                    message.as_ptr() as *mut GLchar,
                );

                return Err(message.to_string_lossy().into_owned());
            }
        }

        Ok(ShaderModule { id, stage })
    }

    fn new_from_text(stage: ShaderStage, text_source: &str) -> Result<ShaderModule, String> {
        let id: GLuint;
        let c_string_source = CString::new(text_source).unwrap();

        unsafe {
            id = gl::CreateShader(stage as u32);

            gl::ShaderSource(id, 1, &c_string_source.as_ptr(), ptr::null());

            gl::CompileShader(id);

            let mut compilation_status: GLint = 0;

            gl::GetShaderiv(id, gl::COMPILE_STATUS, &mut compilation_status);

            if compilation_status != gl::TRUE as i32 {
                let mut message_size = 0;

                gl::GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut message_size);

                let mut buffer = Vec::with_capacity(message_size as usize + 1); //+1 for nul termination

                buffer.extend([b' '].iter().cycle().take(message_size as usize));

                let message = CString::from_vec_unchecked(buffer);

                gl::GetShaderInfoLog(
                    id,
                    message_size as i32,
                    ptr::null_mut(),
                    message.as_ptr() as *mut GLchar,
                );

                return Err(message.to_string_lossy().into_owned());
            }

            Ok(ShaderModule { id, stage })
        }
    }

    pub fn id(&self) -> GLuint {
        self.id
    }

    pub fn stage(&self) -> ShaderStage {
        self.stage
    }
}

impl Drop for ShaderModule {
    fn drop(&mut self) {
        unsafe { gl::DeleteShader(self.id) }
    }
}
