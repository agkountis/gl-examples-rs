use crate::{
    core::application::clear_default_framebuffer,
    framebuffer::Framebuffer,
    imgui::{im_str, Gui, Ui},
    math::Vec4,
    mesh::FULLSCREEN_MESH,
    rendering::{
        buffer::{Buffer, BufferStorageFlags, BufferTarget, MapModeFlags},
        postprocess::{AsAny, AsAnyMut, PostprocessingEffect, FULLSCREEN_VERTEX_SHADER},
        program_pipeline::ProgramPipeline,
        sampler::{Anisotropy, MagnificationFilter, MinificationFilter, Sampler, WrappingMode},
        shader::{Shader, ShaderStage},
        state::{FrontFace, StateManager},
        Draw,
    },
    Context,
};

use std::{any::Any, ops::RangeInclusive};

#[repr(C)]
struct ToneMappingPerFrameUniforms {
    operator: i32,
    white_threshold: f32,
    exposure: f32,
    _pad: f32,
}

pub struct ToneMapper {
    pipeline: ProgramPipeline,
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
        let pipeline = ProgramPipeline::new()
            .add_shader(&FULLSCREEN_VERTEX_SHADER)
            .add_shader(
                &Shader::new(
                    ShaderStage::Fragment,
                    "src/rendering/postprocess/shaders/tonemap.frag",
                )
                .unwrap(),
            )
            .build()
            .unwrap();

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
            pipeline,
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

        StateManager::set_viewport(0, 0, width as i32, height as i32);

        self.pipeline.bind();

        let tone_mapping_uniforms = ToneMappingPerFrameUniforms {
            operator: self.operator as i32,
            white_threshold: self.white_threshold,
            exposure: self.exposure,
            _pad: 0.0,
        };

        self.tone_mapper_ubo.fill_mapped(0, &tone_mapping_uniforms);

        self.pipeline.set_texture_2d_with_id(
            0,
            input.texture_attachment(0).id(),
            &self.sampler_nearest,
        );

        StateManager::set_front_face(FrontFace::Clockwise);
        FULLSCREEN_MESH.draw();
        StateManager::set_front_face(FrontFace::CounterClockwise);

        self.pipeline.unbind()
    }
}

impl Default for ToneMapper {
    fn default() -> Self {
        ToneMapper::new()
    }
}

impl Gui for ToneMapper {
    fn gui(&mut self, ui: &Ui) {
        imgui::TreeNode::new(im_str!("Tone Mapping"))
            .default_open(true)
            .open_on_arrow(true)
            .open_on_double_click(true)
            .build(ui, || {
                ui.spacing();
                imgui::ComboBox::new(im_str!("Operator")).build_simple_string(
                    &ui,
                    &mut self.operator,
                    &[
                        im_str!("ACESFitted"),
                        im_str!("ACESFilmic"),
                        im_str!("Reinhard"),
                        im_str!("Luma-Based Reinhard"),
                        im_str!("White-Preserving Luma-Based Reinhard"),
                        im_str!("Uncharted 2"),
                        im_str!("RomBinDaHouse"),
                    ],
                );

                if self.operator == 4 {
                    imgui::Slider::new(im_str!("White Threshold"))
                        .range(RangeInclusive::new(0.3, 30.0))
                        .display_format(im_str!("%.2f"))
                        .build(&ui, &mut self.white_threshold);
                }

                ui.new_line()
            });
    }
}
