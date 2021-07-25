use std::{
    mem,
    {ops::RangeInclusive, rc::Rc},
};

use engine::{
    camera::Camera,
    color::srgb_to_linear3f,
    imgui::*,
    math::{
        inverse,
        matrix::{perspective, Mat4},
        transpose,
        vector::{UVec2, Vec2, Vec3, Vec4},
    },
    rendering::{
        buffer::{Buffer, BufferStorageFlags, BufferTarget, MapModeFlags},
        framebuffer::{AttachmentType, Framebuffer, FramebufferAttachmentCreateInfo},
        material::{Material, PbsMetallicRoughnessMaterial},
        mesh::{Mesh, MeshUtilities},
        postprocess::{
            bloom::BloomBuilder, tone_mapper::ToneMapper, PostprocessingStack,
            PostprocessingStackBuilder,
        },
        program_pipeline::ProgramPipeline,
        sampler::{Anisotropy, MagnificationFilter, MinificationFilter, Sampler, WrappingMode},
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
    geometric_specular_aa: bool,
    specular_ao: bool,
    brdf_type: usize,
    multi_scattering: bool,
    ss_variance_and_threshold: Vec2,
    max_reflection_lod: i32,
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

#[repr(C)]
struct VertexPerDrawUniforms {
    model_matrix: Mat4,
    normal_matrix: Mat4,
}

#[repr(C)]
struct FragmentPerFrameUniforms {
    light_direction: Vec4,
    light_color: Vec4,
    ss_variance_and_threshold: Vec2,
    geometric_specular_aa: i32,
    specular_ao: i32,
    render_mode: i32,
    brdf_type: i32,
    multi_scattering: i32,
    max_reflection_lod: f32,
}

// TODO: Use this to group framebuffers
pub struct Framebuffers {
    msaa_framebuffer: Framebuffer,
    resolve_framebuffer: Framebuffer,
}

pub struct PbsScene {
    camera: Camera,
    model: Model,
    material: PbsMetallicRoughnessMaterial,
    environment: Environment,
    msaa_framebuffers: [Framebuffer; 4],
    resolve_framebuffer: Framebuffer,
    sampler_linear: Sampler,
    post_stack: PostprocessingStack,
    controls: Controls,
    lighting: Lighting,
    render_mode: usize,
    vertex_per_draw_ubo: Buffer,
    fragment_per_frame_ubo: Buffer,
    msaa_framebuffer_index: usize,
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
            60,
            0.5,
            500.0,
            10.0,
            30.0,
            10.0,
            200.0,
            3.0,
            4.0,
        );

        let skybox_prog = ProgramPipeline::new()
            .add_shader(
                &Shader::new(ShaderStage::Vertex, asset_path.join("sdr/skybox.vert")).unwrap(),
            )
            .add_shader(
                &Shader::new(ShaderStage::Fragment, asset_path.join("sdr/skybox.frag")).unwrap(),
            )
            .build()
            .unwrap();

        let mesh = asset_manager
            .load_mesh(asset_path.join("models/cerberus/cerberus.glb"))
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

        let msaa_framebuffers = [
            Framebuffer::new(
                UVec2::new(window.inner_size().width, window.inner_size().height),
                Msaa::None,
                vec![
                    FramebufferAttachmentCreateInfo::new(
                        SizedTextureFormat::Rgba16f,
                        AttachmentType::Renderbuffer,
                    ),
                    FramebufferAttachmentCreateInfo::new(
                        SizedTextureFormat::Depth24Stencil8,
                        AttachmentType::Renderbuffer,
                    ),
                ],
            )
            .unwrap_or_else(|error| panic!("Framebuffer creation error: {}", error)),
            Framebuffer::new(
                UVec2::new(window.inner_size().width, window.inner_size().height),
                Msaa::X2,
                vec![
                    FramebufferAttachmentCreateInfo::new(
                        SizedTextureFormat::Rgba16f,
                        AttachmentType::Renderbuffer,
                    ),
                    FramebufferAttachmentCreateInfo::new(
                        SizedTextureFormat::Depth24Stencil8,
                        AttachmentType::Renderbuffer,
                    ),
                ],
            )
            .unwrap_or_else(|error| panic!("Framebuffer creation error: {}", error)),
            Framebuffer::new(
                UVec2::new(window.inner_size().width, window.inner_size().height),
                Msaa::X4,
                vec![
                    FramebufferAttachmentCreateInfo::new(
                        SizedTextureFormat::Rgba16f,
                        AttachmentType::Renderbuffer,
                    ),
                    FramebufferAttachmentCreateInfo::new(
                        SizedTextureFormat::Depth24Stencil8,
                        AttachmentType::Renderbuffer,
                    ),
                ],
            )
            .unwrap_or_else(|error| panic!("Framebuffer creation error: {}", error)),
            Framebuffer::new(
                UVec2::new(window.inner_size().width, window.inner_size().height),
                Msaa::X8,
                vec![
                    FramebufferAttachmentCreateInfo::new(
                        SizedTextureFormat::Rgba16f,
                        AttachmentType::Renderbuffer,
                    ),
                    FramebufferAttachmentCreateInfo::new(
                        SizedTextureFormat::Depth24Stencil8,
                        AttachmentType::Renderbuffer,
                    ),
                ],
            )
            .unwrap_or_else(|error| panic!("Framebuffer creation error: {}", error)),
        ];

        let resolve_framebuffer = Framebuffer::new(
            UVec2::new(window.inner_size().width, window.inner_size().height),
            Msaa::None,
            vec![
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
            //.with_effect(BloomBuilder::new(asset_path).build())
            .with_effect(ToneMapper::new())
            .build();

        let sampler_mipmap_linear = Sampler::new(
            MinificationFilter::LinearMipmapLinear,
            MagnificationFilter::Linear,
            WrappingMode::ClampToEdge,
            WrappingMode::ClampToEdge,
            WrappingMode::ClampToEdge,
            Vec4::new(0.0, 0.0, 0.0, 0.0),
            Anisotropy::X16,
        );

        let material = PbsMetallicRoughnessMaterial::new(
            asset_path,
            albedo,
            metallic_roughness_ao,
            normals,
            None,
        );

        let mut vertex_per_draw_ubo = Buffer::new(
            "Vertex Per Draw UBO",
            mem::size_of::<VertexPerDrawUniforms>() as isize,
            BufferTarget::Uniform,
            BufferStorageFlags::MAP_WRITE_PERSISTENT_COHERENT,
        );
        vertex_per_draw_ubo.bind(1);
        vertex_per_draw_ubo.map(MapModeFlags::MAP_WRITE_PERSISTENT_COHERENT);

        let mut fragment_per_frame_ubo = Buffer::new(
            "Fragment Per Frame UBO",
            std::mem::size_of::<FragmentPerFrameUniforms>() as isize,
            BufferTarget::Uniform,
            BufferStorageFlags::MAP_WRITE_PERSISTENT_COHERENT,
        );
        fragment_per_frame_ubo.bind(2);
        fragment_per_frame_ubo.map(MapModeFlags::MAP_WRITE_PERSISTENT_COHERENT);

        PbsScene {
            camera,
            model: Model {
                mesh,
                transform: Mat4::identity(),
            },
            material,
            environment: Environment {
                maps: environments,
                skybox_program_pipeline: skybox_prog,
                skybox_mesh,
                active_environment: 1,
                skybox_type: SkyboxType::Radiance,
            },
            msaa_framebuffers,
            resolve_framebuffer,
            sampler_linear: sampler_mipmap_linear,
            post_stack,
            controls: Controls {
                mouse_sensitivity: 2.0,
                ..Default::default()
            },
            lighting: Lighting {
                light_direction: [0.4, 0.0, -1.0],
                light_color: [1.0, 1.0, 1.0],
                light_intensity: 5.0,
                geometric_specular_aa: true,
                specular_ao: true,
                brdf_type: 0,
                multi_scattering: true,
                ss_variance_and_threshold: Vec2::new(0.25, 0.18),
                max_reflection_lod: 5,
            },
            render_mode: 0,
            vertex_per_draw_ubo,
            fragment_per_frame_ubo,
            msaa_framebuffer_index: 2,
        }
    }

    fn geometry_pass(&self) {
        let framebuffer = &self.msaa_framebuffers[self.msaa_framebuffer_index];
        framebuffer.bind();
        framebuffer.clear(&Vec4::new(0.0, 0.0, 0.0, 1.0));

        self.material.bind();

        let program_pipeline = self.material.program_pipeline();

        const IRRADIANCE_MAP_BINDING_INDEX: u32 = 4;
        const RADIANCE_MAP_BINDING_INDEX: u32 = 5;
        program_pipeline
            .set_texture_cube(
                IRRADIANCE_MAP_BINDING_INDEX,
                &self.environment.maps[self.environment.active_environment].irradiance,
                &self.sampler_linear,
            )
            .set_texture_cube(
                RADIANCE_MAP_BINDING_INDEX,
                &self.environment.maps[self.environment.active_environment].radiance,
                &self.sampler_linear,
            );

        self.model.mesh.draw();

        framebuffer.unbind(false);

        self.material.unbind()
    }

    fn skybox_pass(&self) {
        StateManager::set_depth_function(DepthFunction::LessOrEqual);
        StateManager::set_face_culling(FaceCulling::Front);

        let framebuffer = &self.msaa_framebuffers[self.msaa_framebuffer_index];

        framebuffer.bind();

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

        self.environment.skybox_program_pipeline.set_texture_cube(
            0,
            &environment_map,
            &self.sampler_linear,
        );

        self.environment.skybox_mesh.draw();

        self.environment.skybox_program_pipeline.unbind();

        StateManager::set_depth_function(DepthFunction::Less);
        StateManager::set_face_culling(FaceCulling::Back)
    }

    fn msaa_resolve(&self) {
        let framebuffer = &self.msaa_framebuffers[self.msaa_framebuffer_index];
        self.resolve_framebuffer
            .clear(&Vec4::new(0.0, 1.0, 0.0, 1.0));
        Framebuffer::blit(framebuffer, &self.resolve_framebuffer);
        framebuffer.unbind(true);
    }

    fn update_uniform_buffers(&self) {
        let vertex_per_draw_uniforms = VertexPerDrawUniforms {
            model_matrix: self.model.transform,
            normal_matrix: transpose(&inverse(&self.model.transform)),
        };

        self.vertex_per_draw_ubo
            .fill_mapped(0, &vertex_per_draw_uniforms);

        let mut light_color: Vec3 = srgb_to_linear3f(&self.lighting.light_color.into());
        light_color *= self.lighting.light_intensity;

        let fragment_per_frame_uniforms = FragmentPerFrameUniforms {
            light_direction: Vec4::new(
                self.lighting.light_direction[0],
                self.lighting.light_direction[1],
                self.lighting.light_direction[2],
                1.0,
            ),
            light_color: Vec4::new(light_color.x, light_color.y, light_color.z, 0.0),
            ss_variance_and_threshold: self.lighting.ss_variance_and_threshold.clone_owned(),
            geometric_specular_aa: self.lighting.geometric_specular_aa as i32,
            specular_ao: self.lighting.specular_ao as i32,
            render_mode: self.render_mode as i32,
            brdf_type: self.lighting.brdf_type as i32,
            multi_scattering: self.lighting.multi_scattering as i32,
            max_reflection_lod: self.lighting.max_reflection_lod as f32,
        };

        self.fragment_per_frame_ubo
            .fill_mapped(0, &fragment_per_frame_uniforms);
    }
}

impl Scene for PbsScene {
    fn start(&mut self, _: Context) {}

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
                //self.projection_matrix = perspective(x, y, 60, 0.5, 500.0);
                StateManager::set_viewport(0, 0, x as i32, y as i32)
            }
            _ => {}
        }
        Transition::None
    }

    fn update(&mut self, context: Context) -> Transition {
        let Context { timer, window, .. } = context;

        let mut dx = 0.0;
        let mut dy = 0.0;

        if self.controls.prev_x != 0.0 || self.controls.prev_y != 0.0 {
            dx = (self.controls.mouse_x - self.controls.prev_x) * self.controls.mouse_sensitivity;
            dy = (self.controls.mouse_y - self.controls.prev_y) * self.controls.mouse_sensitivity;
        }

        self.controls.prev_x = self.controls.mouse_x;
        self.controls.prev_y = self.controls.mouse_y;

        self.camera.update(
            window.inner_size(),
            dx,
            dy,
            self.controls.scroll,
            timer.delta_time(),
        );

        self.controls.scroll = 0.0;

        Transition::None
    }

    fn pre_draw(&mut self, _: Context) {
        self.update_uniform_buffers()
    }

    fn draw(&mut self, context: Context) {
        let Context {
            window,
            asset_manager,
            timer,
            framebuffer_cache,
            settings,
        } = context;

        self.geometry_pass();
        self.skybox_pass();
        self.msaa_resolve();

        if let Some(tone_mapper) = self.post_stack.get_mut::<ToneMapper>() {
            tone_mapper.set_exposure(self.camera.exposure())
        }

        self.post_stack.apply(
            &self.resolve_framebuffer,
            Context::new(window, asset_manager, timer, framebuffer_cache, settings),
        );

        self.resolve_framebuffer.unbind(true);
    }

    fn gui(&mut self, context: Context, ui: &Ui) {
        imgui::Window::new(im_str!("Inspector"))
            .size([358.0, 500.0], Condition::Appearing)
            .position([2.0, 0.0], Condition::Appearing)
            .mouse_inputs(true)
            .resizable(true)
            .movable(false)
            .build(ui, || {
                ui.dummy([358.0, 0.0]);

                imgui::ComboBox::new(im_str!("Render Mode")).build_simple_string(
                    ui,
                    &mut self.render_mode,
                    &[
                        im_str!("Lit"),
                        im_str!("Albedo"),
                        im_str!("Metallic"),
                        im_str!("Roughness"),
                        im_str!("Normals"),
                        im_str!("Tangents"),
                        im_str!("UV"),
                        im_str!("NdotV"),
                        im_str!("AO"),
                        im_str!("Specular AO"),
                        im_str!("Horizon Specular AO"),
                        im_str!("Diffuse Ambient"),
                        im_str!("Specular Ambient"),
                        im_str!("Fresnel"),
                        im_str!("Fresnel * Radiance"),
                        im_str!("Analytical Lights Only"),
                        im_str!("IBL only"),
                    ],
                );

                ui.spacing();

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
                    imgui::TreeNode::new(im_str!("Analytical"))
                        .default_open(true)
                        .open_on_arrow(true)
                        .open_on_double_click(true)
                        .framed(false)
                        .build(ui, || {
                            imgui::TreeNode::new(im_str!("Directional Light"))
                                .default_open(true)
                                .open_on_arrow(true)
                                .open_on_double_click(true)
                                .framed(false)
                                .build(ui, || {
                                    imgui::Drag::new(im_str!("Light Direction"))
                                        .range(RangeInclusive::new(-1.0, 1.0))
                                        .display_format(im_str!("%.2f"))
                                        .speed(0.01)
                                        .build_array(ui, &mut self.lighting.light_direction);

                                    imgui::ColorEdit::new(
                                        im_str!("Light Color"),
                                        &mut self.lighting.light_color,
                                    )
                                    .format(ColorFormat::Float)
                                    .options(true)
                                    .picker(true)
                                    .alpha(false)
                                    .build(&ui);
                                    imgui::Slider::new(im_str!("Light Intensity"))
                                        .range(RangeInclusive::new(0.01, 300.0))
                                        .display_format(im_str!("%.1f"))
                                        .build(&ui, &mut self.lighting.light_intensity);
                                });

                            imgui::TreeNode::new(im_str!("BRDF"))
                                .default_open(true)
                                .open_on_arrow(true)
                                .open_on_double_click(true)
                                .framed(false)
                                .build(ui, || {
                                    imgui::ComboBox::new(im_str!("BRDF Type")).build_simple_string(
                                        ui,
                                        &mut self.lighting.brdf_type,
                                        &[im_str!("Fillament"), im_str!("Unreal Engine 4")],
                                    );

                                    if self.lighting.brdf_type == 0 {
                                        ui.checkbox(
                                            im_str!("Multi-Scattering"),
                                            &mut self.lighting.multi_scattering,
                                        );
                                    }

                                    ui.checkbox(
                                        im_str!("##geomspecaa"),
                                        &mut self.lighting.geometric_specular_aa,
                                    );
                                    ui.same_line(72.0);
                                    imgui::TreeNode::new(im_str!("Geometric Specular AA"))
                                        .default_open(true)
                                        .open_on_arrow(true)
                                        .open_on_double_click(true)
                                        .framed(false)
                                        .build(ui, || {
                                            ui.indent();
                                            imgui::Slider::new(im_str!("Screen Space Variance"))
                                                .range(RangeInclusive::new(0.01, 1.0))
                                                .display_format(im_str!("%.2f"))
                                                .build(
                                                    &ui,
                                                    &mut self.lighting.ss_variance_and_threshold.x,
                                                );
                                            imgui::Slider::new(im_str!("Threshold"))
                                                .range(RangeInclusive::new(0.01, 1.0))
                                                .display_format(im_str!("%.2f"))
                                                .build(
                                                    &ui,
                                                    &mut self.lighting.ss_variance_and_threshold.y,
                                                );
                                            ui.unindent()
                                        });
                                });
                        });

                    imgui::TreeNode::new(im_str!("Image-Based"))
                        .default_open(true)
                        .open_on_arrow(true)
                        .open_on_double_click(true)
                        .framed(false)
                        .build(ui, || {
                            ui.checkbox(im_str!("Specular AO"), &mut self.lighting.specular_ao);

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

                            imgui::Slider::new(im_str!("Max reflection LOD"))
                                .range(RangeInclusive::new(1, 9))
                                .build(&ui, &mut self.lighting.max_reflection_lod);
                        });
                }

                if imgui::CollapsingHeader::new(im_str!("Anti-Aliasing"))
                    .default_open(true)
                    .open_on_arrow(true)
                    .open_on_double_click(true)
                    .build(ui)
                {
                    imgui::ComboBox::new(im_str!("MSAA")).build_simple_string(
                        ui,
                        &mut self.msaa_framebuffer_index,
                        &[im_str!("X1"), im_str!("X2"), im_str!("X4"), im_str!("X8")],
                    );
                }

                ui.spacing();

                // Camera
                self.camera.gui(ui);

                ui.spacing();

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
