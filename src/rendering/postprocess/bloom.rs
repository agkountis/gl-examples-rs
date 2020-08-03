use crate::core::math::UVec2;
use crate::imgui::{im_str, Gui, Ui};
use crate::rendering::framebuffer::{Framebuffer, TemporaryFramebufferPool};
use crate::rendering::postprocess::{AsAny, AsAnyMut, PostprocessingEffect};
use crate::rendering::program_pipeline::ProgramPipeline;
use crate::rendering::shader::{Shader, ShaderStage};
use crate::rendering::texture::SizedTextureFormat;
use std::any::Any;
use std::ops::RangeInclusive;
use std::path::{Path, PathBuf};

pub struct Bloom {
    iterations: u32,
    threshold: f32,
    smooth_fade: f32,
    intensity: f32,
    downsample_framebuffers: Vec<Framebuffer>,
    upsample_framebuffers: Vec<Framebuffer>,
    v_blur_program_pipeline: ProgramPipeline,
    h_blur_program_pipeline: ProgramPipeline,
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

    fn apply(&self, input: &Framebuffer, framebuffer_cache: &mut TemporaryFramebufferPool) {
        let tmp1 =
            framebuffer_cache.get_temporary(UVec2::new(64, 64), SizedTextureFormat::Rgba16f, None);

        unsafe {
            if FOO {
                let tmp2 = framebuffer_cache.get_temporary(
                    UVec2::new(128, 128),
                    SizedTextureFormat::Rgba16f,
                    Some(SizedTextureFormat::Depth24),
                );

                unsafe {
                    FOO = false;
                }
            }
        }
    }
}
static mut FOO: bool = true;
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
                .build(&ui, || {
                    ui.indent();
                    imgui::Slider::new(im_str!("Iterations"), RangeInclusive::new(1, 16))
                        .build(&ui, &mut self.iterations);
                    imgui::Slider::new(im_str!("Threshold"), RangeInclusive::new(0.1, 10.0))
                        .display_format(im_str!("%.2f"))
                        .build(&ui, &mut self.threshold);
                    imgui::Slider::new(im_str!("Smooth Fade"), RangeInclusive::new(0.1, 1.0))
                        .display_format(im_str!("%.2f"))
                        .build(&ui, &mut self.smooth_fade);
                    imgui::Slider::new(im_str!("Intensity"), RangeInclusive::new(0.1, 10.0))
                        .display_format(im_str!("%.2f"))
                        .build(&ui, &mut self.intensity);
                    ui.unindent()
                });
        });
    }
}

impl Into<Box<dyn PostprocessingEffect>> for Bloom {
    fn into(self) -> Box<dyn PostprocessingEffect> {
        Box::new(self)
    }
}

pub struct BloomBuilder {
    assets_path: PathBuf,
    iterations: u32,
    threshold: f32,
    smooth_fade: f32,
    intensity: f32,
    downsample_framebuffers: Vec<Framebuffer>,
    upsample_framebuffers: Vec<Framebuffer>,
    v_blur_program_pipeline: Option<ProgramPipeline>,
    h_blur_program_pipeline: Option<ProgramPipeline>,
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
            downsample_framebuffers: vec![],
            upsample_framebuffers: vec![],
            v_blur_program_pipeline: None,
            h_blur_program_pipeline: None,
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
            let blur_vs = Shader::new(
                ShaderStage::Vertex,
                self.assets_path.join("sdr/fullscreen.vert.spv"),
            )
            .unwrap();

            let v_blur_fs = Shader::new(
                ShaderStage::Fragment,
                self.assets_path.join("sdr/gaussian_blur_vertical.frag.spv"),
            )
            .unwrap();

            let h_blur_fs = Shader::new(
                ShaderStage::Fragment,
                self.assets_path
                    .join("sdr/gaussian_blur_horizontal.frag.spv"),
            )
            .unwrap();

            let v_blur_pipeline = ProgramPipeline::new()
                .add_shader(&blur_vs)
                .add_shader(&v_blur_fs)
                .build()
                .unwrap();

            let h_blur_pipeline = ProgramPipeline::new()
                .add_shader(&blur_vs)
                .add_shader(&h_blur_fs)
                .build()
                .unwrap();

            (v_blur_pipeline, h_blur_pipeline)
        };

        Bloom {
            iterations: self.iterations,
            threshold: self.threshold,
            smooth_fade: self.smooth_fade,
            intensity: self.intensity,
            downsample_framebuffers: vec![],
            upsample_framebuffers: vec![],
            v_blur_program_pipeline,
            h_blur_program_pipeline,
            enabled: self.enabled,
        }
    }
}
