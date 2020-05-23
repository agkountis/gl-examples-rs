use crate::rendering::texture::Texture2D;
use crate::rendering::program_pipeline::ProgramPipeline;
use crate::rendering::shader::{Shader, ShaderStage};
use crate::rendering::sampler::{Sampler, MinificationFilter, MagnificationFilter, WrappingMode};
use crate::core::math::Vec4;
use std::rc::Rc;
use std::path::Path;

pub trait Material {
    fn bind(&self);
    fn unbind(&self);
    fn program_pipeline(&self) -> &ProgramPipeline;
}

pub struct PbsMetallicRoughnessMaterial {
    pub albedo: Rc<Texture2D>,
    pub metallic: Rc<Texture2D>,
    pub roughness: Rc<Texture2D>,
    pub normals:  Rc<Texture2D>,
    pub ao: Rc<Texture2D>,
    pub ibl_brdf_lut: Rc<Texture2D>,
    sampler: Sampler,
    program_pipeline: ProgramPipeline
}

impl PbsMetallicRoughnessMaterial {
    pub fn new<P: AsRef<Path>>(asset_path: P,
                               albedo: Rc<Texture2D>,
                               metallic: Rc<Texture2D>,
                               roughness: Rc<Texture2D>,
                               normals: Rc<Texture2D>,
                               ao: Rc<Texture2D>,
                               ibl_brdf_lut: Rc<Texture2D>) -> Self {

        let program_pipeline = ProgramPipeline::new()
            .add_shader(&Shader::new_from_text(ShaderStage::Vertex,
                                               asset_path.as_ref().join("sdr/pbs.vert")).unwrap())
            .add_shader(&Shader::new_from_text(ShaderStage::Fragment,
                                               asset_path.as_ref().join("sdr/pbs.frag")).unwrap())
            .build()
            .unwrap();

        let sampler = Sampler::new(MinificationFilter::LinearMipmapLinear,
                                   MagnificationFilter::Linear,
                                   WrappingMode::Repeat,
                                   WrappingMode::Repeat,
                                   WrappingMode::Repeat,
                                   Vec4::new(0.0, 0.0, 0.0, 0.0));

        Self {
            albedo,
            metallic,
            roughness,
            normals,
            ao,
            ibl_brdf_lut,
            sampler,
            program_pipeline
        }
    }

    pub fn set_program_pipeline(&mut self, program_pipeline: ProgramPipeline) {
        self.program_pipeline = program_pipeline
    }
}

impl Material for PbsMetallicRoughnessMaterial{
    fn bind(&self) {
        self.program_pipeline.bind();

        self.program_pipeline
            .set_texture_2d("albedoMap",
                            &self.albedo,
                            &self.sampler,
                            ShaderStage::Fragment)
            .set_texture_2d("metallicMap",
                            &self.metallic,
                            &self.sampler,
                            ShaderStage::Fragment)
            .set_texture_2d("roughnessMap",
                            &self.roughness,
                            &self.sampler,
                            ShaderStage::Fragment)
            .set_texture_2d("normalMap",
                            &self.normals,
                            &self.sampler,
                            ShaderStage::Fragment)
            .set_texture_2d("aoMap",
                            &self.ao,
                            &self.sampler,
                            ShaderStage::Fragment)
            .set_texture_2d("brdfLUT",
                            &self.ibl_brdf_lut,
                            &self.sampler,
                            ShaderStage::Fragment);
    }

    fn unbind(&self) {
        self.program_pipeline.unbind();
    }

    fn program_pipeline(&self) -> &ProgramPipeline {
        &self.program_pipeline
    }
}
