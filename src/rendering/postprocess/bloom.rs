use std::any::Any;
use std::ops::RangeInclusive;
use std::rc::Rc;

use crate::color::srgb_to_linear;
use crate::core::math::{UVec2, Vec3, Vec4};
use crate::imgui::{im_str, Condition, Gui, Ui};
use crate::rendering::sampler::{Anisotropy, MinificationFilter, Sampler, WrappingMode};
use crate::rendering::state::{BlendFactor, FrontFace, StateManager};
use crate::rendering::{
    buffer::{Buffer, BufferStorageFlags, BufferTarget, MapModeFlags},
    framebuffer::Framebuffer,
    mesh::FULLSCREEN_MESH,
    postprocess::{AsAny, AsAnyMut, PostprocessingEffect, FULLSCREEN_VERTEX_SHADER},
    program_pipeline::ProgramPipeline,
    shader::{Shader, ShaderStage},
};
use crate::sampler::MagnificationFilter;
use crate::{Context, Draw};
use imgui::TextureId;

const UBO_BINDING_INDEX: u32 = 7;
const EPSILON: f32 = 0.00001;
const MIN_ITERATIONS: u32 = 1;
const MAX_ITERATIONS: u32 = 16;
const MIN_THRESHOLD: f32 = 0.1;
const MAX_THRESHOLD: f32 = 10.0;
const MIN_SMOOTH_FADE: f32 = 0.01;
const MAX_SMOOTH_FADE: f32 = 1.0;
const MIN_INTENSITY: f32 = 0.01;
const MAX_INTENSITY: f32 = 10.0;

#[repr(C)]
#[derive(Default, Debug)]
struct BloomUboData {
    filter: Vec4,
    intensity: f32,
    _pad: Vec3,
}

pub struct Bloom {
    iterations: u32,
    threshold: f32,
    smooth_fade: f32,
    intensity: f32,
    downsample_program_pipeline: ProgramPipeline,
    downsample_prefilter_program_pipeline: ProgramPipeline,
    upsample_program_pipeline: ProgramPipeline,
    bloom_upsample_apply_program_pipeline: ProgramPipeline,
    ubo_data: BloomUboData,
    ubo: Buffer,
    linear_sampler: Sampler,
    blit_framebuffers: Vec<Rc<Framebuffer>>,
    enabled: bool,
    show_debug_window: bool,
}

impl_as_any!(Bloom);

impl PostprocessingEffect for Bloom {
    fn name(&self) -> &str {
        "bloom"
    }

    fn enable(&mut self) {
        self.enabled = true
    }

    fn disable(&mut self) {
        self.enabled = false
    }

    fn enabled(&self) -> bool {
        self.enabled
    }

    fn apply(&mut self, input: &Framebuffer, context: Context) {
        let Context {
            framebuffer_cache, ..
        } = context;
        //TODO: Implement me

        let attachment = input.texture_attachment(0);

        assert!(
            !attachment.is_depth_stencil(),
            "Bloom effect do not support depth texture attachments."
        );

        let threshold = srgb_to_linear(self.threshold);
        let knee = threshold * self.smooth_fade;
        self.ubo_data.filter = Vec4::new(
            threshold,
            threshold - knee,
            2.0 * knee,
            0.25 / (knee + EPSILON),
        );
        self.ubo_data.intensity = self.intensity;

        self.ubo.fill_mapped(0, &self.ubo_data);

        // Blit to half resolution and filter bright pixels.
        let mut size = UVec2::new(input.size().x / 2, input.size().y / 2);
        let format = attachment.format();

        self.blit_framebuffers
            .iter()
            .for_each(|fb| framebuffer_cache.release_temporary(Rc::clone(fb)));
        self.blit_framebuffers.clear();

        let mut current_destination = framebuffer_cache.get_temporary(size, format, None);

        //self.blit_framebuffers = Vec::with_capacity(self.iterations as usize);

        self.blit_framebuffers.push(Rc::clone(&current_destination));

        // TODO: Do a Box downsample blit here and filter brights

        current_destination.bind();
        current_destination.clear(&Vec4::new(0.5, 0.5, 0.5, 1.0));

        self.downsample_prefilter_program_pipeline.bind();
        self.downsample_prefilter_program_pipeline
            .set_texture_2d_with_id(0, input.texture_attachments()[0].id(), &self.linear_sampler);

        StateManager::set_front_face(FrontFace::Clockwise);
        FULLSCREEN_MESH.draw();
        StateManager::set_front_face(FrontFace::CounterClockwise);

        current_destination.unbind(false);
        self.downsample_prefilter_program_pipeline.unbind();

        let mut current_source = Rc::clone(&current_destination);

        for _ in 1..self.iterations {
            size.x /= 2;
            size.y /= 2;

            if size.y < 2 {
                break;
            }

            current_destination = framebuffer_cache.get_temporary(size, format, None);

            self.blit_framebuffers.push(Rc::clone(&current_destination));
            //TODO: Do a downsample blit here

            current_destination.bind();
            current_destination.clear(&Vec4::new(0.5, 0.5, 0.5, 1.0));

            self.downsample_program_pipeline.bind();
            self.downsample_program_pipeline.set_texture_2d_with_id(
                0,
                current_source.texture_attachments()[0].id(),
                &self.linear_sampler,
            );

            StateManager::set_front_face(FrontFace::Clockwise);
            FULLSCREEN_MESH.draw();
            StateManager::set_front_face(FrontFace::CounterClockwise);

            current_destination.unbind(false);
            self.downsample_program_pipeline.unbind();

            current_source = Rc::clone(&current_destination);
        }

        self.downsample_program_pipeline.unbind();

        self.blit_framebuffers
            .iter()
            .rev()
            .skip(1)
            .for_each(|temporary| {
                current_destination = Rc::clone(temporary);
                //TODO: Do an upsampling blit here
                current_destination.bind();

                self.upsample_program_pipeline.bind();
                self.upsample_program_pipeline.set_texture_2d_with_id(
                    0,
                    current_source.texture_attachments()[0].id(),
                    &self.linear_sampler,
                );

                StateManager::enable_blending();
                StateManager::set_blend_function(BlendFactor::One, BlendFactor::One);

                StateManager::set_front_face(FrontFace::Clockwise);
                FULLSCREEN_MESH.draw();
                StateManager::set_front_face(FrontFace::CounterClockwise);

                StateManager::disable_blending();

                self.upsample_program_pipeline.unbind();
                current_destination.unbind(false);

                current_source = Rc::clone(&current_destination);
            });

        //TODO: blit to add bloom tex to input render target

        self.bloom_upsample_apply_program_pipeline.bind();
        self.bloom_upsample_apply_program_pipeline
            .set_texture_2d_with_id(
                0,
                current_source.texture_attachments()[0].id(),
                &self.linear_sampler,
            );
        self.bloom_upsample_apply_program_pipeline
            .set_texture_2d_with_id(1, input.texture_attachments()[0].id(), &self.linear_sampler);
        input.bind();

        StateManager::set_front_face(FrontFace::Clockwise);
        FULLSCREEN_MESH.draw();
        StateManager::set_front_face(FrontFace::CounterClockwise);

        input.unbind(false);
        self.bloom_upsample_apply_program_pipeline.unbind();
    }
}

impl Gui for Bloom {
    fn gui(&mut self, ui: &Ui) {
        ui.group(|| {
            ui.checkbox(im_str!("##bloom"), &mut self.enabled);
            ui.same_line(20.0);
            imgui::TreeNode::new(im_str!("Bloom"))
                .default_open(true)
                .open_on_arrow(true)
                .open_on_double_click(true)
                .framed(false)
                .build(ui, || {
                    ui.indent();

                    ui.checkbox(im_str!("Show debug window"), &mut self.show_debug_window);

                    if self.show_debug_window {
                        imgui::Window::new(im_str!("Bloom Debug"))
                            .focus_on_appearing(true)
                            .bring_to_front_on_focus(true)
                            .size([256.0f32, 500.0f32], Condition::Appearing)
                            .build(ui, || {
                                self.blit_framebuffers.iter().for_each(|fb| {
                                    let tex_id =
                                        fb.texture_attachments().iter().next().unwrap().id();
                                    ui.text(format!("Framebuffer ID: {}", fb.id()));
                                    ui.indent();

                                    ui.text(format!("Texture ID: {}", tex_id));

                                    let dimensions =
                                        format!("Dimensions: {}x{}", fb.size().x, fb.size().y);
                                    ui.text(dimensions);

                                    imgui::Image::new(
                                        TextureId::new(tex_id as usize),
                                        [fb.size().x as f32, fb.size().y as f32],
                                    )
                                    .uv0([0.0, 1.0])
                                    .uv1([1.0, 0.0])
                                    .build(ui);

                                    ui.unindent()
                                });
                            });
                    }

                    if imgui::Slider::new(im_str!("Iterations"))
                        .range(RangeInclusive::new(MIN_ITERATIONS, MAX_ITERATIONS))
                        .build(ui, &mut self.iterations)
                    {
                        self.blit_framebuffers = Vec::with_capacity(self.iterations as usize);
                    }

                    imgui::Slider::new(im_str!("Threshold"))
                        .range(RangeInclusive::new(MIN_THRESHOLD, MAX_THRESHOLD))
                        .display_format(im_str!("%.2f"))
                        .build(ui, &mut self.threshold);

                    imgui::Slider::new(im_str!("Smooth Fade"))
                        .range(RangeInclusive::new(MIN_SMOOTH_FADE, MAX_SMOOTH_FADE))
                        .display_format(im_str!("%.2f"))
                        .build(ui, &mut self.smooth_fade);

                    imgui::Slider::new(im_str!("Intensity"))
                        .range(RangeInclusive::new(MIN_INTENSITY, MAX_INTENSITY))
                        .display_format(im_str!("%.2f"))
                        .build(ui, &mut self.intensity);

                    ui.unindent()
                });
        });
    }
}

pub struct BloomBuilder {
    iterations: u32,
    threshold: f32,
    smooth_fade: f32,
    intensity: f32,
    enabled: bool,
}

impl Default for BloomBuilder {
    fn default() -> Self {
        Self {
            iterations: 8,
            threshold: 1.0,
            smooth_fade: 0.54,
            intensity: 0.1,
            enabled: true,
        }
    }
}

impl BloomBuilder {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn iterations(mut self, iterations: u32) -> Self {
        self.iterations = iterations;
        self
    }

    pub fn threshold(mut self, threshold: f32) -> Self {
        self.threshold = threshold;
        self
    }

    pub fn smooth_fade(mut self, smooth_fade: f32) -> Self {
        self.smooth_fade = smooth_fade;
        self
    }

    pub fn intensity(mut self, intensity: f32) -> Self {
        self.intensity = intensity;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn build(self) -> Bloom {
        let (
            downsample_program_pipeline,
            downsample_prefilter_program_pipeline,
            upsample_program_pipeline,
            bloom_upsample_apply_program_pipeline,
        ) = {
            let downsample_fs = Shader::new(
                ShaderStage::Fragment,
                "src/rendering/postprocess/shaders/bloom_dual_filtering_blur_downsample.frag",
            )
            .unwrap();

            let downsample_prefilter_fs = Shader::new(
                ShaderStage::Fragment,
                "src/rendering/postprocess/shaders/bloom_dual_filtering_blur_downsample_prefilter.frag",
            ).unwrap();

            let upsample_blur_fs = Shader::new(
                ShaderStage::Fragment,
                "src/rendering/postprocess/shaders/bloom_dual_filtering_blur_upsample.frag",
            )
            .unwrap();

            let bloom_apply_fs = Shader::new(
                ShaderStage::Fragment,
                "src/rendering/postprocess/shaders/bloom_dual_filtering_apply.frag",
            )
            .unwrap();

            let downsample_pipeline = ProgramPipeline::new()
                .add_shader(&FULLSCREEN_VERTEX_SHADER)
                .add_shader(&downsample_fs)
                .build()
                .unwrap();

            let downsample_prefilter_pipeline = ProgramPipeline::new()
                .add_shader(&FULLSCREEN_VERTEX_SHADER)
                .add_shader(&downsample_prefilter_fs)
                .build()
                .unwrap();

            let upsample_pipeline = ProgramPipeline::new()
                .add_shader(&FULLSCREEN_VERTEX_SHADER)
                .add_shader(&upsample_blur_fs)
                .build()
                .unwrap();

            let bloom_apply_pipeline = ProgramPipeline::new()
                .add_shader(&FULLSCREEN_VERTEX_SHADER)
                .add_shader(&bloom_apply_fs)
                .build()
                .unwrap();

            (
                downsample_pipeline,
                downsample_prefilter_pipeline,
                upsample_pipeline,
                bloom_apply_pipeline,
            )
        };

        let mut ubo = Buffer::new(
            "Bloom UBO",
            std::mem::size_of::<BloomUboData>() as isize,
            BufferTarget::Uniform,
            BufferStorageFlags::MAP_WRITE_PERSISTENT_COHERENT,
        );
        ubo.bind(UBO_BINDING_INDEX);
        ubo.map(MapModeFlags::MAP_WRITE_PERSISTENT_COHERENT);

        let linear_sampler = Sampler::new(
            MinificationFilter::Linear,
            MagnificationFilter::Linear,
            WrappingMode::ClampToEdge,
            WrappingMode::ClampToEdge,
            WrappingMode::ClampToEdge,
            Vec4::new(0.0, 0.0, 0.0, 1.0),
            Anisotropy::None,
        );

        Bloom {
            iterations: self.iterations,
            threshold: self.threshold,
            smooth_fade: self.smooth_fade,
            intensity: self.intensity,
            downsample_program_pipeline,
            downsample_prefilter_program_pipeline,
            upsample_program_pipeline,
            bloom_upsample_apply_program_pipeline,
            ubo_data: Default::default(),
            ubo,
            linear_sampler,
            blit_framebuffers: Vec::with_capacity(self.iterations as usize),
            enabled: self.enabled,
            show_debug_window: false,
        }
    }
}
