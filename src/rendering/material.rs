use crate::core::asset::Asset;
use crate::rendering::texture::Texture2DLoadConfig;
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
const M_R_AO_BIAS_UNIFORM_NAME: &str = "m_r_aoBias";
const POM_PARAMETERS_UNIFORM_NAME: &str = "pomParameters";
const PARALLAX_MAPPING_METHOD_UNIFORM_NAME: &str = "parallaxMappingMethod";

pub trait Material: Gui {
    fn bind(&self);
    fn unbind(&self);
    fn program_pipeline(&self) -> &ProgramPipeline;
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct MaterialPropertyBlock {
    base_color: Vec3,
    metallic_scale: f32,
    metallic_bias: f32,
    roughness_scale: f32,
    roughness_bias: f32,
    ao_scale: f32,
    ao_bias: f32,
    min_pom_layers: f32,
    max_pom_layers: f32,
    displacement_scale: f32,
    parallax_mapping_method: usize,
}

pub struct PbsMetallicRoughnessMaterial {
    albedo: Rc<Texture2D>,
    metallic_roughness_ao: Rc<Texture2D>,
    normals: Rc<Texture2D>,
    displacement: Option<Rc<Texture2D>>,
    ibl_brdf_lut: Texture2D,
    sampler: Sampler,
    property_block: MaterialPropertyBlock,
    program_pipeline: ProgramPipeline,
}

impl PbsMetallicRoughnessMaterial {
    pub fn new<P: AsRef<Path>>(
        asset_path: P,
        albedo: Rc<Texture2D>,
        metallic_roughness_ao: Rc<Texture2D>,
        normals: Rc<Texture2D>,
        displacement: Option<Rc<Texture2D>>,
    ) -> Self {
        let (vertex_shader, fragment_shader) = match displacement {
            Some(_) => (
                Shader::new(
                    ShaderStage::Vertex,
                    asset_path.as_ref().join("sdr/pbs_pom.vert.spv"),
                )
                .unwrap(),
                Shader::new(
                    ShaderStage::Fragment,
                    asset_path.as_ref().join("sdr/pbs_pom.frag.spv"),
                )
                .unwrap(),
            ),
            None => (
                Shader::new(
                    ShaderStage::Vertex,
                    asset_path.as_ref().join("sdr/pbs.vert.spv"),
                )
                .unwrap(),
                Shader::new(
                    ShaderStage::Fragment,
                    asset_path.as_ref().join("sdr/pbs.frag.spv"),
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

        let ibl_brdf_lut = Texture2D::load(
            asset_path.as_ref().join("textures/pbs/ibl_brdf_lut.png"),
            Some(Texture2DLoadConfig {
                is_srgb: false,
                generate_mipmap: false,
            }),
        )
        .expect("Failed to load BRDF LUT texture");

        Self {
            albedo,
            metallic_roughness_ao,
            normals,
            displacement,
            ibl_brdf_lut,
            sampler,
            property_block: MaterialPropertyBlock {
                base_color: Vec3::new(1.0, 1.0, 1.0),
                metallic_scale: 1.0,
                metallic_bias: 0.0,
                roughness_scale: 1.0,
                roughness_bias: 0.0,
                ao_scale: 1.0,
                ao_bias: 0.0,
                min_pom_layers: 8.0,
                max_pom_layers: 32.0,
                displacement_scale: 0.018,
                parallax_mapping_method: 4,
            },
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
                &self.property_block.base_color,
                ShaderStage::Fragment,
            )
            .set_vector3f(
                M_R_AO_SCALE_UNIFORM_NAME,
                &Vec3::new(
                    self.property_block.metallic_scale,
                    self.property_block.roughness_scale,
                    self.property_block.ao_scale,
                ),
                ShaderStage::Fragment,
            )
            .set_vector3f(
                M_R_AO_BIAS_UNIFORM_NAME,
                &Vec3::new(
                    self.property_block.metallic_bias,
                    self.property_block.roughness_bias,
                    self.property_block.ao_bias,
                ),
                ShaderStage::Fragment,
            );

        if let Some(displacement) = &self.displacement {
            self.program_pipeline
                .set_texture_2d(
                    DISPLACEMENT_MAP_UNIFORM_NAME,
                    &displacement,
                    &self.sampler,
                    ShaderStage::Fragment,
                )
                .set_vector3f(
                    POM_PARAMETERS_UNIFORM_NAME,
                    &Vec3::new(
                        self.property_block.min_pom_layers,
                        self.property_block.max_pom_layers,
                        self.property_block.displacement_scale,
                    ),
                    ShaderStage::Fragment,
                )
                .set_integer(
                    PARALLAX_MAPPING_METHOD_UNIFORM_NAME,
                    self.property_block.parallax_mapping_method as i32,
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

                    let mut albedo_color: [f32; 3] = self.property_block.base_color.into();
                    if imgui::ColorEdit::new(im_str!("Base Color"), &mut albedo_color)
                        .format(ColorFormat::Float)
                        .alpha(true)
                        .hdr(false)
                        .picker(true)
                        .build(&ui)
                    {
                        self.property_block.base_color = albedo_color.into()
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
                        .build(&ui, &mut self.property_block.metallic_scale);
                    imgui::Slider::new(im_str!("Metallic Bias"), RangeInclusive::new(-1.0, 1.0))
                        .display_format(im_str!("%.2f"))
                        .build(&ui, &mut self.property_block.metallic_bias);
                    ui.spacing();
                    imgui::Slider::new(im_str!("Roughness Scale"), RangeInclusive::new(0.0, 1.0))
                        .display_format(im_str!("%.2f"))
                        .build(&ui, &mut self.property_block.roughness_scale);
                    imgui::Slider::new(im_str!("Roughness Bias"), RangeInclusive::new(-1.0, 1.0))
                        .display_format(im_str!("%.2f"))
                        .build(&ui, &mut self.property_block.roughness_bias);
                    ui.spacing();
                    imgui::Slider::new(im_str!("AO Scale"), RangeInclusive::new(0.0, 1.0))
                        .display_format(im_str!("%.2f"))
                        .build(&ui, &mut self.property_block.ao_scale);
                    imgui::Slider::new(im_str!("AO Bias"), RangeInclusive::new(-1.0, 1.0))
                        .display_format(im_str!("%.2f"))
                        .build(&ui, &mut self.property_block.ao_bias);

                    ui.spacing();
                    ui.spacing();

                    ui.group(|| {
                        ui.text(im_str!("Normal Map"));
                        imgui::Image::new((self.normals.get_id() as usize).into(), [128.0, 128.0])
                            .build(&ui);
                        ui.spacing();
                    });
                });

                if let Some(displacement) = self.displacement.as_ref() {
                    ui.spacing();
                    ui.spacing();

                    ui.text(im_str!("Displacement Map"));
                    imgui::Image::new((displacement.get_id() as usize).into(), [128.0, 128.0])
                        .build(&ui);
                    ui.spacing();

                    imgui::TreeNode::new(im_str!("Parallax Mapping"))
                        .default_open(true)
                        .open_on_arrow(true)
                        .open_on_double_click(true)
                        .framed(false)
                        .tree_push_on_open(false)
                        .build(ui, || {
                            ui.spacing();
                            ui.group(|| {
                                imgui::ComboBox::new(im_str!("Method")).build_simple_string(
                                    ui,
                                    &mut self.property_block.parallax_mapping_method,
                                    &[
                                        im_str!("None"),
                                        im_str!("Parallax Mapping"),
                                        im_str!("Parallax Mapping + Offset Limiting"),
                                        im_str!("Steep Parallax Mapping"),
                                        im_str!("Parallax Occlusion Mapping"),
                                    ],
                                );

                                ui.drag_float(
                                    im_str!("Displacement Scale"),
                                    &mut self.property_block.displacement_scale,
                                )
                                .min(0.001)
                                .max(1.0)
                                .speed(0.001)
                                .display_format(im_str!("%.3f"))
                                .build();

                                if ui.is_item_hovered() {
                                    ui.tooltip_text(im_str!(
                                        "Drag left/right or double click to edit"
                                    ));
                                }

                                if self.property_block.parallax_mapping_method == 3
                                    || self.property_block.parallax_mapping_method == 4
                                {
                                    ui.drag_float_range2(
                                        im_str!("Min/Max Layers"),
                                        &mut self.property_block.min_pom_layers,
                                        &mut self.property_block.max_pom_layers,
                                    )
                                    .min(1.0)
                                    .max(256.0)
                                    .display_format(im_str!("%.0f"))
                                    .build();
                                }
                            });
                        });
                    ui.new_line();
                }
            });
        }
    }
}
