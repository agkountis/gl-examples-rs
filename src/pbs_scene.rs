use std::rc::Rc;

use pbs_engine::application::clear_default_framebuffer;
use pbs_engine::camera::Camera;
use pbs_engine::core::engine::event::Event;
use pbs_engine::core::engine::input::Modifiers;
use pbs_engine::core::scene::Transition;
use pbs_engine::engine::{input, Context};
use pbs_engine::math::{
    matrix::{perspective, rotate, Mat4},
    scale,
    vector::{UVec2, Vec3, Vec4},
};
use pbs_engine::rendering::mesh::{FullscreenMesh, Mesh, MeshUtilities};
use pbs_engine::rendering::program_pipeline::ProgramPipeline;
use pbs_engine::rendering::{
    framebuffer::{AttachmentType, Framebuffer, FramebufferAttachmentCreateInfo},
    material::{Material, PbsMetallicRoughnessMaterial},
    sampler::{MagnificationFilter, MinificationFilter, Sampler, WrappingMode},
    shader::{Shader, ShaderStage},
    state::{DepthFunction, FaceCulling, FrontFace, StateManager},
    texture::{SizedTextureFormat, TextureCube},
    Draw,
};
use pbs_engine::scene::Scene;

use crate::ApplicationData;

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
    material: Box<dyn Material>,
    environment_maps: EnvironmentMaps,
    skybox_program_pipeline: ProgramPipeline,
    horizontal_gaussian_pipeline: ProgramPipeline,
    vertical_gaussian_pipeline: ProgramPipeline,
    tonemapping_pipeline: ProgramPipeline,
    framebuffer: Framebuffer,
    blur_framebuffers: [Framebuffer; 2],
    default_fb_size: UVec2,
    sampler: Sampler,
    sampler_nearest: Sampler,
    projection_matrix: Mat4,
    left_mouse_button_pressed: bool,
    mouse_x: f32,
    mouse_y: f32,
    scroll: f32,
    prev_x: f32,
    prev_y: f32,
    dt: f32,
}

impl PbsScene {
    pub fn new(context: Context<ApplicationData>) -> Self {
        let window = context.window;

        let asset_manager = context.asset_manager;

        let camera = Camera::default();

        let skybox_prog = ProgramPipeline::new()
            .add_shader(&Shader::new_from_text(ShaderStage::Vertex, "sdr/skybox.vert").unwrap())
            .add_shader(&Shader::new_from_text(ShaderStage::Fragment, "sdr/skybox.frag").unwrap())
            .build()
            .unwrap();

        let fullscreen_shader =
            Shader::new_from_text(ShaderStage::Vertex, "sdr/fullscreen.vert").unwrap();
        let horizontal_gaussian_prog = ProgramPipeline::new()
            .add_shader(&fullscreen_shader)
            .add_shader(
                &Shader::new_from_text(ShaderStage::Fragment, "sdr/gaussian_blur_horizontal.frag")
                    .unwrap(),
            )
            .build()
            .unwrap();

        let vertical_gaussian_prog = ProgramPipeline::new()
            .add_shader(&fullscreen_shader)
            .add_shader(
                &Shader::new_from_text(ShaderStage::Fragment, "sdr/gaussian_blur_vertical.frag")
                    .unwrap(),
            )
            .build()
            .unwrap();

        let tonemapping_prog = ProgramPipeline::new()
            .add_shader(&fullscreen_shader)
            .add_shader(&Shader::new_from_text(ShaderStage::Fragment, "sdr/tonemap.frag").unwrap())
            .build()
            .unwrap();

        let mesh = asset_manager
            .load_mesh("assets/models/cerberus/cerberus.fbx")
            .expect("Failed to load mesh");

        let skybox_mesh = MeshUtilities::generate_cube(1.0);

        let albedo = asset_manager
            .load_texture_2d("assets/textures/cerberus/Cerberus_A.png", true, true)
            .expect("Failed to load albedo texture");

        let metallic = asset_manager
            .load_texture_2d("assets/textures/cerberus/Cerberus_M.png", false, true)
            .expect("Failed to load metallic texture");

        let roughness = asset_manager
            .load_texture_2d("assets/textures/cerberus/Cerberus_R.png", false, true)
            .expect("Failed to load roughness texture");

        let normals = asset_manager
            .load_texture_2d("assets/textures/cerberus/Cerberus_N.png", false, true)
            .expect("Failed to load normals texture");

        let ao = asset_manager
            .load_texture_2d("assets/textures/cerberus/Cerberus_AO.png", false, true)
            .expect("Failed to load ao texture");

        let ibl_brdf_lut = asset_manager
            .load_texture_2d("assets/textures/pbs/ibl_brdf_lut.png", false, false)
            .expect("Failed to load BRDF LUT texture");

        let skybox = TextureCube::new_from_file("assets/textures/pbs/ktx/skybox/ibl_skybox.ktx")
            .expect("Failed to load Skybox");

        let irradiance =
            TextureCube::new_from_file("assets/textures/pbs/ktx/irradiance/ibl_irradiance.ktx")
                .expect("Failed to load Irradiance map");

        let radiance =
            TextureCube::new_from_file("assets/textures/pbs/ktx/radiance/ibl_radiance.ktx")
                .expect("Failed to load Radiance map");

        let framebuffer = Framebuffer::new(
            UVec2::new(
                window.get_framebuffer_width(),
                window.get_framebuffer_height(),
            ),
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

        let blur_framebuffers: [Framebuffer; 2] = [
            Framebuffer::new(
                UVec2::new(
                    window.get_framebuffer_width() / 4,
                    window.get_framebuffer_height() / 4,
                ),
                vec![FramebufferAttachmentCreateInfo::new(
                    SizedTextureFormat::Rgba16f,
                    AttachmentType::Texture,
                )],
            )
            .unwrap_or_else(|error| panic!("Framebuffer creation error: {}", error)),
            Framebuffer::new(
                UVec2::new(
                    window.get_framebuffer_width() / 4,
                    window.get_framebuffer_height() / 4,
                ),
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
            window.get_framebuffer_width(),
            window.get_framebuffer_height(),
            60,
            0.1,
            500.0,
        );

        let material = Box::new(PbsMetallicRoughnessMaterial::new(
            albedo.clone(),
            metallic.clone(),
            roughness.clone(),
            normals.clone(),
            ao.clone(),
            ibl_brdf_lut.clone(),
        ));

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
            blur_framebuffers,
            default_fb_size: UVec2::new(
                window.get_framebuffer_width(),
                window.get_framebuffer_height(),
            ),
            sampler,
            sampler_nearest,
            projection_matrix: projection,
            left_mouse_button_pressed: false,
            mouse_x: 0.0,
            mouse_y: 0.0,
            scroll: 0.0,
            prev_x: 0.0,
            prev_y: 0.0,
            dt: 0.0,
        }
    }

    fn geometry_pass(&self, context: Context<ApplicationData>) {
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
                &Vec3::new(5.0, 5.0, 5.0),
                ShaderStage::Fragment,
            )
            .set_matrix4f("model", &self.model.transform, ShaderStage::Vertex)
            .set_matrix4f("view", &self.camera.get_transform(), ShaderStage::Vertex)
            .set_vector3f(
                "eyePosition",
                &self.camera.get_position(),
                ShaderStage::Vertex,
            )
            .set_matrix4f("projection", &self.projection_matrix, ShaderStage::Vertex);

        self.model.mesh.draw();

        self.framebuffer.unbind(false);
        self.material.unbind()
    }

    fn bloom_pass(&self, context: Context<ApplicationData>) {
        let blur_strength = 6;

        let size = self.blur_framebuffers[0].get_size();
        StateManager::set_viewport(0, 0, size.x as i32, size.y as i32);
        for i in 0..blur_strength {
            let ping_pong_index = i % 2;

            let mut attachment_id: u32 = 0;
            if ping_pong_index == 0 {
                self.blur_framebuffers[ping_pong_index].bind();
                self.vertical_gaussian_pipeline.bind();

                if i == 0 {
                    attachment_id = self.framebuffer.get_texture_attachment(1).get_id();
                } else {
                    attachment_id = self.blur_framebuffers[1 - ping_pong_index]
                        .get_texture_attachment(0)
                        .get_id();
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
                    .get_texture_attachment(0)
                    .get_id();
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

    fn skybox_pass(&self, context: Context<ApplicationData>) {
        let Context { window, .. } = context;

        StateManager::set_depth_function(DepthFunction::LessOrEqual);
        StateManager::set_face_culling(FaceCulling::Front);
        StateManager::set_viewport(0, 0, window.get_width() as i32, window.get_height() as i32);

        self.framebuffer.bind();

        self.skybox_program_pipeline.bind();

        self.skybox_program_pipeline.set_matrix4f(
            "view",
            &self.camera.get_transform(),
            ShaderStage::Vertex,
        );

        self.skybox_mesh.draw();

        self.framebuffer.unbind(true);
        self.skybox_program_pipeline.unbind();

        StateManager::set_depth_function(DepthFunction::Less);
        StateManager::set_face_culling(FaceCulling::Back)
    }

    pub fn tonemap_pass(&self, context: Context<ApplicationData>) {
        clear_default_framebuffer(&Vec4::new(0.0, 1.0, 0.0, 1.0));

        self.tonemapping_pipeline.bind();

        let exposure: f32 = 1.5;
        self.tonemapping_pipeline
            .set_texture_2d_with_id(
                "image",
                self.framebuffer.get_texture_attachment(0).get_id(),
                &self.sampler_nearest,
                ShaderStage::Fragment,
            )
            .set_texture_2d_with_id(
                "bloomImage",
                self.blur_framebuffers[1].get_texture_attachment(0).get_id(),
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

impl Scene<ApplicationData> for PbsScene {
    fn start(&mut self, context: Context<ApplicationData>) {
        self.model.transform = {
            let tx = rotate(&Mat4::identity(), -90.0, &Vec3::new(1.0, 0.0, 0.0));
            scale(&tx, &Vec3::new(0.2, 0.2, 0.2))
        };

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

    fn stop(&mut self, context: Context<ApplicationData>) {}

    fn pause(&mut self, context: Context<ApplicationData>) {}

    fn resume(&mut self, context: Context<ApplicationData>) {}

    fn handle_event(
        &mut self,
        context: Context<ApplicationData>,
        event: Event,
    ) -> Transition<ApplicationData> {
        let Context {
            window,
            asset_manager,
            timer,
            settings,
            user_data,
        } = context;

        match event {
            Event::MouseButton(button, action, modifiers) => {
                println!("{:?} : {:?}", button, action);
                match button {
                    input::MouseButton::Left => match action {
                        input::Action::Press => self.left_mouse_button_pressed = true,
                        input::Action::Release => {
                            self.left_mouse_button_pressed = false;
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }
            Event::CursorPosition(x, y) => {
                if self.left_mouse_button_pressed {
                    self.mouse_x = x as f32;
                    self.mouse_y = y as f32;
                }
            }
            Event::Scroll(x, y) => {
                self.scroll = y as f32; //maybe accumulate?
                println!("Scroll: {}, {}", x, y)
            }
            Event::Key(key, action, m) => {
                if m.intersects(Modifiers::Shift) {
                    println!("Shift + {:?} : {:?}", key, action)
                } else {
                    println!("{:?} : {:?}", key, action);
                    match key {
                        input::Key::Escape => return Transition::Quit,
                        _ => (),
                    }
                }
            }
            Event::WindowFramebufferSize(x, y) => {
                println!("Framebuffer size: {}, {}", x, y);
                self.projection_matrix = perspective(x as u32, y as u32, 60, 0.1, 100.0);
                StateManager::set_viewport(0, 0, x, y)
            }
            _ => (),
        }
        Transition::None
    }

    fn update(&mut self, context: Context<ApplicationData>) -> Transition<ApplicationData> {
        let Context {
            window,
            asset_manager,
            timer,
            settings,
            user_data,
        } = context;

        self.dt = timer.get_delta();
        let dx = self.mouse_x - self.prev_x;
        let dy = self.mouse_y - self.prev_y;

        //        println!("dx: {} dy: {}", dx, dy);

        self.prev_x = self.mouse_x;
        self.prev_y = self.mouse_y;

        self.camera.update(dx, dy, self.scroll, self.dt);

        self.scroll = 0.0;

        Transition::None
    }

    fn pre_draw(&mut self, context: Context<ApplicationData>) {}

    fn draw(&mut self, context: Context<ApplicationData>) {
        let Context {
            window,
            asset_manager,
            timer,
            settings,
            user_data,
        } = context;

        self.geometry_pass(Context::new(
            window,
            asset_manager,
            timer,
            settings,
            user_data,
        ));
        self.bloom_pass(Context::new(
            window,
            asset_manager,
            timer,
            settings,
            user_data,
        ));
        self.skybox_pass(Context::new(
            window,
            asset_manager,
            timer,
            settings,
            user_data,
        ));
        self.tonemap_pass(Context::new(
            window,
            asset_manager,
            timer,
            settings,
            user_data,
        ));
    }

    fn post_draw(&mut self, context: Context<ApplicationData>) {}
}
