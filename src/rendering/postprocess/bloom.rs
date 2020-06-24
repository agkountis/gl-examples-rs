use crate::rendering::framebuffer::Framebuffer;
use crate::rendering::postprocess::{AsAny, AsAnyMut, PostprocessingEffect};
use crate::rendering::program_pipeline::ProgramPipeline;
use crate::rendering::shader::{Shader, ShaderStage};
use bitflags::_core::any::Any;
use std::path::{Path, PathBuf};

pub struct Bloom {
    downsample_framebuffers: Vec<Framebuffer>,
    upsample_framebuffers: Vec<Framebuffer>,
    v_blur_program_pipeline: ProgramPipeline,
    h_blur_program_pipeline: ProgramPipeline,
    enabled: bool,
}

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

    fn apply(&self, input: &Framebuffer) {
        println!("Applying Bloom")
    }
}

impl_as_any!(Bloom);

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
            let blur_vs = Shader::new_from_text(
                ShaderStage::Vertex,
                self.assets_path.join("sdr/fullscreen.vert"),
            )
            .unwrap();

            let v_blur_fs = Shader::new_from_text(
                ShaderStage::Fragment,
                self.assets_path.join("sdr/gaussian_blur_vertical.frag"),
            )
            .unwrap();

            let h_blur_fs = Shader::new_from_text(
                ShaderStage::Fragment,
                self.assets_path.join("sdr/gaussian_blur_horizontal.frag"),
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
            downsample_framebuffers: vec![],
            upsample_framebuffers: vec![],
            v_blur_program_pipeline,
            h_blur_program_pipeline,
            enabled: self.enabled,
        }
    }
}

pub struct Foo;

impl PostprocessingEffect for Foo {
    fn name(&self) -> &str {
        "Foo"
    }

    fn enable(&mut self) {
        unimplemented!()
    }

    fn disable(&mut self) {
        unimplemented!()
    }

    fn enabled(&self) -> bool {
        true
    }

    fn apply(&self, input: &Framebuffer) {
        println!("Applying Foo")
    }
}

impl_as_any!(Foo);

pub struct Bla;

impl PostprocessingEffect for Bla {
    fn name(&self) -> &str {
        "Bla"
    }

    fn enable(&mut self) {
        unimplemented!()
    }

    fn disable(&mut self) {
        unimplemented!()
    }

    fn enabled(&self) -> bool {
        false
    }

    fn apply(&self, input: &Framebuffer) {
        println!("Applying Bla!")
    }
}

impl_as_any!(Bla);
