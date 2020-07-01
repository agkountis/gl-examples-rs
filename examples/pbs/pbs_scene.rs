use std::{ops::RangeInclusive, rc::Rc};

use engine::{
    application::clear_default_framebuffer,
    camera::Camera,
    color::srgb_to_linear3f,
    imgui::*,
    math::{
        matrix::{perspective, Mat4},
        vector::{UVec2, Vec3, Vec4},
    },
    rendering::{
        framebuffer::{AttachmentType, Framebuffer, FramebufferAttachmentCreateInfo},
        material::{Material, PbsMetallicRoughnessMaterial},
        mesh::{FullscreenMesh, Mesh, MeshUtilities},
        postprocess::{
            bloom::{Bloom, BloomBuilder},
            PostprocessingEffect, PostprocessingStack, PostprocessingStackBuilder,
        },
        program_pipeline::ProgramPipeline,
        sampler::{MagnificationFilter, MinificationFilter, Sampler, WrappingMode},
        shader::{Shader, ShaderStage},
        state::{DepthFunction, FaceCulling, FrontFace, StateManager},
        texture::{SizedTextureFormat, TextureCube},
        Draw,
    },
    scene::Scene,
    scene::Transition,
    Context, Msaa,
};
use glutin::event::{
    ElementState, KeyboardInput, MouseButton, MouseScrollDelta, VirtualKeyCode, WindowEvent,
};

struct EnvironmentMaps {
    pub skybox: TextureCube,
    pub irradiance: TextureCube,
    pub radiance: TextureCube,
}

struct Model {
    pub mesh: Rc<Mesh>,
    pub transform: Mat4,
}

pub struct PbsScene {
    camera: Camera,
    model: Model,
    skybox_mesh: Mesh,
    fullscreen_mesh: FullscreenMesh,
    material: PbsMetallicRoughnessMaterial,
    environment_maps: EnvironmentMaps,
    skybox_program_pipeline: ProgramPipeline,
    horizontal_gaussian_pipeline: ProgramPipeline,
    vertical_gaussian_pipeline: ProgramPipeline,
    tonemapping_pipeline: ProgramPipeline,
    framebuffer: Framebuffer,
    resolve_framebuffer: Framebuffer,
    blur_framebuffers: [Framebuffer; 2],
    sampler: Sampler,
    sampler_nearest: Sampler,
    projection_matrix: Mat4,
    post_stack: PostprocessingStack,
    left_mouse_button_pressed: bool,
    mouse_x: f32,
    mouse_y: f32,
    mouse_sensitivity: f32,
    scroll: f32,
    prev_x: f32,
    prev_y: f32,
    dt: f32,
    light_color: [f32; 3],
    light_intensity: f32,
    exposure: f32,
    cursor_over_ui: bool,
    tone_mapping_operator: usize,
    white_threshold: f32,
}

impl PbsScene {
    pub fn new(context: Context) -> Self {
        let Context {
            window,
            settings,
            asset_manager,
            ..
        } = context;

        let asset_path = settings.asset_path.as_path();
        let camera = Camera::new(
            Vec3::new(0.0, 0.0, -60.0),
            Vec3::new(0.0, 0.0, 0.0),
            10.0,
            30.0,
            10.0,
            200.0,
            3.0,
            4.0,
        );

        let skybox_prog = ProgramPipeline::new()
            .add_shader(
                &Shader::new_from_text(ShaderStage::Vertex, asset_path.join("sdr/skybox.vert"))
                    .unwrap(),
            )
            .add_shader(
                &Shader::new_from_text(ShaderStage::Fragment, asset_path.join("sdr/skybox.frag"))
                    .unwrap(),
            )
            .build()
            .unwrap();

        let fullscreen_shader =
            Shader::new_from_text(ShaderStage::Vertex, asset_path.join("sdr/fullscreen.vert"))
                .unwrap();
        let horizontal_gaussian_prog = ProgramPipeline::new()
            .add_shader(&fullscreen_shader)
            .add_shader(
                &Shader::new_from_text(
                    ShaderStage::Fragment,
                    asset_path.join("sdr/gaussian_blur_horizontal.frag"),
                )
                .unwrap(),
            )
            .build()
            .unwrap();

        let vertical_gaussian_prog = ProgramPipeline::new()
            .add_shader(&fullscreen_shader)
            .add_shader(
                &Shader::new_from_text(
                    ShaderStage::Fragment,
                    asset_path.join("sdr/gaussian_blur_vertical.frag"),
                )
                .unwrap(),
            )
            .build()
            .unwrap();

        let tonemapping_prog = ProgramPipeline::new()
            .add_shader(&fullscreen_shader)
            .add_shader(
                &Shader::new_from_text(ShaderStage::Fragment, asset_path.join("sdr/tonemap.frag"))
                    .unwrap(),
            )
            .build()
            .unwrap();

        let mesh = asset_manager
            .load_mesh(asset_path.join("models/cerberus/cerberus.fbx"))
            .expect("Failed to load mesh");

        let skybox_mesh = MeshUtilities::generate_cube(1.0);

        let albedo = asset_manager
            .load_texture_2d(
                asset_path.join("textures/cerberus/Cerberus_A.png"),
                true,
                true,
            )
            .expect("Failed to load albedo texture");

        let metallic_roughness_ao = asset_manager
            .load_texture_2d(
                asset_path.join("textures/cerberus/Cerberus_M_R_AO.png"),
                false,
                true,
            )
            .expect("Failed to load metallic/roughness/ao texture");

        let normals = asset_manager
            .load_texture_2d(
                asset_path.join("textures/cerberus/Cerberus_N.png"),
                false,
                true,
            )
            .expect("Failed to load normals texture");

        let ibl_brdf_lut = asset_manager
            .load_texture_2d(
                asset_path.join("textures/pbs/ibl_brdf_lut.png"),
                false,
                false,
            )
            .expect("Failed to load BRDF LUT texture");

        let skybox =
            TextureCube::new_from_file(asset_path.join("textures/pbs/ktx/skybox/skybox2.ktx"))
                .expect("Failed to load Skybox");

        let irradiance = TextureCube::new_from_file(
            asset_path.join("textures/pbs/ktx/irradiance/irradiance2.ktx"),
        )
        .expect("Failed to load Irradiance map");

        let radiance =
            TextureCube::new_from_file(asset_path.join("textures/pbs/ktx/radiance/radiance2.ktx"))
                .expect("Failed to load Radiance map");

        let framebuffer = Framebuffer::new(
            UVec2::new(window.inner_size().width, window.inner_size().height),
            Msaa::X8,
            vec![
                FramebufferAttachmentCreateInfo::new(
                    SizedTextureFormat::Rgba16f,
                    AttachmentType::Texture,
                ),
                FramebufferAttachmentCreateInfo::new(
                    SizedTextureFormat::Rgba16f,
                    AttachmentType::Texture,
                ),
                FramebufferAttachmentCreateInfo::new(
                    SizedTextureFormat::Depth24Stencil8,
                    AttachmentType::Renderbuffer,
                ),
            ],
        )
        .unwrap_or_else(|error| panic!("Framebuffer creation error: {}", error));

        let resolve_framebuffer = Framebuffer::new(
            UVec2::new(window.inner_size().width, window.inner_size().height),
            Msaa::None,
            vec![
                FramebufferAttachmentCreateInfo::new(
                    SizedTextureFormat::Rgba16f,
                    AttachmentType::Texture,
                ),
                FramebufferAttachmentCreateInfo::new(
                    SizedTextureFormat::Rgba16f,
                    AttachmentType::Texture,
                ),
                FramebufferAttachmentCreateInfo::new(
                    SizedTextureFormat::Depth24Stencil8,
                    AttachmentType::Renderbuffer,
                ),
            ],
        )
        .unwrap_or_else(|error| panic!("Framebuffer creation error: {}", error));

        let blur_framebuffers = [
            Framebuffer::new(
                UVec2::new(
                    window.inner_size().width / 4,
                    window.inner_size().height / 4,
                ),
                Msaa::None,
                vec![FramebufferAttachmentCreateInfo::new(
                    SizedTextureFormat::Rgba16f,
                    AttachmentType::Texture,
                )],
            )
            .unwrap_or_else(|error| panic!("Framebuffer creation error: {}", error)),
            Framebuffer::new(
                UVec2::new(
                    window.inner_size().width / 4,
                    window.inner_size().height / 4,
                ),
                Msaa::None,
                vec![FramebufferAttachmentCreateInfo::new(
                    SizedTextureFormat::Rgba16f,
                    AttachmentType::Texture,
                )],
            )
            .unwrap_or_else(|error| panic!("Framebuffer creation error: {}", error)),
        ];

        let post_stack = PostprocessingStackBuilder::new()
            .with_effect(BloomBuilder::new(asset_path).build())
            .build();

        let sampler = Sampler::new(
            MinificationFilter::LinearMipmapLinear,
            MagnificationFilter::Linear,
            WrappingMode::ClampToEdge,
            WrappingMode::ClampToEdge,
            WrappingMode::ClampToEdge,
            Vec4::new(0.0, 0.0, 0.0, 0.0),
        );

        let sampler_nearest = Sampler::new(
            MinificationFilter::Nearest,
            MagnificationFilter::Nearest,
            WrappingMode::ClampToEdge,
            WrappingMode::ClampToEdge,
            WrappingMode::ClampToEdge,
            Vec4::new(0.0, 0.0, 0.0, 0.0),
        );

        let projection = perspective(
            window.inner_size().width,
            window.inner_size().height,
            60,
            0.5,
            500.0,
        );

        let material = PbsMetallicRoughnessMaterial::new(
            asset_path,
            albedo.clone(),
            metallic_roughness_ao.clone(),
            normals.clone(),
            None,
            ibl_brdf_lut.clone(),
        );

        PbsScene {
            camera,
            model: Model {
                mesh,
                transform: Mat4::identity(),
            },
            skybox_mesh,
            fullscreen_mesh: FullscreenMesh::new(),
            material,
            environment_maps: EnvironmentMaps {
                skybox,
                irradiance,
                radiance,
            },
            skybox_program_pipeline: skybox_prog,
            horizontal_gaussian_pipeline: horizontal_gaussian_prog,
            vertical_gaussian_pipeline: vertical_gaussian_prog,
            tonemapping_pipeline: tonemapping_prog,
            framebuffer,
            resolve_framebuffer,
            blur_framebuffers,
            sampler,
            sampler_nearest,
            projection_matrix: projection,
            post_stack,
            left_mouse_button_pressed: false,
            mouse_x: 0.0,
            mouse_y: 0.0,
            mouse_sensitivity: 2.0,
            scroll: 0.0,
            prev_x: 0.0,
            prev_y: 0.0,
            dt: 0.0,
            light_color: [1.0, 1.0, 1.0],
            light_intensity: 5.0,
            exposure: 1.5,
            cursor_over_ui: false,
            tone_mapping_operator: 0,
            white_threshold: 2.0,
        }
    }

    fn geometry_pass(&self) {
        self.framebuffer.bind();
        self.framebuffer.clear(&Vec4::new(0.0, 0.0, 0.0, 1.0));

        self.material.bind();

        let program_pipeline = self.material.program_pipeline();

        let mut light_color: Vec3 = srgb_to_linear3f(&self.light_color.into());
        light_color *= self.light_intensity;
        program_pipeline
            .set_texture_cube(
                "irradianceMap",
                &self.environment_maps.irradiance,
                &self.sampler,
                ShaderStage::Fragment,
            )
            .set_texture_cube(
                "radianceMap",
                &self.environment_maps.radiance,
                &self.sampler,
                ShaderStage::Fragment,
            )
            .set_vector3f(
                "wLightDirection",
                &Vec3::new(0.4, 0.0, -1.0),
                ShaderStage::Fragment,
            )
            .set_vector3f("lightColor", &light_color, ShaderStage::Fragment)
            .set_matrix4f("model", &self.model.transform, ShaderStage::Vertex)
            .set_matrix4f("view", &self.camera.transform(), ShaderStage::Vertex)
            .set_vector3f("eyePosition", &self.camera.position(), ShaderStage::Vertex)
            .set_matrix4f("projection", &self.projection_matrix, ShaderStage::Vertex);

        self.model.mesh.draw();

        self.framebuffer.unbind(false);

        Framebuffer::blit(&self.framebuffer, &self.resolve_framebuffer);

        self.material.unbind()
    }

    fn bloom_pass(&self) {
        let blur_strength = 6;

        self.blur_framebuffers.iter().for_each(|fb| {
            fb.bind();
            fb.clear(&Vec4::new(0.0, 0.0, 0.0, 1.0));
            fb.unbind(false)
        });

        for i in 0..blur_strength {
            let ping_pong_index = i % 2;

            let attachment_id: u32;
            if ping_pong_index == 0 {
                self.blur_framebuffers[ping_pong_index].bind();
                self.vertical_gaussian_pipeline.bind();

                if i == 0 {
                    attachment_id = self.resolve_framebuffer.texture_attachment(1).id();
                } else {
                    attachment_id = self.blur_framebuffers[1 - ping_pong_index]
                        .texture_attachment(0)
                        .id();
                }

                self.vertical_gaussian_pipeline.set_texture_2d_with_id(
                    "image",
                    attachment_id,
                    &self.sampler,
                    ShaderStage::Fragment,
                );
                StateManager::set_front_face(FrontFace::Clockwise);
                self.fullscreen_mesh.draw();
                StateManager::set_front_face(FrontFace::CounterClockwise);
                self.blur_framebuffers[ping_pong_index].unbind(false);
            } else {
                attachment_id = self.blur_framebuffers[1 - ping_pong_index]
                    .texture_attachment(0)
                    .id();
                self.blur_framebuffers[ping_pong_index].bind();

                self.horizontal_gaussian_pipeline.bind();
                self.horizontal_gaussian_pipeline.set_texture_2d_with_id(
                    "image",
                    attachment_id,
                    &self.sampler,
                    ShaderStage::Fragment,
                );

                StateManager::set_front_face(FrontFace::Clockwise);
                self.fullscreen_mesh.draw();
                StateManager::set_front_face(FrontFace::CounterClockwise);
                self.blur_framebuffers[ping_pong_index].unbind(false);
            }
        }
    }

    fn skybox_pass(&self) {
        StateManager::set_depth_function(DepthFunction::LessOrEqual);
        StateManager::set_face_culling(FaceCulling::Front);

        self.resolve_framebuffer.bind();

        self.skybox_program_pipeline.bind();

        self.skybox_program_pipeline.set_matrix4f(
            "view",
            &self.camera.transform(),
            ShaderStage::Vertex,
        );

        self.skybox_mesh.draw();

        self.resolve_framebuffer.unbind(false);
        self.skybox_program_pipeline.unbind();

        StateManager::set_depth_function(DepthFunction::Less);
        StateManager::set_face_culling(FaceCulling::Back)
    }

    pub fn tonemap_pass(&self) {
        clear_default_framebuffer(&Vec4::new(0.0, 1.0, 0.0, 1.0));

        StateManager::set_viewport(0, 0, 1920, 1080);

        self.tonemapping_pipeline.bind();

        self.tonemapping_pipeline
            .set_texture_2d_with_id(
                "image",
                self.resolve_framebuffer.texture_attachment(0).id(),
                &self.sampler_nearest,
                ShaderStage::Fragment,
            )
            .set_texture_2d_with_id(
                "bloomImage",
                self.blur_framebuffers[1].texture_attachment(0).id(),
                &self.sampler,
                ShaderStage::Fragment,
            )
            .set_integer(
                "tonemappingOperator",
                self.tone_mapping_operator as i32,
                ShaderStage::Fragment,
            )
            .set_float("exposure", self.exposure, ShaderStage::Fragment)
            .set_float(
                "whiteThreshold",
                self.white_threshold,
                ShaderStage::Fragment,
            );

        StateManager::set_front_face(FrontFace::Clockwise);
        self.fullscreen_mesh.draw();
        StateManager::set_front_face(FrontFace::CounterClockwise);

        self.tonemapping_pipeline.unbind()
    }
}

impl Scene for PbsScene {
    fn start(&mut self, _: Context) {
        self.skybox_program_pipeline.bind();
        self.skybox_program_pipeline.set_matrix4f(
            "projection",
            &self.projection_matrix,
            ShaderStage::Vertex,
        );

        self.skybox_program_pipeline.set_texture_cube(
            "skybox",
            &self.environment_maps.radiance,
            &self.sampler,
            ShaderStage::Fragment,
        );
        self.skybox_program_pipeline.unbind();
    }

    fn stop(&mut self, _: Context) {}

    fn pause(&mut self, _: Context) {}

    fn resume(&mut self, _: Context) {}

    fn handle_event(&mut self, _: Context, event: WindowEvent) -> Transition {
        match event {
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } => self.left_mouse_button_pressed = true,
            WindowEvent::MouseInput {
                state: ElementState::Released,
                button: MouseButton::Left,
                ..
            } => {
                self.left_mouse_button_pressed = false;

                self.mouse_x = 0.0;
                self.mouse_y = 0.0;
                self.prev_x = 0.0;
                self.prev_y = 0.0;
            }
            WindowEvent::CursorMoved { position, .. } => {
                if self.left_mouse_button_pressed && !self.cursor_over_ui {
                    self.mouse_x = position.x as f32;
                    self.mouse_y = position.y as f32;
                }
            }
            WindowEvent::MouseWheel {
                delta: MouseScrollDelta::LineDelta(_, y),
                ..
            } => {
                if !self.cursor_over_ui {
                    self.scroll = y;
                }
            }
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state: ElementState::Released,
                        virtual_keycode: Some(VirtualKeyCode::Escape),
                        ..
                    },
                ..
            } => return Transition::Quit,
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state: ElementState::Released,
                        virtual_keycode: Some(VirtualKeyCode::A),
                        ..
                    },
                ..
            } => {
                use gl_bindings as gl;
                unsafe { gl::Enable(gl::MULTISAMPLE) }
            }
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        scancode,
                        state,
                        virtual_keycode: Some(VirtualKeyCode::S),
                        ..
                    },
                ..
            } => {
                use gl_bindings as gl;
                unsafe { gl::Disable(gl::MULTISAMPLE) }
            }
            WindowEvent::Resized(size) => {
                let x = size.width;
                let y = size.height;
                self.projection_matrix = perspective(x, y, 60, 0.5, 500.0);
                StateManager::set_viewport(0, 0, x as i32, y as i32)
            }
            _ => {}
        }
        Transition::None
    }

    fn update(&mut self, context: Context) -> Transition {
        let Context { timer, .. } = context;

        self.dt = timer.get_delta();

        let mut dx = 0.0;
        let mut dy = 0.0;

        if self.prev_x != 0.0 || self.prev_y != 0.0 {
            dx = (self.mouse_x - self.prev_x) * self.mouse_sensitivity;
            dy = (self.mouse_y - self.prev_y) * self.mouse_sensitivity;
        }

        self.prev_x = self.mouse_x;
        self.prev_y = self.mouse_y;

        self.camera.update(dx, dy, self.scroll, self.dt);

        self.scroll = 0.0;

        Transition::None
    }

    fn pre_draw(&mut self, _: Context) {}

    fn draw(&mut self, _: Context) {
        self.geometry_pass();
        self.skybox_pass();
        self.bloom_pass();
        self.tonemap_pass();
        // self.post_stack.apply(&self.resolve_framebuffer)
    }

    fn gui(&mut self, ui: &Ui) {
        imgui::Window::new(im_str!("Inspector"))
            .size([358.0, 1079.0], Condition::Appearing)
            .position([2.0, 0.0], Condition::Always)
            .mouse_inputs(true)
            .resizable(false)
            .movable(false)
            .always_auto_resize(true)
            .build(ui, || {
                ui.dummy([358.0, 0.0]);

                // Material
                self.material.gui(ui);

                // Lighting
                if imgui::CollapsingHeader::new(im_str!("Lighting"))
                    .default_open(true)
                    .open_on_arrow(true)
                    .open_on_double_click(true)
                    .build(ui)
                {
                    ui.spacing();
                    ui.group(|| {
                        // imgui::ComboBox::new("Light Color Presets")
                        imgui::ColorEdit::new(im_str!("Light Color"), &mut self.light_color)
                            .format(ColorFormat::Float)
                            .options(true)
                            .picker(true)
                            .alpha(false)
                            .build(&ui);
                        imgui::Slider::new(
                            im_str!("Light Intensity"),
                            RangeInclusive::new(0.01, 300.0),
                        )
                        .display_format(im_str!("%.1f"))
                        .build(&ui, &mut self.light_intensity);
                        ui.new_line()
                    });
                }

                // Camera
                if imgui::CollapsingHeader::new(im_str!("Camera"))
                    .default_open(true)
                    .open_on_arrow(true)
                    .open_on_double_click(true)
                    .build(ui)
                {
                    ui.spacing();
                    ui.group(|| {
                        let mut orbit_speed = self.camera.orbit_speed();
                        if imgui::Slider::new(
                            im_str!("Orbit Speed"),
                            RangeInclusive::new(1.0, 10.0),
                        )
                        .display_format(im_str!("%.2f"))
                        .build(&ui, &mut orbit_speed)
                        {
                            self.camera.set_orbit_speed(orbit_speed)
                        }

                        let mut orbit_dampening = self.camera.orbit_dampening();
                        if imgui::Slider::new(
                            im_str!("Orbit Dampening"),
                            RangeInclusive::new(1.0, 10.0),
                        )
                        .display_format(im_str!("%.2f"))
                        .build(&ui, &mut orbit_dampening)
                        {
                            self.camera.set_orbit_dampening(orbit_dampening)
                        }

                        let mut zoom_speed = self.camera.zoom_speed();
                        if imgui::Slider::new(im_str!("Zoom Speed"), RangeInclusive::new(1.0, 40.0))
                            .display_format(im_str!("%.2f"))
                            .build(&ui, &mut zoom_speed)
                        {
                            self.camera.set_zoom_speed(zoom_speed)
                        }

                        let mut zoom_dampening = self.camera.zoom_dampening();
                        if imgui::Slider::new(
                            im_str!("Zoom Dampening"),
                            RangeInclusive::new(0.1, 10.0),
                        )
                        .display_format(im_str!("%.2f"))
                        .build(&ui, &mut zoom_dampening)
                        {
                            self.camera.set_zoom_dampening(zoom_dampening)
                        }

                        ui.new_line()
                    });
                }

                // Tonemapping
                if imgui::CollapsingHeader::new(im_str!("Tone Mapping"))
                    .default_open(true)
                    .open_on_arrow(true)
                    .open_on_double_click(true)
                    .build(ui)
                {
                    ui.spacing();
                    imgui::ComboBox::new(im_str!("Operator")).build_simple_string(
                        &ui,
                        &mut self.tone_mapping_operator,
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

                    if self.tone_mapping_operator == 4 {
                        imgui::Slider::new(
                            im_str!("White Threshold"),
                            RangeInclusive::new(0.3, 30.0),
                        )
                        .display_format(im_str!("%.2f"))
                        .build(&ui, &mut self.white_threshold);
                    }

                    imgui::Slider::new(im_str!("Exposure"), RangeInclusive::new(0.05, 30.0))
                        .display_format(im_str!("%.2f"))
                        .build(&ui, &mut self.exposure);
                    ui.new_line()
                }

                // Post processing
                self.post_stack.gui(ui);

                ui.dummy([358.0, 0.0]);
                self.cursor_over_ui = ui.is_window_focused() || ui.is_window_hovered();
            });

        self.cursor_over_ui = (self.cursor_over_ui
            || ui.is_any_item_hovered()
            || ui.is_any_item_focused()
            || ui.is_any_item_active())
            && !ui.is_window_collapsed();
    }

    fn post_draw(&mut self, _: Context) {}
}
