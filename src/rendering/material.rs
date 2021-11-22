use std::borrow::BorrowMut;
use std::{path::Path, rc::Rc};

use crevice::std140::AsStd140;

use crate::core::asset::Asset;
use crate::rendering::buffer::{Buffer, BufferStorageFlags, BufferTarget, MapModeFlags};
use crate::rendering::shader::ShaderCreateInfo;
use crate::rendering::texture::Texture2DLoadConfig;
use crate::sampler::Anisotropy;
use crate::{
    core::math::Vec4,
    imgui::{ColorFormat, Gui, Ui},
    rendering::{
        sampler::{MagnificationFilter, MinificationFilter, Sampler, WrappingMode},
        shader::{Shader, ShaderStage},
        texture::Texture2D,
    },
    Context,
};

const MATERIAL_UBO_BINDING_INDEX: u32 = 4;
const ALBEDO_MAP_BINDING_INDEX: u32 = 0;
const NORMAL_MAP_BINDING_INDEX: u32 = 1;
// [Metalness (R), Roughness (G), AO (B)]
const M_R_AO_MAP_BINDING_INDEX: u32 = 2;
const BRDF_LUT_MAP_BINDING_INDEX: u32 = 3;
const DISPLACEMENT_MAP_BINDING_INDEX: u32 = 6;

pub trait Material: Gui {
    fn bind(&self);
    fn unbind(&self);
    fn shader(&self) -> Rc<Shader>;
}

#[repr(C)]
#[derive(Debug, Clone, Copy, AsStd140)]
struct MaterialPropertyBlock {
    base_color: mint::Vector4<f32>,
    metallic_scale: f32,
    metallic_bias: f32,
    roughness_scale: f32,
    roughness_bias: f32,
    ao_scale: f32,
    ao_bias: f32,
    reflectance: f32,
    min_pom_layers: f32,
    max_pom_layers: f32,
    displacement_scale: f32,
    parallax_mapping_method: i32,
}

pub struct PbsMetallicRoughnessMaterial {
    albedo: Rc<Texture2D>,
    metallic_roughness_ao: Rc<Texture2D>,
    normals: Rc<Texture2D>,
    displacement: Option<Rc<Texture2D>>,
    ibl_brdf_lut: Texture2D,
    sampler: Sampler,
    property_block: MaterialPropertyBlock,
    shader: Rc<Shader>,
    material_ubo: Buffer,
    parallax_mapping_method: usize,
}

impl PbsMetallicRoughnessMaterial {
    pub fn new<P: AsRef<Path>>(
        context: Context,
        asset_path: P,
        albedo: Rc<Texture2D>,
        metallic_roughness_ao: Rc<Texture2D>,
        normals: Rc<Texture2D>,
        displacement: Option<Rc<Texture2D>>,
    ) -> Self {
        let Context { device, .. } = context;

        let create_info = ShaderCreateInfo::builder("PBS Shader")
            .stage(
                ShaderStage::Vertex,
                asset_path.as_ref().join("shaders/pbs.vert"),
            )
            .stage(
                ShaderStage::Fragment,
                asset_path.as_ref().join("shaders/pbs.frag"),
            )
            .keyword_set(&["_", "FEATURE_PARALLAX_MAPPING"])
            // .keyword_set(&["_", "FEATURE_SPECULAR_AA"])
            // .keyword_set(&["_", "FEATURE_SPECULAR_AO"])
            // .keyword_set(&["FEATURE_BRDF_FILLAMENT", "FEATURE_BRDF_UE4"])
            .build();

        let mut shader = device.shader_manager().create_shader(&create_info);

        if displacement.is_some() {
            shader.enable_keyword("FEATURE_PARALLAX_MAPPING")
        }

        let sampler = Sampler::new(
            MinificationFilter::LinearMipmapLinear,
            MagnificationFilter::Linear,
            WrappingMode::Repeat,
            WrappingMode::Repeat,
            WrappingMode::Repeat,
            Vec4::new(0.0, 0.0, 0.0, 0.0),
            Anisotropy::X4,
        );

        let ibl_brdf_lut = Texture2D::load(
            asset_path.as_ref().join("textures/pbs/ibl_brdf_lut.png"),
            Some(Texture2DLoadConfig {
                is_srgb: false,
                generate_mipmap: false,
            }),
        )
        .expect("Failed to load BRDF LUT texture");

        let mut material_ubo = Buffer::new(
            "MaterialPropertyBlock UBO",
            std::mem::size_of::<<MaterialPropertyBlock as AsStd140>::Std140Type>() as isize,
            BufferTarget::Uniform,
            BufferStorageFlags::MAP_WRITE_PERSISTENT_COHERENT,
        );
        material_ubo.bind(MATERIAL_UBO_BINDING_INDEX);
        material_ubo.map(MapModeFlags::MAP_WRITE_PERSISTENT_COHERENT);

        Self {
            albedo,
            metallic_roughness_ao,
            normals,
            displacement,
            ibl_brdf_lut,
            sampler,
            property_block: MaterialPropertyBlock {
                base_color: [1.0, 1.0, 1.0, 1.0].into(),
                metallic_scale: 1.0,
                metallic_bias: 0.0,
                roughness_scale: 1.0,
                roughness_bias: 0.0,
                ao_scale: 1.0,
                ao_bias: 0.0,
                reflectance: 0.5,
                min_pom_layers: 8.0,
                max_pom_layers: 32.0,
                displacement_scale: 0.018,
                parallax_mapping_method: 0,
            },
            shader,
            material_ubo,
            parallax_mapping_method: 4,
        }
    }
}

impl Material for PbsMetallicRoughnessMaterial {
    fn bind(&self) {
        self.shader.bind();

        self.material_ubo
            .fill_mapped(0, &self.property_block.as_std140());

        self.shader
            .set_texture_2d(ALBEDO_MAP_BINDING_INDEX, &self.albedo, &self.sampler)
            .set_texture_2d(
                M_R_AO_MAP_BINDING_INDEX,
                &self.metallic_roughness_ao,
                &self.sampler,
            )
            .set_texture_2d(NORMAL_MAP_BINDING_INDEX, &self.normals, &self.sampler)
            .set_texture_2d(
                BRDF_LUT_MAP_BINDING_INDEX,
                &self.ibl_brdf_lut,
                &self.sampler,
            );

        if let Some(displacement) = &self.displacement {
            self.shader
                .set_texture_2d(DISPLACEMENT_MAP_BINDING_INDEX, displacement, &self.sampler);
        }
    }

    fn unbind(&self) {
        self.shader.unbind();
    }

    fn shader(&self) -> Rc<Shader> {
        Rc::clone(&self.shader)
    }
}

impl Gui for PbsMetallicRoughnessMaterial {
    fn gui(&mut self, ui: &Ui) {
        if imgui::CollapsingHeader::new("Material")
            .default_open(true)
            .open_on_arrow(true)
            .open_on_double_click(true)
            .build(ui)
        {
            ui.spacing();
            ui.group(|| {
                ui.group(|| {
                    ui.text("Albedo Map");
                    imgui::Image::new((self.albedo.get_id() as usize).into(), [128.0, 128.0])
                        .build(ui);
                    ui.spacing();

                    let mut albedo_color: [f32; 4] = self.property_block.base_color.into();
                    if imgui::ColorEdit::new("Base Color", &mut albedo_color)
                        .format(ColorFormat::Float)
                        .alpha(true)
                        .hdr(true)
                        .picker(true)
                        .build(ui)
                    {
                        self.property_block.base_color = albedo_color.into()
                    }
                });
                ui.spacing();
                ui.spacing();
                ui.group(|| {
                    ui.text("Metallic/Roughness/Ao Map");
                    imgui::Image::new(
                        (self.metallic_roughness_ao.get_id() as usize).into(),
                        [128.0, 128.0],
                    )
                    .build(ui);
                    ui.spacing();
                    imgui::Slider::new("Metallic Scale", 0.0, 1.0)
                        .display_format("%.2f")
                        .build(ui, &mut self.property_block.metallic_scale);
                    imgui::Slider::new("Metallic Bias", -1.0, 1.0)
                        .display_format("%.2f")
                        .build(ui, &mut self.property_block.metallic_bias);
                    ui.spacing();
                    imgui::Slider::new("Roughness Scale", 0.0, 1.0)
                        .display_format("%.2f")
                        .build(ui, &mut self.property_block.roughness_scale);
                    imgui::Slider::new("Roughness Bias", -1.0, 1.0)
                        .display_format("%.2f")
                        .build(ui, &mut self.property_block.roughness_bias);
                    ui.spacing();
                    imgui::Slider::new("AO Scale", 0.0, 1.0)
                        .display_format("%.2f")
                        .build(ui, &mut self.property_block.ao_scale);
                    imgui::Slider::new("AO Bias", -1.0, 1.0)
                        .display_format("%.2f")
                        .build(ui, &mut self.property_block.ao_bias);
                    imgui::Slider::new("Reflectance", 0.0, 1.0)
                        .display_format("%.1f")
                        .build(ui, &mut self.property_block.reflectance);

                    ui.spacing();
                    ui.spacing();

                    ui.group(|| {
                        ui.text("Normal Map");
                        imgui::Image::new((self.normals.get_id() as usize).into(), [128.0, 128.0])
                            .build(ui);
                        ui.spacing();
                    });
                });

                if let Some(displacement) = self.displacement.as_ref() {
                    ui.spacing();
                    ui.spacing();

                    ui.text("Displacement Map");
                    imgui::Image::new((displacement.get_id() as usize).into(), [128.0, 128.0])
                        .build(ui);
                    ui.spacing();

                    imgui::TreeNode::new("Parallax Mapping")
                        .default_open(true)
                        .open_on_arrow(true)
                        .open_on_double_click(true)
                        .framed(false)
                        .tree_push_on_open(false)
                        .build(ui, || {
                            ui.spacing();
                            ui.group(|| {
                                ui.combo_simple_string(
                                    "Method",
                                    &mut self.parallax_mapping_method,
                                    &[
                                        "None",
                                        "Parallax Mapping",
                                        "Parallax Mapping + Offset Limiting",
                                        "Steep Parallax Mapping",
                                        "Parallax Occlusion Mapping",
                                    ],
                                );

                                self.property_block.parallax_mapping_method =
                                    self.parallax_mapping_method as i32;

                                imgui::Drag::new("Displacement Scale")
                                    .range(0.001, 1.0)
                                    .speed(0.001)
                                    .display_format("%.3f")
                                    .build(ui, &mut self.property_block.displacement_scale);

                                if ui.is_item_hovered() {
                                    ui.tooltip_text("Drag left/right or double click to edit");
                                }

                                if self.property_block.parallax_mapping_method == 3
                                    || self.property_block.parallax_mapping_method == 4
                                {
                                    imgui::DragRange::new("Min/Max Layers")
                                        .range(1.0, 256.0)
                                        .display_format("%.0f")
                                        .build(
                                            ui,
                                            &mut self.property_block.min_pom_layers,
                                            &mut self.property_block.max_pom_layers,
                                        );
                                }
                            });
                        });
                    ui.new_line();
                }
            });
        }
    }
}
