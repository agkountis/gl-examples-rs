pub mod shader_manager;

mod compiler;
pub(crate) mod module;
mod program;

use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::{fmt::Debug, path::Path};

use crate::rendering::sampler::Sampler;
use crate::rendering::shader::program::ShaderProgram;
use crate::rendering::texture::{Texture2D, TextureCube};
use gl::types::*;
use gl_bindings as gl;
use itertools::Itertools;

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

pub struct ShaderCreateInfo<'a> {
    name: String,
    keyword_sets: Vec<Vec<&'a str>>,
    stages: Vec<(ShaderStage, PathBuf)>,
}

impl<'a> ShaderCreateInfo<'a> {
    pub fn builder(shader_name: &str) -> ShaderCreateInfoBuilder<'a> {
        ShaderCreateInfoBuilder {
            name: shader_name.into(),
            ..Default::default()
        }
    }
}

#[derive(Default)]
pub struct ShaderCreateInfoBuilder<'a> {
    name: String,
    keyword_sets: Vec<Vec<&'a str>>,
    modules: Vec<(ShaderStage, PathBuf)>,
}

impl<'a> ShaderCreateInfoBuilder<'a> {
    pub fn stage<P: 'a + AsRef<Path>>(mut self, stage: ShaderStage, file_path: P) -> Self {
        self.modules.push((stage, file_path.as_ref().into()));
        self
    }

    pub fn keyword_set(mut self, keyword_set: &'a [&'a str]) -> Self {
        let keyword_set = keyword_set.iter().copied().unique().collect_vec();
        self.keyword_sets.push(keyword_set);
        self
    }

    pub fn build(self) -> ShaderCreateInfo<'a> {
        let keyword_sets = if self.keyword_sets.is_empty() {
            let mut set = vec!["_"];
            vec![set]
        } else {
            self.keyword_sets
        };

        ShaderCreateInfo {
            name: self.name,
            keyword_sets,
            stages: self.modules,
        }
    }
}

#[derive(Debug)]
pub struct Shader {
    name: String,
    bound: RefCell<bool>,
    active_variant: RefCell<GLuint>,
    active_variant_bitfield: RefCell<u32>,
    shader_variants: HashMap<u32, ShaderProgram>,
    keyword_bitfield_map: HashMap<String, u32>,
}

impl Shader {
    pub fn bind_texture_2d(&self, location: u32, texture: &Texture2D, sampler: &Sampler) -> &Self {
        unsafe {
            gl::BindTextureUnit(location as GLuint, texture.get_id());
            gl::BindSampler(location as GLuint, sampler.id)
        }

        self
    }

    pub fn bind_texture_2d_with_id(
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

    pub fn bind_texture_cube(
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

    pub fn enable_keyword(&self, keyword: &str) {
        if let Some(&bits) = self.keyword_bitfield_map.get(keyword) {
            {
                let mut active_variant_bitfield = self.active_variant_bitfield.borrow_mut();

                //TODO: the final bitfield must be a combination of each keyword set's bitfield.

                *active_variant_bitfield |= bits;
                self.set_active_shader_variant(*active_variant_bitfield);
            }

            self.bind_internal();
            *self.bound.borrow_mut() = true
        } else {
            eprintln!("ERROR: Keyword {} not found!", keyword)
        }
    }

    pub fn disable_keyword(&self, keyword: &str) {
        if let Some(&bits) = self.keyword_bitfield_map.get(keyword) {
            let mut active_variant_bitfield = self.active_variant_bitfield.borrow_mut();

            *active_variant_bitfield &= !bits;

            self.set_active_shader_variant(*active_variant_bitfield);
            self.bind_internal();
            *self.bound.borrow_mut() = true
        }
    }

    pub fn bind(&self) {
        let mut bound = self.bound.borrow_mut();
        if !*bound {
            self.bind_internal();
            *bound = true
        }
    }

    pub fn unbind(&self) {
        unsafe {
            gl::BindProgramPipeline(0);
        }
        *self.bound.borrow_mut() = false;
    }

    fn bind_internal(&self) {
        unsafe {
            gl::BindProgramPipeline(*self.active_variant.borrow());
        }
    }

    fn set_active_shader_variant(&self, bitfield: u32) {
        let mut active_variant = self.active_variant.borrow_mut();
        *active_variant = self
            .shader_variants
            .get(&bitfield)
            .map_or_else(|| *active_variant, |variant| variant.id());
    }
}

pub struct ComputeShader {}

impl ComputeShader {
    pub fn dispatch(&self) {}
}
