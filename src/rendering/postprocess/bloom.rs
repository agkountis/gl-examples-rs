use crate::core::math::{UVec2, Vec3, Vec4};
use crate::imgui::{im_str, Gui, Ui};
use crate::rendering::buffer::{Buffer, BufferStorageFlags, BufferTarget, MapModeFlags};
use crate::rendering::framebuffer::{Framebuffer, TemporaryFramebufferPool};
use crate::rendering::postprocess::{
    AsAny, AsAnyMut, PostprocessingEffect, FULLSCREEN_VERTEX_SHADER,
};
use crate::rendering::program_pipeline::ProgramPipeline;
use crate::rendering::shader::{Shader, ShaderStage};
use crate::Context;
use std::any::Any;
use std::ops::RangeInclusive;
use std::path::{Path, PathBuf};
use std::rc::Rc;

const UBO_BINDING_INDEX: u32 = 7;
const EPSILON: f32 = 0.00001;
const MIN_ITERATIONS: u32 = 1;
const MAX_ITERATIONS: u32 = 16;
const MIN_THRESHOLD: f32 = 0.1;
const MAX_THRESHOLD: f32 = 10.0;
const MIN_SMOOTH_FADE: f32 = 0.1;
const MAX_SMOOTH_FADE: f32 = 1.0;
const MIN_INTENSITY: f32 = 0.1;
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
    v_blur_program_pipeline: ProgramPipeline,
    h_blur_program_pipeline: ProgramPipeline,
    ubo_data: BloomUboData,
    ubo: Buffer,
    enabled: bool,
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

        assert_eq!(
            attachment.is_depth_stencil(),
            false,
            "Bloom effect do not support depth texture attachments."
        );

        let knee = self.threshold * self.smooth_fade;
        self.ubo_data.filter = Vec4::new(
            self.threshold,
            self.threshold - knee,
            2.0 * knee,
            0.25 / (knee + EPSILON),
        );

        self.ubo.fill_mapped(0, &self.ubo_data);

        // Blit to half resolution and filter bright pixels.
        let mut size = UVec2::new(input.size().x / 2, input.size().y / 2);
        let format = attachment.format();

        let mut current_destination = framebuffer_cache.get_temporary(size, format, None);

        let mut temporaries = Vec::with_capacity(self.iterations as usize);

        temporaries.push(Rc::clone(&current_destination));

        // TODO: Do a Box downsample blit here and filter brights

        let mut current_source = Rc::clone(&current_destination);
        for _ in 1..self.iterations {
            size.x /= 2;
            size.y /= 2;

            if size.y < 2 {
                break;
            }

            current_destination = framebuffer_cache.get_temporary(size, format, None);
            temporaries.push(Rc::clone(&current_destination));

            //TODO: Do a downsample blit here
            current_source = Rc::clone(&current_destination);
        }

        temporaries.into_iter().rev().skip(1).for_each(|temporary| {
            current_destination = temporary;
            //TODO: Do an upsampling blit here
            current_source = Rc::clone(&current_destination);
        });

        //TODO: blit to add bloom tex to input render target
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
                    imgui::Slider::new(im_str!("Iterations"))
                        .range(RangeInclusive::new(MIN_ITERATIONS, MAX_ITERATIONS))
                        .build(&ui, &mut self.iterations);
                    imgui::Slider::new(im_str!("Threshold"))
                        .range(RangeInclusive::new(MIN_THRESHOLD, MAX_THRESHOLD))
                        .display_format(im_str!("%.2f"))
                        .build(&ui, &mut self.threshold);
                    imgui::Slider::new(im_str!("Smooth Fade"))
                        .range(RangeInclusive::new(MIN_SMOOTH_FADE, MAX_SMOOTH_FADE))
                        .display_format(im_str!("%.2f"))
                        .build(&ui, &mut self.smooth_fade);
                    imgui::Slider::new(im_str!("Intensity"))
                        .range(RangeInclusive::new(MIN_INTENSITY, MAX_INTENSITY))
                        .display_format(im_str!("%.2f"))
                        .build(&ui, &mut self.intensity);
                    ui.unindent()
                });
        });
    }
}

pub struct BloomBuilder {
    assets_path: PathBuf,
    iterations: u32,
    threshold: f32,
    smooth_fade: f32,
    intensity: f32,
    enabled: bool,
}

impl BloomBuilder {
    pub fn new<P: AsRef<Path>>(assets_path: P) -> Self {
        Self {
            assets_path: PathBuf::from(assets_path.as_ref()),
            iterations: 5,
            threshold: 1.0,
            smooth_fade: 0.5,
            intensity: 1.0,
            enabled: true,
        }
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
        let (v_blur_program_pipeline, h_blur_program_pipeline) = {
            let v_blur_fs = Shader::new(
                ShaderStage::Fragment,
                self.assets_path.join("sdr/gaussian_blur_vertical.frag"),
            )
            .unwrap();

            let h_blur_fs = Shader::new(
                ShaderStage::Fragment,
                self.assets_path.join("sdr/gaussian_blur_horizontal.frag"),
            )
            .unwrap();

            let v_blur_pipeline = ProgramPipeline::new()
                .add_shader(&FULLSCREEN_VERTEX_SHADER)
                .add_shader(&v_blur_fs)
                .build()
                .unwrap();

            let h_blur_pipeline = ProgramPipeline::new()
                .add_shader(&FULLSCREEN_VERTEX_SHADER)
                .add_shader(&h_blur_fs)
                .build()
                .unwrap();

            (v_blur_pipeline, h_blur_pipeline)
        };

        let mut ubo = Buffer::new(
            "Bloom UBO",
            std::mem::size_of::<BloomUboData>() as isize,
            BufferTarget::Uniform,
            BufferStorageFlags::MAP_WRITE_PERSISTENT_COHERENT,
        );
        ubo.bind(UBO_BINDING_INDEX);
        ubo.map(MapModeFlags::MAP_WRITE_PERSISTENT_COHERENT);

        Bloom {
            iterations: self.iterations,
            threshold: self.threshold,
            smooth_fade: self.smooth_fade,
            intensity: self.intensity,
            v_blur_program_pipeline,
            h_blur_program_pipeline,
            ubo_data: Default::default(),
            ubo,
            enabled: self.enabled,
        }
    }
}
