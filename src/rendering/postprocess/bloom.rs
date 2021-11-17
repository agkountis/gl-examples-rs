use std::any::Any;
use std::rc::Rc;

use crevice::std140::AsStd140;

use crate::postprocess::FULLSCREEN_VERTEX_SHADER_PATH;
use crate::rendering::shader::Shader;
use crate::shader::ShaderCreateInfo;
use crate::{
    color::srgb_to_linear,
    core::math::{UVec2, Vec4},
    imgui::{ColorFormat, Condition, Gui, TextureId, Ui},
    rendering::{
        buffer::{Buffer, BufferStorageFlags, BufferTarget, MapModeFlags},
        framebuffer::{Framebuffer, TemporaryFramebufferPool},
        mesh::utilities::draw_full_screen_quad,
        postprocess::{AsAny, AsAnyMut, PostprocessingEffect},
        sampler::{Anisotropy, MagnificationFilter, MinificationFilter, Sampler, WrappingMode},
        shader::ShaderStage,
        state::{BlendFactor, StateManager},
        texture::Texture2D,
    },
    Context,
};

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
const MIN_LENS_DIRT_INTENSITY: f32 = 0.0;
const MAX_LENS_DIRT_INTENSITY: f32 = 100.0;

#[repr(C)]
#[derive(Debug, AsStd140)]
struct BloomUboData {
    spread: f32,
    filter: mint::Vector4<f32>,
    intensity: f32,
    use_lens_dirt: i32,
    lens_dirt_intensity: f32,
    tint: mint::Vector3<f32>,
}

impl Default for BloomUboData {
    fn default() -> Self {
        Self {
            spread: 0.0,
            filter: [0.0, 0.0, 0.0, 0.0].into(),
            intensity: 0.0,
            use_lens_dirt: 0,
            lens_dirt_intensity: 0.0,
            tint: [1.0, 1.0, 1.0].into(),
        }
    }
}

pub struct Bloom {
    iterations: u32,
    spread: f32,
    threshold: f32,
    smooth_fade: f32,
    intensity: f32,
    tint: [f32; 3],
    resolution_divisors: [u32; 2],
    resolution_divisor_index: usize,
    bloom_shader: Rc<Shader>,
    ubo_data: BloomUboData,
    ubo: Buffer,
    linear_sampler: Sampler,
    blit_framebuffers: Vec<Rc<Framebuffer>>,
    enabled: bool,
    show_debug_window: bool,
    anamorphic_stretch: f32,
    enable_lens_dirt: bool,
    lens_dirt_intensity: f32,
    lens_dirt: Rc<Texture2D>,
}

impl_as_any!(Bloom);

impl Bloom {
    fn update_uniforms(&mut self) {
        let threshold = srgb_to_linear(self.threshold);
        let knee = threshold * self.smooth_fade;

        self.ubo_data.spread = self.spread;
        self.ubo_data.filter = [
            threshold,
            threshold - knee,
            2.0 * knee,
            0.25 / (knee + EPSILON),
        ]
        .into();
        self.ubo_data.tint = self.tint.into();
        self.ubo_data.intensity = self.intensity;
        self.ubo_data.use_lens_dirt = self.enable_lens_dirt as i32;
        self.ubo_data.lens_dirt_intensity = self.lens_dirt_intensity;

        self.ubo.fill_mapped(0, &self.ubo_data.as_std140());
    }

    fn downsampling_passes(
        &mut self,
        input: &Framebuffer,
        framebuffer_cache: &mut TemporaryFramebufferPool,
    ) -> Rc<Framebuffer> {
        let resolution_divisor = self.resolution_divisors[self.resolution_divisor_index];
        let input_size = input.size();

        // Blit to half resolution and filter bright pixels.
        let mut size = UVec2::new(
            input_size.x / resolution_divisor,
            input_size.y / resolution_divisor,
        );

        //TODO: this can exceed supported texture limits leading to an incomplete attachment error/crash.
        size.y += (size.y as f32 * self.anamorphic_stretch) as u32;

        let attachment = input.texture_attachment(0);

        assert!(
            !attachment.is_depth_stencil(),
            "Bloom effect do not support depth texture attachments."
        );

        let format = attachment.format();

        self.blit_framebuffers
            .iter()
            .for_each(|fb| framebuffer_cache.release_temporary(Rc::clone(fb)));
        self.blit_framebuffers.clear();

        let mut current_destination = framebuffer_cache.get_temporary(size, format, None);

        self.blit_framebuffers.push(Rc::clone(&current_destination));

        current_destination.bind();
        current_destination.clear(&Vec4::new(0.5, 0.5, 0.5, 1.0));

        self.bloom_shader
            .enable_keyword("BLOOM_PASS_DOWNSAMPLE_PREFILTER");
        self.bloom_shader.set_texture_2d_with_id(
            0,
            input.texture_attachments()[0].id(),
            &self.linear_sampler,
        );

        draw_full_screen_quad();

        current_destination.unbind(false);

        let mut current_source = Rc::clone(&current_destination);

        self.bloom_shader.enable_keyword("BLOOM_PASS_DOWNSAMPLE");

        for _ in 1..self.iterations {
            size.x /= resolution_divisor;
            size.y /= resolution_divisor;

            //TODO: this can exceed supported texture limits leading to an incomplete attachment error/crash.
            size.y += (size.y as f32 * self.anamorphic_stretch) as u32;

            if size.y < 2 || size.x < 2 {
                break;
            }

            current_destination = framebuffer_cache.get_temporary(size, format, None);

            self.blit_framebuffers.push(Rc::clone(&current_destination));
            //TODO: Do a downsample blit here

            current_destination.bind();
            current_destination.clear(&Vec4::new(0.5, 0.5, 0.5, 1.0));

            self.bloom_shader.set_texture_2d_with_id(
                0,
                current_source.texture_attachments()[0].id(),
                &self.linear_sampler,
            );

            draw_full_screen_quad();

            current_destination.unbind(false);

            current_source = Rc::clone(&current_destination);
        }

        current_source
    }

    fn upsampling_passes(&self, input: Rc<Framebuffer>) -> Rc<Framebuffer> {
        let mut current_source = input;

        self.bloom_shader.enable_keyword("BLOOM_PASS_UPSAMPLE");
        for temporary in self.blit_framebuffers.iter().rev().skip(1) {
            let current_destination = Rc::clone(temporary);
            //TODO: Do an upsampling blit here
            current_destination.bind();

            self.bloom_shader.set_texture_2d_with_id(
                0,
                current_source.texture_attachments()[0].id(),
                &self.linear_sampler,
            );

            StateManager::enable_blending_with_function(BlendFactor::One, BlendFactor::One);
            draw_full_screen_quad();
            StateManager::disable_blending();

            current_destination.unbind(false);

            current_source = Rc::clone(&current_destination);
        }

        current_source
    }

    fn composition_pass(&self, input: &Framebuffer, output: &Framebuffer) {
        self.bloom_shader
            .enable_keyword("BLOOM_PASS_UPSAMPLE_APPLY");
        self.bloom_shader.set_texture_2d_with_id(
            0,
            input.texture_attachments()[0].id(),
            &self.linear_sampler,
        );
        self.bloom_shader.set_texture_2d_with_id(
            1,
            output.texture_attachments()[0].id(),
            &self.linear_sampler,
        );
        self.bloom_shader
            .set_texture_2d_with_id(2, self.lens_dirt.get_id(), &self.linear_sampler);
        output.bind();

        draw_full_screen_quad();

        output.unbind(false);
        self.bloom_shader.unbind();
    }
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

    fn apply(&mut self, input: &Framebuffer, context: Context) {
        let Context {
            framebuffer_cache, ..
        } = context;

        self.update_uniforms();

        let mut current_source = self.downsampling_passes(input, framebuffer_cache);
        current_source = self.upsampling_passes(Rc::clone(&current_source));
        self.composition_pass(&current_source, input);
    }
}

impl Gui for Bloom {
    fn gui(&mut self, ui: &Ui) {
        ui.group(|| {
            ui.checkbox("##bloom", &mut self.enabled);
            ui.same_line_with_pos(20.0);
            imgui::TreeNode::new("Bloom")
                .default_open(true)
                .open_on_arrow(true)
                .open_on_double_click(true)
                .framed(false)
                .build(ui, || {
                    ui.indent();

                    ui.checkbox("Show debug window", &mut self.show_debug_window);

                    if self.show_debug_window {
                        imgui::Window::new("Bloom Debug")
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

                    ui.combo(
                        "Resolution",
                        &mut self.resolution_divisor_index,
                        &self.resolution_divisors,
                        |&a| match a {
                            2 => "Half".into(),
                            4 => "Quarter".into(),
                            _ => "".into(),
                        },
                    );

                    imgui::ColorEdit::new("Tint", &mut self.tint)
                        .format(ColorFormat::Float)
                        .options(true)
                        .picker(true)
                        .alpha(false)
                        .build(ui);

                    if imgui::Slider::new("Iterations", MIN_ITERATIONS, MAX_ITERATIONS)
                        .build(ui, &mut self.iterations)
                    {
                        self.blit_framebuffers = Vec::with_capacity(self.iterations as usize);
                    }

                    imgui::Slider::new("Spread", 1.0, 10.0)
                        .display_format("%.2f")
                        .build(ui, &mut self.spread);

                    imgui::Slider::new("Threshold", MIN_THRESHOLD, MAX_THRESHOLD)
                        .display_format("%.2f")
                        .build(ui, &mut self.threshold);

                    imgui::Slider::new("Smooth Fade", MIN_SMOOTH_FADE, MAX_SMOOTH_FADE)
                        .display_format("%.2f")
                        .build(ui, &mut self.smooth_fade);

                    imgui::Slider::new("Intensity", MIN_INTENSITY, MAX_INTENSITY)
                        .display_format("%.2f")
                        .build(ui, &mut self.intensity);

                    imgui::Slider::new("Anamorphic Stretch", 0.0, 1.0)
                        .display_format("%.2f")
                        .build(ui, &mut self.anamorphic_stretch);

                    ui.checkbox("Lens Dirt", &mut self.enable_lens_dirt);
                    if self.enable_lens_dirt {
                        imgui::Slider::new(
                            "Lens Dirt Intensity",
                            MIN_LENS_DIRT_INTENSITY,
                            MAX_LENS_DIRT_INTENSITY,
                        )
                        .display_format("%.2f")
                        .build(ui, &mut self.lens_dirt_intensity);

                        ui.text("Lens Dirt Map");
                        let tex_id = self.lens_dirt.get_id();
                        imgui::Image::new(TextureId::new(tex_id as usize), [128.0, 128.0])
                            .build(ui);
                    }

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

    pub fn build(self, context: Context) -> Bloom {
        let Context {
            asset_manager,
            settings,
            device,
            ..
        } = context;

        let asset_path = settings.asset_path.as_path();
        let lens_dirt = asset_manager
            .load_texture_2d(asset_path.join("textures/lens_dirt_mask.png"), true, false)
            .expect("Failed to load lens dirt texture.");

        let mut bloom_shader = device.shader_manager().create_shader(
            &ShaderCreateInfo::builder("Bloom Shader")
                .stage(ShaderStage::Vertex, FULLSCREEN_VERTEX_SHADER_PATH)
                .stage(
                    ShaderStage::Fragment,
                    "src/rendering/postprocess/shaders/bloom.frag",
                )
                .keyword_set(&[
                    "BLOOM_PASS_DOWNSAMPLE_PREFILTER",
                    "BLOOM_PASS_DOWNSAMPLE",
                    "BLOOM_PASS_UPSAMPLE",
                    "BLOOM_PASS_UPSAMPLE_APPLY",
                ])
                .build(),
        );

        let mut ubo = Buffer::new(
            "Bloom UBO",
            std::mem::size_of::<<BloomUboData as AsStd140>::Std140Type>() as isize,
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
            spread: 1.0,
            threshold: self.threshold,
            smooth_fade: self.smooth_fade,
            intensity: self.intensity,
            tint: [1.0; 3],
            resolution_divisors: [2, 4],
            resolution_divisor_index: 0,
            bloom_shader,
            ubo_data: Default::default(),
            ubo,
            linear_sampler,
            blit_framebuffers: Vec::with_capacity(self.iterations as usize),
            enabled: self.enabled,
            show_debug_window: false,
            anamorphic_stretch: 0.0,
            enable_lens_dirt: true,
            lens_dirt_intensity: 30.0,
            lens_dirt,
        }
    }
}
