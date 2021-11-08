use std::any::Any;

use crate::{
    core::application::clear_default_framebuffer,
    framebuffer::Framebuffer,
    imgui::{Gui, Ui},
    math::Vec4,
    mesh::utilities::draw_full_screen_quad,
    rendering::{
        buffer::{Buffer, BufferStorageFlags, BufferTarget, MapModeFlags},
        postprocess::{AsAny, AsAnyMut, PostprocessingEffect, FULLSCREEN_VERTEX_SHADER},
        sampler::{Anisotropy, MagnificationFilter, MinificationFilter, Sampler, WrappingMode},
        shader::{Shader, ShaderBuilder, ShaderStage},
        state::StateManager,
    },
    Context,
};

const TONEMAPPER_FRAGMENT_SHADER_PATH: &str = "src/rendering/postprocess/shaders/tonemap.frag";

#[repr(C)]
struct ToneMappingPerFrameUniforms {
    operator: i32,
    white_threshold: f32,
    exposure: f32,
    _pad: f32,
}

pub struct ToneMapper {
    shader: Shader,
    tone_mapper_ubo: Buffer,
    sampler_nearest: Sampler,
    operator: usize,
    white_threshold: f32,
    exposure: f32,
    enabled: bool,
}

impl_as_any!(ToneMapper);

impl ToneMapper {
    pub fn new() -> Self {
        let shader = ShaderBuilder::new("ToneMapping Shader")
            .with_module(&FULLSCREEN_VERTEX_SHADER)
            .with_stage(ShaderStage::Fragment, TONEMAPPER_FRAGMENT_SHADER_PATH)
            .build();

        let mut tone_mapper_ubo = Buffer::new(
            "Tonemapping Fragment UBO",
            std::mem::size_of::<ToneMappingPerFrameUniforms>() as isize,
            BufferTarget::Uniform,
            BufferStorageFlags::MAP_WRITE_PERSISTENT_COHERENT,
        );
        tone_mapper_ubo.bind(3);
        tone_mapper_ubo.map(MapModeFlags::MAP_WRITE_PERSISTENT_COHERENT);

        let sampler_nearest = Sampler::new(
            MinificationFilter::Nearest,
            MagnificationFilter::Nearest,
            WrappingMode::ClampToEdge,
            WrappingMode::ClampToEdge,
            WrappingMode::ClampToEdge,
            Vec4::new(0.0, 0.0, 0.0, 0.0),
            Anisotropy::None,
        );

        ToneMapper {
            shader: shader,
            tone_mapper_ubo,
            sampler_nearest,
            operator: 0,
            white_threshold: 2.0,
            exposure: 1.5,
            enabled: true,
        }
    }

    pub fn set_exposure(&mut self, exposure: f32) {
        self.exposure = exposure
    }
}

impl PostprocessingEffect for ToneMapper {
    fn name(&self) -> &str {
        "ToneMapper"
    }

    fn enable(&mut self) {
        self.enabled = true;
    }

    fn disable(&mut self) {}

    fn enabled(&self) -> bool {
        self.enabled
    }

    fn apply(&mut self, input: &Framebuffer, context: Context) {
        let Context { window, .. } = context;

        let width = window.inner_size().width;
        let height = window.inner_size().height;
        clear_default_framebuffer(&Vec4::new(0.0, 1.0, 0.0, 1.0));

        StateManager::viewport(0, 0, width as i32, height as i32);

        self.shader.bind();

        let tone_mapping_uniforms = ToneMappingPerFrameUniforms {
            operator: self.operator as i32,
            white_threshold: self.white_threshold,
            exposure: self.exposure,
            _pad: 0.0,
        };

        self.tone_mapper_ubo.fill_mapped(0, &tone_mapping_uniforms);

        self.shader.set_texture_2d_with_id(
            0,
            input.texture_attachment(0).id(),
            &self.sampler_nearest,
        );

        draw_full_screen_quad();

        self.shader.unbind()
    }
}

impl Default for ToneMapper {
    fn default() -> Self {
        ToneMapper::new()
    }
}

impl Gui for ToneMapper {
    fn gui(&mut self, ui: &Ui) {
        imgui::TreeNode::new("Tone Mapping")
            .default_open(true)
            .open_on_arrow(true)
            .open_on_double_click(true)
            .build(ui, || {
                ui.spacing();
                ui.combo_simple_string(
                    "Operator",
                    &mut self.operator,
                    &[
                        "ACESFitted",
                        "ACESFilmic",
                        "Reinhard",
                        "Luma-Based Reinhard",
                        "White-Preserving Luma-Based Reinhard",
                        "Uncharted 2",
                        "RomBinDaHouse",
                    ],
                );

                if self.operator == 4 {
                    imgui::Slider::new("White Threshold", 0.3, 30.0)
                        .display_format("%.2f")
                        .build(ui, &mut self.white_threshold);
                }

                ui.new_line()
            });
    }
}
