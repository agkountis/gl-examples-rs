mod compiler;
pub(crate) mod module;
mod program;

use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::{fmt::Debug, path::Path};

use crate::rendering::sampler::Sampler;
use crate::rendering::shader::module::ShaderModule;
use crate::rendering::shader::{compiler::SHADER_COMPILER, program::ShaderProgram};
use crate::rendering::texture::{Texture2D, TextureCube};
use crate::shader::program::ShaderProgramBuilder;
use gl::types::*;
use gl_bindings as gl;
use itertools::Itertools;
use shaderc::CompilationArtifact;

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
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ShaderStage {
    Vertex = gl::VERTEX_SHADER,
    TesselationControl = gl::TESS_CONTROL_SHADER,
    TesselationEvaluation = gl::TESS_EVALUATION_SHADER,
    Geometry = gl::GEOMETRY_SHADER,
    Fragment = gl::FRAGMENT_SHADER,
}

pub struct ShaderBuilder<'a> {
    name: String,
    keyword_sets: Vec<HashSet<&'a str>>,
    modules: Vec<(ShaderStage, PathBuf)>,
    precompiled_modules: Vec<&'a ShaderModule>,
}

impl<'a> ShaderBuilder<'a> {
    pub fn new(name: &str) -> ShaderBuilder<'a> {
        Self {
            name: name.into(),
            keyword_sets: vec![],
            modules: vec![],
            precompiled_modules: vec![],
        }
    }

    pub fn with_stage<P: 'a + AsRef<Path>>(mut self, stage: ShaderStage, file_path: P) -> Self {
        self.modules.push((stage, file_path.as_ref().into()));
        self
    }

    pub(crate) fn with_module(mut self, shader_module: &'a ShaderModule) -> Self {
        self.precompiled_modules.push(shader_module);
        self
    }

    pub fn with_keyword_set(mut self, keyword_set: &'a [&'a str]) -> Self {
        let keyword_hash_set = keyword_set.iter().cloned().collect::<HashSet<&str>>();
        self.keyword_sets.push(keyword_hash_set);
        self
    }

    pub fn build(self) -> Shader {
        let keyword_bitfield_map = self
            .keyword_sets
            .iter()
            .flatten()
            .filter(|&s| s != &"_")
            .map(|&s| String::from(s))
            .enumerate()
            .map(|(i, a)| (a, 1u64 << i))
            .collect::<HashMap<_, _>>();

        println!("{:?}", keyword_bitfield_map);

        let modules = self
            .modules
            .iter()
            .map(|(stage, path)| {
                let file_name = path.file_name().unwrap().to_string_lossy().to_string();
                let source = Self::load_shader_source(path);
                (file_name, stage, source)
            })
            .collect::<Vec<_>>();

        let shader_variants = self
            .keyword_sets
            .iter()
            .combinations(self.keyword_sets.len())
            .multi_cartesian_product()
            .flatten()
            .map(|mut keywords| {
                keywords.retain(|s| s != &"_");
                keywords
            })
            .map(|keywords| {
                let compiled_modules = modules
                    .iter()
                    .map(|(file_name, &stage, source)| {
                        (
                            stage,
                            Self::compile_module(file_name, stage, source, &keywords),
                        )
                    })
                    .collect::<Vec<_>>();
                (keywords, compiled_modules)
            })
            .map(|(keywords, mut compiled_modules)| {
                let bitfield = keywords
                    .iter()
                    .map(|s| String::from(s))
                    .fold(0u64, |acc, s| acc | keyword_bitfield_map[&s]);

                let mut program_builder = ShaderProgramBuilder::new();

                compiled_modules
                    .iter()
                    .for_each(|(stage, compilation_artifact)| {
                        program_builder.with_shader_module(
                            &ShaderModule::new(*stage, compilation_artifact)
                                .expect("Failed to create shader module."),
                        );
                    });

                self.precompiled_modules.iter().for_each(|&module| {
                    program_builder.with_shader_module(module);
                });

                let program = program_builder
                    .build()
                    .expect("Failed to create shader program.");

                (bitfield, program)
            })
            .collect::<HashMap<u64, ShaderProgram>>();

        Shader {
            name: self.name,
            active_variant: 0,
            active_variant_bitfield: 0,
            shader_variants,
            keyword_bitfield_map,
        }
    }

    fn load_shader_source<P: AsRef<Path> + Debug>(path: P) -> String {
        let mut source = String::new();

        {
            let mut file = match File::open(path.as_ref()) {
                Err(why) => panic!("couldn't open {:?}: {}", path, why),
                Ok(file) => file,
            };

            let size = file.read_to_string(&mut source).unwrap();

            assert_eq!(
                size,
                source.len(),
                "Could not read the entirety of the file."
            );
        }

        source
    }

    fn compile_module(
        file_name: &str,
        stage: ShaderStage,
        source: &str,
        keywords: &[&str],
    ) -> CompilationArtifact {
        let res = unsafe { SHADER_COMPILER.compile(&source, file_name, stage, keywords) };
        match res {
            Ok(artifact) => artifact,
            Err(e) => panic!("{}", e),
        }
    }
}

mod tests {
    use crate::shader::ShaderBuilder;

    #[test]
    fn test_foo() {
        ShaderBuilder::new("test")
            .with_keyword_set(&["_", "FOO", "BLA"])
            .with_keyword_set(&["_", "BAR"])
            .with_keyword_set(&["LOL"])
            .build();
    }
}

#[derive(Debug)]
pub struct Shader {
    name: String,
    active_variant: GLuint,
    active_variant_bitfield: u64,
    shader_variants: HashMap<u64, ShaderProgram>,
    keyword_bitfield_map: HashMap<String, u64>,
}

impl Shader {
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

    pub fn enable_keyword(&mut self, keyword: &str) {
        let bits = self.keyword_bitfield_map[keyword];
        self.active_variant_bitfield = self.active_variant | bits;
        self.active_variant = self.shader_variants[self.active_variant_bitfield];
    }

    pub fn disable_keyword(&mut self, keyword: &str) {
        let bits = self.keyword_bitfield_map[keyword];
        self.active_variant_bitfield = self.active_variant & bits;
        self.active_variant = self.shader_variants[self.active_variant_bitfield];
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindProgramPipeline(self.active_variant);
        }
    }

    pub fn unbind(&self) {
        unsafe {
            gl::BindProgramPipeline(0);
        }
    }
}

pub struct ComputeShader {}

impl ComputeShader {
    pub fn dispatch(&self) {}
}
