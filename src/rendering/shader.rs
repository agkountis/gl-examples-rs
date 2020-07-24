use gl::types::*;
use gl_bindings as gl;
use std::ffi::CString;
use std::fmt::Debug;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::ptr;

pub fn check_spirv_support() -> bool {
    let mut format_count: GLint = 0;

    unsafe {
        gl::GetIntegerv(gl::NUM_SHADER_BINARY_FORMATS, &mut format_count);
    }

    if format_count > 0 {
        let mut formats = vec![0; format_count as usize];

        unsafe {
            gl::GetIntegerv(gl::SHADER_BINARY_FORMATS, formats.as_mut_ptr());
        }

        let opt = formats
            .iter()
            .find(|&m| *m == gl::SHADER_BINARY_FORMAT_SPIR_V_ARB as i32);
        match opt {
            Some(_) => return true,
            _ => false,
        };
    }

    false
}

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum ShaderStage {
    Vertex = gl::VERTEX_SHADER,
    TesselationControl = gl::TESS_CONTROL_SHADER,
    TesselationEvaluation = gl::TESS_EVALUATION_SHADER,
    Geometry = gl::GEOMETRY_SHADER,
    Fragment = gl::FRAGMENT_SHADER,
    Compute = gl::COMPUTE_SHADER,
}

#[derive(Debug, Clone)]
pub struct Shader {
    id: GLuint,
    stage: ShaderStage,
}

impl Shader {
    pub fn new_from_spirv<P: AsRef<Path>>(
        stage: ShaderStage,
        entry_point: &str,
        filename: P,
    ) -> Result<Shader, String> {
        let mut spir_v = Vec::new();

        {
            let mut file = File::open(filename.as_ref()).unwrap();

            let file_size_in_bytes = file.metadata().unwrap().len() as usize;

            let bytes_read = file.read_to_end(&mut spir_v).unwrap();

            assert_eq!(
                bytes_read, file_size_in_bytes,
                "Could not read the entirety of the file."
            );
        }

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

            let cstr = CString::new(entry_point).unwrap();

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

        Ok(Shader { id, stage })
    }

    pub fn new_from_text<P: AsRef<Path> + Debug>(
        stage: ShaderStage,
        path: P,
    ) -> Result<Shader, String> {
        let mut text_source = String::new();

        {
            let mut file = match File::open(path.as_ref()) {
                Err(why) => panic!("couldn't open {:?}: {}", path, why),
                Ok(file) => file,
            };

            let size = file.read_to_string(&mut text_source).unwrap();

            assert_eq!(
                size,
                text_source.len(),
                "Could not read the entirety of the file."
            );
        }

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

            Ok(Shader { id, stage })
        }
    }

    pub fn get_id(&self) -> GLuint {
        self.id
    }

    pub fn get_stage(&self) -> ShaderStage {
        self.stage
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe { gl::DeleteShader(self.id) }
    }
}
