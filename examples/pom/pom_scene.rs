use std::rc::Rc;

use engine::{
    application::clear_default_framebuffer,
    camera::Camera,
    math::{
        matrix::{perspective, Mat4},
        vector::{UVec2, Vec3, Vec4},
    },
    rendering::{
        framebuffer::{AttachmentType, Framebuffer, FramebufferAttachmentCreateInfo},
        material::{Material, PbsMetallicRoughnessMaterial},
        mesh::{FullscreenMesh, Mesh, MeshUtilities},
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

#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq)]
enum ParallaxMappingMethod {
    None = 0,
    ParallaxMapping = 1,
    ParallaxMappingOffsetLimiting = 2,
    SteepParallaxMapping = 3,
    ParallaxOcclusionMapping = 4,
}

pub struct PomScene {
    camera: Camera,
    cube_mesh: Mesh,
    fullscreen_mesh: FullscreenMesh,
    material: Box<dyn Material>,
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
    left_mouse_button_pressed: bool,
    mouse_x: f32,
    mouse_y: f32,
    mouse_sensitivity: f32,
    scroll: f32,
    prev_x: f32,
    prev_y: f32,
    dt: f32,
    parallax_mapping_method: ParallaxMappingMethod,
}

impl PomScene {
    pub fn new(context: Context) -> Self {
        let Context {
            window,
            settings,
            asset_manager,
            ..
        } = context;

        let asset_path = settings.asset_path.as_path();
        let camera = Camera::new(
            Vec3::new(0.0, 0.0, -2.0),
            Vec3::new(0.0, 0.0, 0.0),
            10.0,
            30.0,
            1.0,
            2.0,
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

        let cube_mesh = MeshUtilities::generate_cube(1.0);

        let albedo = asset_manager
            .load_texture_2d(
                asset_path.join("textures/pbs/castle_brick/castle_brick_albedo.png"),
                true,
                true,
            )
            .expect("Failed to load albedo texture");

        let metallic_roughness_ao = asset_manager
            .load_texture_2d(
                asset_path.join("textures/pbs/castle_brick/castle_brick_m_r_ao.png"),
                false,
                true,
            )
            .expect("Failed to load metallic/roughness/ao texture");

        let normals = asset_manager
            .load_texture_2d(
                asset_path.join("textures/pbs/castle_brick/castle_brick_normals.png"),
                false,
                true,
            )
            .expect("Failed to load normals texture");

        let displacement = asset_manager
            .load_texture_2d(
                asset_path.join("textures/pbs/castle_brick/castle_brick_displacement.png"),
                false,
                true,
            )
            .expect("Failed to load displacement texture");

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
                UVec2::new(window.inner_size().width / 4, window.inner_size().width / 4),
                Msaa::None,
                vec![FramebufferAttachmentCreateInfo::new(
                    SizedTextureFormat::Rgba16f,
                    AttachmentType::Texture,
                )],
            )
            .unwrap_or_else(|error| panic!("Framebuffer creation error: {}", error)),
            Framebuffer::new(
                UVec2::new(window.inner_size().width / 4, window.inner_size().width / 4),
                Msaa::None,
                vec![FramebufferAttachmentCreateInfo::new(
                    SizedTextureFormat::Rgba16f,
                    AttachmentType::Texture,
                )],
            )
            .unwrap_or_else(|error| panic!("Framebuffer creation error: {}", error)),
        ];

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
            0.1,
            500.0,
        );

        let material = Box::new(PbsMetallicRoughnessMaterial::new(
            asset_path,
            albedo.clone(),
            metallic_roughness_ao.clone(),
            normals.clone(),
            Some(displacement.clone()),
            ibl_brdf_lut.clone(),
        ));

        PomScene {
            camera,
            cube_mesh,
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
            left_mouse_button_pressed: false,
            mouse_x: 0.0,
            mouse_y: 0.0,
            mouse_sensitivity: 2.0,
            scroll: 0.0,
            prev_x: 0.0,
            prev_y: 0.0,
            dt: 0.0,
            parallax_mapping_method: ParallaxMappingMethod::ParallaxOcclusionMapping,
        }
    }

    fn geometry_pass(&self) {
        self.framebuffer.bind();
        self.framebuffer.clear(&Vec4::new(0.0, 0.0, 0.0, 1.0));

        self.material.bind();

        let program_pipeline = self.material.program_pipeline();

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
            .set_vector3f(
                "lightColor",
                &Vec3::new(1.0, 1.0, 1.0),
                ShaderStage::Fragment,
            )
            .set_matrix4f("model", &Mat4::identity(), ShaderStage::Vertex)
            .set_matrix4f("view", &self.camera.get_transform(), ShaderStage::Vertex)
            .set_vector3f(
                "eyePosition",
                &self.camera.get_position(),
                ShaderStage::Vertex,
            )
            .set_matrix4f("projection", &self.projection_matrix, ShaderStage::Vertex)
            .set_integer(
                "parallaxMappingMethod",
                self.parallax_mapping_method as i32,
                ShaderStage::Fragment,
            );

        self.cube_mesh.draw();

        self.framebuffer.unbind(false);
        self.material.unbind();

        Framebuffer::blit(&self.framebuffer, &self.resolve_framebuffer)
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
            &self.camera.get_transform(),
            ShaderStage::Vertex,
        );

        self.cube_mesh.draw();

        self.resolve_framebuffer.unbind(false);
        self.skybox_program_pipeline.unbind();

        StateManager::set_depth_function(DepthFunction::Less);
        StateManager::set_face_culling(FaceCulling::Back)
    }

    pub fn tonemap_pass(&self) {
        clear_default_framebuffer(&Vec4::new(0.0, 1.0, 0.0, 1.0));

        self.tonemapping_pipeline.bind();

        let exposure: f32 = 1.5;
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
            .set_float("exposure", exposure, ShaderStage::Fragment);

        StateManager::set_front_face(FrontFace::Clockwise);
        self.fullscreen_mesh.draw();
        StateManager::set_front_face(FrontFace::CounterClockwise);

        self.tonemapping_pipeline.unbind()
    }
}

impl Scene for PomScene {
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
                if self.left_mouse_button_pressed {
                    self.mouse_x = position.x as f32;
                    self.mouse_y = position.y as f32;
                }
            }
            WindowEvent::MouseWheel {
                delta: MouseScrollDelta::LineDelta(_, y),
                ..
            } => {
                self.scroll = y;
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
                        virtual_keycode: Some(VirtualKeyCode::Key1),
                        ..
                    },
                ..
            } => self.parallax_mapping_method = ParallaxMappingMethod::None,
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state: ElementState::Released,
                        virtual_keycode: Some(VirtualKeyCode::Key2),
                        ..
                    },
                ..
            } => self.parallax_mapping_method = ParallaxMappingMethod::ParallaxMapping,
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state: ElementState::Released,
                        virtual_keycode: Some(VirtualKeyCode::Key3),
                        ..
                    },
                ..
            } => {
                self.parallax_mapping_method = ParallaxMappingMethod::ParallaxMappingOffsetLimiting
            }
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state: ElementState::Released,
                        virtual_keycode: Some(VirtualKeyCode::Key4),
                        ..
                    },
                ..
            } => self.parallax_mapping_method = ParallaxMappingMethod::SteepParallaxMapping,
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state: ElementState::Released,
                        virtual_keycode: Some(VirtualKeyCode::Key5),
                        ..
                    },
                ..
            } => self.parallax_mapping_method = ParallaxMappingMethod::ParallaxOcclusionMapping,
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
                        state: ElementState::Released,
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
        self.bloom_pass();
        self.skybox_pass();
        self.tonemap_pass();
    }

    fn post_draw(&mut self, _: Context) {}
}
