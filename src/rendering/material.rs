use crate::{
    core::math::{Vec3, Vec4},
    imgui::{im_str, ColorFormat, Gui, Ui},
    rendering::{
        program_pipeline::ProgramPipeline,
        sampler::{MagnificationFilter, MinificationFilter, Sampler, WrappingMode},
        shader::{Shader, ShaderStage},
        texture::Texture2D,
    },
};
use std::{ops::RangeInclusive, path::Path, rc::Rc};

const ALBEDO_MAP_UNIFORM_NAME: &str = "albedoMap";
// [Metalness (R), Roughness (G), AO (B)]
const M_R_AO_MAP_UNIFORM_NAME: &str = "m_r_aoMap";
const NORMAL_MAP_UNIFORM_NAME: &str = "normalMap";
const DISPLACEMENT_MAP_UNIFORM_NAME: &str = "displacementMap";
const BRDF_LUT_MAP_UNIFORM_NAME: &str = "brdfLUT";
const BASE_COLOR_UNIFORM_NAME: &str = "baseColor";
const M_R_AO_SCALE_UNIFORM_NAME: &str = "m_r_aoScale";

pub trait Material: Gui {
    fn bind(&self);
    fn unbind(&self);
    fn program_pipeline(&self) -> &ProgramPipeline;
}

pub struct PbsMetallicRoughnessMaterial {
    pub albedo: Rc<Texture2D>,
    pub metallic_roughness_ao: Rc<Texture2D>,
    pub normals: Rc<Texture2D>,
    pub displacement: Option<Rc<Texture2D>>,
    pub ibl_brdf_lut: Rc<Texture2D>,
    pub base_color: Vec3,
    pub metallic_scale: f32,
    pub roughness_scale: f32,
    pub ao_scale: f32,
    sampler: Sampler,
    program_pipeline: ProgramPipeline,
}

impl PbsMetallicRoughnessMaterial {
    pub fn new<P: AsRef<Path>>(
        asset_path: P,
        albedo: Rc<Texture2D>,
        metallic_roughness_ao: Rc<Texture2D>,
        normals: Rc<Texture2D>,
        displacement: Option<Rc<Texture2D>>,
        ibl_brdf_lut: Rc<Texture2D>,
    ) -> Self {
        let (vertex_shader, fragment_shader) = match displacement {
            Some(_) => (
                Shader::new_from_text(
                    ShaderStage::Vertex,
                    asset_path.as_ref().join("sdr/pbs_pom.vert"),
                )
                .unwrap(),
                Shader::new_from_text(
                    ShaderStage::Fragment,
                    asset_path.as_ref().join("sdr/pbs_pom.frag"),
                )
                .unwrap(),
            ),
            None => (
                Shader::new_from_text(
                    ShaderStage::Vertex,
                    asset_path.as_ref().join("sdr/pbs.vert"),
                )
                .unwrap(),
                Shader::new_from_text(
                    ShaderStage::Fragment,
                    asset_path.as_ref().join("sdr/pbs.frag"),
                )
                .unwrap(),
            ),
        };

        let program_pipeline = ProgramPipeline::new()
            .add_shader(&vertex_shader)
            .add_shader(&fragment_shader)
            .build()
            .unwrap();

        let sampler = Sampler::new(
            MinificationFilter::LinearMipmapLinear,
            MagnificationFilter::Linear,
            WrappingMode::Repeat,
            WrappingMode::Repeat,
            WrappingMode::Repeat,
            Vec4::new(0.0, 0.0, 0.0, 0.0),
        );

        Self {
            albedo,
            metallic_roughness_ao,
            normals,
            displacement,
            ibl_brdf_lut,
            base_color: Vec3::new(1.0, 1.0, 1.0),
            metallic_scale: 1.0,
            roughness_scale: 1.0,
            ao_scale: 1.0,
            sampler,
            program_pipeline,
        }
    }

    pub fn set_program_pipeline(&mut self, program_pipeline: ProgramPipeline) {
        self.program_pipeline = program_pipeline
    }
}

impl Material for PbsMetallicRoughnessMaterial {
    fn bind(&self) {
        self.program_pipeline.bind();

        self.program_pipeline
            .set_texture_2d(
                ALBEDO_MAP_UNIFORM_NAME,
                &self.albedo,
                &self.sampler,
                ShaderStage::Fragment,
            )
            .set_texture_2d(
                M_R_AO_MAP_UNIFORM_NAME,
                &self.metallic_roughness_ao,
                &self.sampler,
                ShaderStage::Fragment,
            )
            .set_texture_2d(
                NORMAL_MAP_UNIFORM_NAME,
                &self.normals,
                &self.sampler,
                ShaderStage::Fragment,
            )
            .set_texture_2d(
                BRDF_LUT_MAP_UNIFORM_NAME,
                &self.ibl_brdf_lut,
                &self.sampler,
                ShaderStage::Fragment,
            )
            .set_vector3f(
                BASE_COLOR_UNIFORM_NAME,
                &self.base_color,
                ShaderStage::Fragment,
            )
            .set_vector3f(
                M_R_AO_SCALE_UNIFORM_NAME,
                &Vec3::new(self.metallic_scale, self.roughness_scale, self.ao_scale),
                ShaderStage::Fragment,
            );

        if let Some(displacement) = &self.displacement {
            self.program_pipeline.set_texture_2d(
                DISPLACEMENT_MAP_UNIFORM_NAME,
                &displacement,
                &self.sampler,
                ShaderStage::Fragment,
            );
        }
    }

    fn unbind(&self) {
        self.program_pipeline.unbind();
    }

    fn program_pipeline(&self) -> &ProgramPipeline {
        &self.program_pipeline
    }
}

impl Gui for PbsMetallicRoughnessMaterial {
    fn gui(&mut self, ui: &Ui) {
        if imgui::CollapsingHeader::new(im_str!("Material"))
            .default_open(true)
            .open_on_arrow(true)
            .open_on_double_click(true)
            .build(ui)
        {
            ui.spacing();
            ui.group(|| {
                ui.group(|| {
                    ui.text(im_str!("Albedo Map"));
                    imgui::Image::new((self.albedo.get_id() as usize).into(), [128.0, 128.0])
                        .build(&ui);
                    ui.spacing();

                    let mut albedo_color: [f32; 3] = self.base_color.into();
                    if imgui::ColorEdit::new(im_str!("Base Color"), &mut albedo_color)
                        .format(ColorFormat::Float)
                        .alpha(true)
                        .hdr(false)
                        .picker(true)
                        .build(&ui)
                    {
                        self.base_color = albedo_color.into()
                    }
                });
                ui.spacing();
                ui.spacing();
                ui.group(|| {
                    ui.text(im_str!("Metallic/Roughness/Ao Map"));
                    imgui::Image::new(
                        (self.metallic_roughness_ao.get_id() as usize).into(),
                        [128.0, 128.0],
                    )
                    .build(&ui);
                    ui.spacing();
                    imgui::Slider::new(im_str!("Metallic Scale"), RangeInclusive::new(0.0, 1.0))
                        .display_format(im_str!("%.2f"))
                        .build(&ui, &mut self.metallic_scale);
                    imgui::Slider::new(im_str!("Roughness Scale"), RangeInclusive::new(0.0, 1.0))
                        .display_format(im_str!("%.2f"))
                        .build(&ui, &mut self.roughness_scale);
                    imgui::Slider::new(im_str!("AO Scale"), RangeInclusive::new(0.0, 1.0))
                        .display_format(im_str!("%.2f"))
                        .build(&ui, &mut self.ao_scale);
                });
                ui.new_line()
            });
        }
    }
}
