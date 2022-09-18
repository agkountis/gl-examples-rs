use crate::framebuffer::Framebuffer;
use crate::imgui::Gui;
use crate::postprocess::{AsAny, AsAnyMut, PostprocessingEffect, FULLSCREEN_VERTEX_SHADER_PATH};
use crate::shader::{Shader, ShaderCreateInfo, ShaderStage};
use crate::Context;
use imgui::{Condition, TextureId, Ui};
use std::any::Any;
use std::rc::Rc;

use crate::math::{UVec2, Vec4};
use crate::mesh::utilities::draw_full_screen_quad;
use crate::sampler::{Anisotropy, MagnificationFilter, MinificationFilter, Sampler, WrappingMode};
use crate::texture::SizedTextureFormat;

pub struct DepthOfField {
    dof_shader: Rc<Shader>,
    depth_fb: Rc<Framebuffer>,
    linear_sampler: Sampler,
    enabled: bool,
    show_debug_window: bool,
}

impl_as_any!(DepthOfField);

impl DepthOfField {
    pub fn new(context: Context, enabled: bool) -> Self {
        let Context {
            device,
            framebuffer_cache,
            window,
            ..
        } = context;

        let dof_shader = device.shader_manager().create_shader(
            &ShaderCreateInfo::builder("DoF Shader")
                .stage(ShaderStage::Vertex, FULLSCREEN_VERTEX_SHADER_PATH)
                .stage(ShaderStage::Fragment, "assets/shaders/dof.frag")
                .keyword_set(&[
                    "DOF_PASS_COC",
                    "DOF_PASS_DOWNSAMPLE",
                    "DOF_PASS_BOKEH",
                    "DOF_PASS_BOKEH_BLUR",
                    "DOF_PASS_COMBINE",
                ])
                .build(),
        );

        let linear_sampler = Sampler::new(
            MinificationFilter::Linear,
            MagnificationFilter::Linear,
            WrappingMode::ClampToEdge,
            WrappingMode::ClampToEdge,
            WrappingMode::ClampToEdge,
            Vec4::new(0.0, 0.0, 0.0, 1.0),
            Anisotropy::None,
        );

        let size = UVec2::new(window.inner_size().width, window.inner_size().height);
        Self {
            depth_fb: framebuffer_cache.get_temporary(
                "CoC Framebuffer",
                size,
                SizedTextureFormat::R16f,
                None,
            ),
            dof_shader,
            enabled,
            show_debug_window: false,
            linear_sampler,
        }
    }
}

impl PostprocessingEffect for DepthOfField {
    fn name(&self) -> &str {
        "DepthOfField"
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

        let color = input.texture_attachment(0);
        let depth = input.texture_attachment(1);
        assert!(
            depth.is_depth_stencil(),
            "Texture attachment at index 1 is not depth-stencil"
        );

        // CoC pass
        self.dof_shader.enable_keyword("DOF_PASS_COC");
        self.dof_shader
            .bind_texture_2d_with_id(1, depth.id(), &self.linear_sampler);
        self.depth_fb.bind();
        self.depth_fb.clear(&Vec4::new(0.5, 0.5, 0.5, 1.0));
        draw_full_screen_quad();
        self.depth_fb.unbind(false);
        self.dof_shader.disable_keyword("DOF_PASS_COC");

        // Blit to half size
        let half_size = input.size() / 2;
        let tmp0 = framebuffer_cache.get_temporary("tmp0", half_size, color.format(), None);
        self.dof_shader.enable_keyword("DOF_PASS_DOWNSAMPLE");
        self.dof_shader.bind_texture_2d_with_id(
            0,
            input.texture_attachment(0).id(),
            &self.linear_sampler,
        );
        self.dof_shader.bind_texture_2d_with_id(
            2,
            self.depth_fb.texture_attachment(0).id(),
            &self.linear_sampler,
        );
        tmp0.bind();
        draw_full_screen_quad();
        tmp0.unbind(false);
        self.dof_shader.disable_keyword("DOF_PASS_DOWNSAMPLE");

        // Bokeh pass at half size.
        let tmp1 = framebuffer_cache.get_temporary("tmp1", half_size, color.format(), None);
        self.dof_shader.enable_keyword("DOF_PASS_BOKEH");
        self.dof_shader.bind_texture_2d_with_id(
            0,
            tmp0.texture_attachment(0).id(),
            &self.linear_sampler,
        );
        tmp1.bind();
        draw_full_screen_quad();
        tmp1.unbind(false);
        self.dof_shader.disable_keyword("DOF_PASS_BOKEH");

        // Blur Bokeh at half size.
        self.dof_shader.enable_keyword("DOF_PASS_BOKEH_BLUR");
        self.dof_shader.bind_texture_2d_with_id(
            0,
            tmp1.texture_attachment(0).id(),
            &self.linear_sampler,
        );
        tmp0.bind();
        draw_full_screen_quad();
        tmp0.unbind(false);
        self.dof_shader.unbind();
        self.dof_shader.disable_keyword("DOF_PASS_BOKEH_BLUR");

        // Blit back to main image.
        self.dof_shader.enable_keyword("DOF_PASS_COMBINE");
        self.dof_shader
            .bind_texture_2d_with_id(0, color.id(), &self.linear_sampler);
        self.dof_shader.bind_texture_2d_with_id(
            2,
            self.depth_fb.texture_attachment(0).id(),
            &self.linear_sampler,
        );
        self.dof_shader.bind_texture_2d_with_id(
            3,
            tmp0.texture_attachment(0).id(),
            &self.linear_sampler,
        );
        input.bind();
        draw_full_screen_quad();
        input.unbind(false);
        self.dof_shader.disable_keyword("DOF_PASS_COMBINE");
    }
}

impl Gui for DepthOfField {
    fn gui(&mut self, ui: &Ui) {
        ui.group(|| {
            ui.checkbox("##dof", &mut self.enabled);
            ui.same_line_with_pos(20.0);
            imgui::TreeNode::new("Depth Of Field")
                .default_open(true)
                .open_on_arrow(true)
                .open_on_double_click(true)
                .framed(false)
                .build(ui, || {
                    ui.indent();

                    ui.checkbox("Show debug window", &mut self.show_debug_window);

                    if self.show_debug_window {
                        imgui::Window::new("DoF Debug")
                            .focus_on_appearing(true)
                            .bring_to_front_on_focus(true)
                            .size([512.0f32, 512.0f32], Condition::Appearing)
                            .build(ui, || {
                                imgui::Image::new(
                                    TextureId::new(
                                        self.depth_fb.texture_attachment(0).id() as usize
                                    ),
                                    [512f32, 512f32],
                                )
                                .uv0([0.0, 1.0])
                                .uv1([1.0, 0.0])
                                .build(ui);
                            });
                    }

                    ui.unindent()
                });
        });
    }
}
