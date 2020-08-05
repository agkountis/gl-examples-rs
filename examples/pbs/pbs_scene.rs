use std::{ops::RangeInclusive, rc::Rc};

use engine::color::srgb_to_linear;
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
    skybox: TextureCube,
    irradiance: TextureCube,
    radiance: TextureCube,
}

#[repr(usize)]
enum SkyboxType {
    Original,
    Radiance,
    Irradiance,
}

struct Environment {
    maps: [EnvironmentMaps; 2],
    skybox_program_pipeline: ProgramPipeline,
    skybox_mesh: Mesh,
    active_environment: usize,
    skybox_type: SkyboxType,
}

struct Lighting {
    light_direction: [f32; 3],
    light_color: [f32; 3],
    light_intensity: f32,
}

struct ToneMapping {
    pipeline: ProgramPipeline,
    operator: usize,
    white_threshold: f32,
    exposure: f32,
}

struct Model {
    pub mesh: Rc<Mesh>,
    pub transform: Mat4,
}

#[derive(Default)]
struct Controls {
    left_mouse_button_pressed: bool,
    mouse_x: f32,
    mouse_y: f32,
    mouse_sensitivity: f32,
    scroll: f32,
    prev_x: f32,
    prev_y: f32,
    cursor_over_ui: bool,
}

struct Samplers {
    linear: Sampler,
    nearest: Sampler,
}

pub struct PbsScene {
    camera: Camera,
    model: Model,
    fullscreen_mesh: FullscreenMesh,
    material: PbsMetallicRoughnessMaterial,
    environment: Environment,
    framebuffer: Framebuffer,
    resolve_framebuffer: Framebuffer,
    samplers: Samplers,
    projection_matrix: Mat4,
    post_stack: PostprocessingStack,
    controls: Controls,
    lighting: Lighting,
    tone_mapping: ToneMapping,
    dt: f32,
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
                &Shader::new(ShaderStage::Vertex, asset_path.join("sdr/skybox.vert.spv")).unwrap(),
            )
            .add_shader(
                &Shader::new(
                    ShaderStage::Fragment,
                    asset_path.join("sdr/skybox.frag.spv"),
                )
                .unwrap(),
            )
            .build()
            .unwrap();

        let fullscreen_shader = Shader::new(
            ShaderStage::Vertex,
            asset_path.join("sdr/fullscreen.vert.spv"),
        )
        .unwrap();

        let tonemapping_prog = ProgramPipeline::new()
            .add_shader(&fullscreen_shader)
            .add_shader(
                &Shader::new(
                    ShaderStage::Fragment,
                    asset_path.join("sdr/tonemap.frag.spv"),
                )
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

        let skybox_exterior =
            TextureCube::new_from_file(asset_path.join("textures/pbs/ktx/skybox/skybox2.ktx"))
                .expect("Failed to load Skybox");

        let irradiance_exterior = TextureCube::new_from_file(
            asset_path.join("textures/pbs/ktx/irradiance/irradiance2.ktx"),
        )
        .expect("Failed to load Irradiance map");

        let radiance_exterior =
            TextureCube::new_from_file(asset_path.join("textures/pbs/ktx/radiance/radiance2.ktx"))
                .expect("Failed to load Radiance map");

        let skybox_interior =
            TextureCube::new_from_file(asset_path.join("textures/pbs/ktx/skybox/ibl_skybox.ktx"))
                .expect("Failed to load Skybox");

        let irradiance_interior = TextureCube::new_from_file(
            asset_path.join("textures/pbs/ktx/irradiance/ibl_irradiance.ktx"),
        )
        .expect("Failed to load Irradiance map");

        let radiance_interior = TextureCube::new_from_file(
            asset_path.join("textures/pbs/ktx/radiance/ibl_radiance.ktx"),
        )
        .expect("Failed to load Radiance map");

        let environments = [
            EnvironmentMaps {
                skybox: skybox_exterior,
                irradiance: irradiance_exterior,
                radiance: radiance_exterior,
            },
            EnvironmentMaps {
                skybox: skybox_interior,
                irradiance: irradiance_interior,
                radiance: radiance_interior,
            },
        ];

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
        );

        PbsScene {
            camera,
            model: Model {
                mesh,
                transform: Mat4::identity(),
            },
            fullscreen_mesh: FullscreenMesh::new(),
            material,
            environment: Environment {
                maps: environments,
                skybox_program_pipeline: skybox_prog,
                skybox_mesh,
                active_environment: 1,
                skybox_type: SkyboxType::Radiance,
            },
            framebuffer,
            resolve_framebuffer,
            samplers: Samplers {
                linear: sampler,
                nearest: sampler_nearest,
            },
            projection_matrix: projection,
            post_stack,
            controls: Controls {
                mouse_sensitivity: 2.0,
                ..Default::default()
            },
            lighting: Lighting {
                light_direction: [0.4, 0.0, -1.0],
                light_color: [1.0, 1.0, 1.0],
                light_intensity: 5.0,
            },
            tone_mapping: ToneMapping {
                pipeline: tonemapping_prog,
                operator: 0,
                white_threshold: 2.0,
                exposure: 1.5,
            },
            dt: 0.0,
        }
    }

    fn geometry_pass(&self) {
        self.framebuffer.bind();
        self.framebuffer.clear(&Vec4::new(0.0, 0.0, 0.0, 1.0));

        self.material.bind();

        let program_pipeline = self.material.program_pipeline();

        let mut light_color: Vec3 = srgb_to_linear3f(&self.lighting.light_color.into());
        light_color *= self.lighting.light_intensity;
        program_pipeline
            .set_texture_cube(
                "irradianceMap",
                &self.environment.maps[self.environment.active_environment].irradiance,
                &self.samplers.linear,
                ShaderStage::Fragment,
            )
            .set_texture_cube(
                "radianceMap",
                &self.environment.maps[self.environment.active_environment].radiance,
                &self.samplers.linear,
                ShaderStage::Fragment,
            )
            .set_vector3f(
                "wLightDirection",
                &self.lighting.light_direction.into(),
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

    fn skybox_pass(&self) {
        StateManager::set_depth_function(DepthFunction::LessOrEqual);
        StateManager::set_face_culling(FaceCulling::Front);

        self.resolve_framebuffer.bind();

        self.environment.skybox_program_pipeline.bind();

        let environment_map = match self.environment.skybox_type {
            SkyboxType::Original => {
                &self.environment.maps[self.environment.active_environment].skybox
            }
            SkyboxType::Radiance => {
                &self.environment.maps[self.environment.active_environment].radiance
            }
            SkyboxType::Irradiance => {
                &self.environment.maps[self.environment.active_environment].irradiance
            }
        };

        self.environment
            .skybox_program_pipeline
            .set_matrix4f("view", &self.camera.transform(), ShaderStage::Vertex)
            .set_texture_cube(
                "skybox",
                &environment_map,
                &self.samplers.linear,
                ShaderStage::Fragment,
            );

        self.environment.skybox_mesh.draw();

        self.resolve_framebuffer.unbind(false);
        self.environment.skybox_program_pipeline.unbind();

        StateManager::set_depth_function(DepthFunction::Less);
        StateManager::set_face_culling(FaceCulling::Back)
    }

    pub fn tonemap_pass(&self, width: u32, height: u32) {
        clear_default_framebuffer(&Vec4::new(0.0, 1.0, 0.0, 1.0));

        StateManager::set_viewport(0, 0, width as i32, height as i32);

        self.tone_mapping.pipeline.bind();

        self.tone_mapping
            .pipeline
            .set_texture_2d_with_id(
                "image",
                self.resolve_framebuffer.texture_attachment(0).id(),
                &self.samplers.nearest,
                ShaderStage::Fragment,
            )
            .set_integer(
                "tonemappingOperator",
                self.tone_mapping.operator as i32,
                ShaderStage::Fragment,
            )
            .set_float(
                "exposure",
                self.tone_mapping.exposure,
                ShaderStage::Fragment,
            )
            .set_float(
                "whiteThreshold",
                self.tone_mapping.white_threshold,
                ShaderStage::Fragment,
            );

        StateManager::set_front_face(FrontFace::Clockwise);
        self.fullscreen_mesh.draw();
        StateManager::set_front_face(FrontFace::CounterClockwise);

        self.tone_mapping.pipeline.unbind()
    }
}

impl Scene for PbsScene {
    fn start(&mut self, _: Context) {
        self.environment.skybox_program_pipeline.bind();
        self.environment.skybox_program_pipeline.set_matrix4f(
            "projection",
            &self.projection_matrix,
            ShaderStage::Vertex,
        );

        self.environment.skybox_program_pipeline.unbind();
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
            } => self.controls.left_mouse_button_pressed = true,
            WindowEvent::MouseInput {
                state: ElementState::Released,
                button: MouseButton::Left,
                ..
            } => {
                self.controls.left_mouse_button_pressed = false;

                self.controls.mouse_x = 0.0;
                self.controls.mouse_y = 0.0;
                self.controls.prev_x = 0.0;
                self.controls.prev_y = 0.0;
            }
            WindowEvent::CursorMoved { position, .. } => {
                if self.controls.left_mouse_button_pressed && !self.controls.cursor_over_ui {
                    self.controls.mouse_x = position.x as f32;
                    self.controls.mouse_y = position.y as f32;
                }
            }
            WindowEvent::MouseWheel {
                delta: MouseScrollDelta::LineDelta(_, y),
                ..
            } => {
                if !self.controls.cursor_over_ui {
                    self.controls.scroll = y;
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
                //TODO: Get this from the camera
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

        if self.controls.prev_x != 0.0 || self.controls.prev_y != 0.0 {
            dx = (self.controls.mouse_x - self.controls.prev_x) * self.controls.mouse_sensitivity;
            dy = (self.controls.mouse_y - self.controls.prev_y) * self.controls.mouse_sensitivity;
        }

        self.controls.prev_x = self.controls.mouse_x;
        self.controls.prev_y = self.controls.mouse_y;

        self.camera.update(dx, dy, self.controls.scroll, self.dt);

        self.controls.scroll = 0.0;

        Transition::None
    }

    fn pre_draw(&mut self, _: Context) {}

    fn draw(&mut self, context: Context) {
        let Context {
            window,
            framebuffer_cache,
            ..
        } = context;
        self.geometry_pass();
        self.skybox_pass();
        //self.bloom_pass();
        let size = window.inner_size();
        self.tonemap_pass(size.width, size.height);
        self.post_stack
            .apply(&self.resolve_framebuffer, framebuffer_cache);
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

                ui.spacing();

                // Lighting
                if imgui::CollapsingHeader::new(im_str!("Lighting"))
                    .default_open(true)
                    .open_on_arrow(true)
                    .open_on_double_click(true)
                    .build(ui)
                {
                    ui.spacing();
                    imgui::TreeNode::new(im_str!("Punctual"))
                        .default_open(true)
                        .open_on_arrow(true)
                        .open_on_double_click(true)
                        .framed(false)
                        .build(ui, || {
                            imgui::DragFloat3::new(
                                ui,
                                im_str!("Light Direction"),
                                &mut self.lighting.light_direction,
                            )
                            .min(-1.0)
                            .max(1.0)
                            .display_format(im_str!("%.2f"))
                            .speed(0.01)
                            .build();
                            imgui::ColorEdit::new(
                                im_str!("Light Color"),
                                &mut self.lighting.light_color,
                            )
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
                            .build(&ui, &mut self.lighting.light_intensity);
                            ui.new_line()
                        });

                    imgui::TreeNode::new(im_str!("Image-Based"))
                        .default_open(true)
                        .open_on_arrow(true)
                        .open_on_double_click(true)
                        .framed(false)
                        .build(ui, || {
                            imgui::ComboBox::new(im_str!("Environment")).build_simple_string(
                                ui,
                                &mut self.environment.active_environment,
                                &[im_str!("Exterior"), im_str!("Interior")],
                            );

                            let skybox_type_ref = unsafe {
                                &mut *(&mut self.environment.skybox_type as *mut SkyboxType
                                    as *mut usize)
                            };
                            imgui::ComboBox::new(im_str!("Skybox")).build_simple_string(
                                ui,
                                skybox_type_ref,
                                &[
                                    im_str!("Original"),
                                    im_str!("Radiance"),
                                    im_str!("Irradiance"),
                                ],
                            );
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
                        &mut self.tone_mapping.operator,
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

                    if self.tone_mapping.operator == 4 {
                        imgui::Slider::new(
                            im_str!("White Threshold"),
                            RangeInclusive::new(0.3, 30.0),
                        )
                        .display_format(im_str!("%.2f"))
                        .build(&ui, &mut self.tone_mapping.white_threshold);
                    }

                    imgui::Slider::new(im_str!("Exposure"), RangeInclusive::new(0.05, 30.0))
                        .display_format(im_str!("%.2f"))
                        .build(&ui, &mut self.tone_mapping.exposure);
                    ui.new_line()
                }

                // Post processing
                self.post_stack.gui(ui);

                ui.dummy([358.0, 0.0]);
                self.controls.cursor_over_ui = ui.is_window_focused() || ui.is_window_hovered();
            });

        self.controls.cursor_over_ui = (self.controls.cursor_over_ui
            || ui.is_any_item_hovered()
            || ui.is_any_item_focused()
            || ui.is_any_item_active())
            && !ui.is_window_collapsed();
    }

    fn post_draw(&mut self, _: Context) {}
}
