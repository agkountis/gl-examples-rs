use pbs_engine::scene::Scene;
use pbs_engine::camera::Camera;
use pbs_engine::rendering::mesh::{Mesh, FullscreenMesh, MeshUtilities};
use pbs_engine::rendering::program_pipeline::ProgramPipeline;
use pbs_engine::math::{
    vector::{Vec3, UVec2, Vec4},
    matrix::{perspective, Mat4, rotate, translate}
};

use pbs_engine::rendering::{
    shader::{ShaderStage, Shader},
    framebuffer::{Framebuffer, FramebufferAttachmentCreateInfo, AttachmentType},
    texture::{SizedTextureFormat, TextureCube},
    sampler::{Sampler, MinificationFilter, MagnificationFilter, WrappingMode},
    material::{Material, PbsMetallicRoughnessMaterial},
    state::{StateManager, DepthFunction, FaceCulling, FrontFace},
    Draw
};


use pbs_engine::window::Window;

use pbs_engine::application::clear_default_framebuffer;


use pbs_engine::core::asset::AssetManager;
use pbs_engine::core::engine::event::Event;
use pbs_engine::engine::Context;
use pbs_engine::core::scene::Transition;
use crate::ApplicationData;
use pbs_engine::core::engine::input::Key::P;


struct EnvironmentMaps {
    pub skybox: TextureCube,
    pub irradiance: TextureCube,
    pub radiance: TextureCube
}

struct Model {
    pub mesh: Mesh,
    pub transform: Mat4
}

pub struct PbsScene {
    a: i32
//    camera: Camera,
//    model: Model,
//    skybox_mesh: Mesh,
//    fullscreen_mesh: FullscreenMesh,
//    material: Box<dyn Material>,
//    environment_maps: EnvironmentMaps,
//    skybox_program_pipeline: ProgramPipeline,
//    horizontal_gaussian_pipeline: ProgramPipeline,
//    vertical_gaussian_pipeline: ProgramPipeline,
//    tonemapping_pipeline: ProgramPipeline,
//    framebuffer: Framebuffer,
//    blur_framebuffers: [Framebuffer; 2],
//    default_fb_size: UVec2,
//    sampler: Sampler,
//    sampler_nearest: Sampler,
//    projection_matrix: Mat4,
//    asset_manager: AssetManager
}

impl PbsScene {
    pub fn new(a: i32) -> Self {
        Self {
            a
        }
    }
//    pub fn new(window: &Window) -> Self {
//        let mut camera = Camera::new();
//        camera.look_at(Vec3::new(0.0, 0.0, 0.0),
//                       Vec3::new(0.0, 0.0, 1.0),
//                       Vec3::new(0.0, 1.0, 0.0));
//
//        let mut asset_manager = AssetManager::new();
//
//        let skybox_prog = ProgramPipeline::new()
//            .add_shader(&Shader::new_from_text(ShaderStage::Vertex,
//                                               "sdr/skybox.vert").unwrap())
//            .add_shader(&Shader::new_from_text(ShaderStage::Fragment,
//                                               "sdr/skybox.frag").unwrap())
//            .build()
//            .unwrap();
//
//        let fullscreen_shader = Shader::new_from_text(ShaderStage::Vertex,
//                                                      "sdr/fullscreen.vert").unwrap();
//        let horizontal_gaussian_prog = ProgramPipeline::new()
//            .add_shader(&fullscreen_shader)
//            .add_shader(&Shader::new_from_text(ShaderStage::Fragment,
//                                               "sdr/gaussian_blur_horizontal.frag").unwrap())
//            .build()
//            .unwrap();
//
//        let vertical_gaussian_prog = ProgramPipeline::new().add_shader(&fullscreen_shader)
//            .add_shader(&Shader::new_from_text(ShaderStage::Fragment,
//                                               "sdr/gaussian_blur_vertical.frag").unwrap())
//            .build()
//            .unwrap();
//
//        let tonemapping_prog = ProgramPipeline::new().add_shader(&fullscreen_shader)
//            .add_shader(&Shader::new_from_text(ShaderStage::Fragment,
//                                               "sdr/tonemap.frag").unwrap())
//            .build()
//            .unwrap();
//
//
//        let mesh = MeshUtilities::generate_cube(1.0);
//        let skybox_mesh = MeshUtilities::generate_cube(1.0);
//
//        let albedo = asset_manager.load_texture_2d("assets/textures/pbs/rusted_iron/albedo.png", true, true)
//            .expect("Failed to load albedo texture");
//
//        let metallic = asset_manager.load_texture_2d("assets/textures/pbs/rusted_iron/metallic.png", false, true)
//            .expect("Failed to load metallic texture");
//
//        let roughness = asset_manager.load_texture_2d("assets/textures/pbs/rusted_iron/roughness.png", false, true)
//            .expect("Failed to load roughness texture");
//
//        let normals = asset_manager.load_texture_2d("assets/textures/pbs/rusted_iron/normal.png", false, true)
//            .expect("Failed to load normals texture");
//
//        let ao = asset_manager.load_texture_2d("assets/textures/pbs/rusted_iron/ao.png", false, true)
//            .expect("Failed to load ao texture");
//
//        let ibl_brdf_lut = asset_manager.load_texture_2d("assets/textures/pbs/ibl_brdf_lut.png", false, false)
//            .expect("Failed to load BRDF LUT texture");
//
//        let skybox = TextureCube::new_from_file("assets/textures/pbs/ktx/skybox/ibl_skybox.ktx")
//            .expect("Failed to load Skybox");
//
//        let irradiance = TextureCube::new_from_file("assets/textures/pbs/ktx/irradiance/ibl_irradiance.ktx")
//            .expect("Failed to load Irradiance map");
//
//        let radiance = TextureCube::new_from_file("assets/textures/pbs/ktx/radiance/ibl_radiance.ktx")
//            .expect("Failed to load Radiance map");
//
//        let framebuffer = Framebuffer::new(UVec2::new(window.get_framebuffer_width(),
//                                                      window.get_framebuffer_height()),
//                                  vec![
//                                      FramebufferAttachmentCreateInfo::new(SizedTextureFormat::Rgba16f,
//                                                                           AttachmentType::Texture),
//                                      FramebufferAttachmentCreateInfo::new(SizedTextureFormat::Rgba16f,
//                                                                           AttachmentType::Texture),
//                                      FramebufferAttachmentCreateInfo::new(SizedTextureFormat::Depth24Stencil8,
//                                                                           AttachmentType::Renderbuffer)
//                                  ])
//            .unwrap_or_else(|error| {
//                panic!("Framebuffer creation error: {}", error)
//            });
//
//        let blur_framebuffers: [Framebuffer; 2] =
//            [ Framebuffer::new(UVec2::new(window.get_framebuffer_width() / 4,
//                                          window.get_framebuffer_height() / 4),
//                               vec![
//                                   FramebufferAttachmentCreateInfo::new(SizedTextureFormat::Rgba16f,
//                                                                        AttachmentType::Texture)])
//                .unwrap_or_else(|error| {panic!("Framebuffer creation error: {}", error)}),
//                Framebuffer::new(UVec2::new(window.get_framebuffer_width() / 4,
//                                            window.get_framebuffer_height() / 4),
//                                 vec![
//                                     FramebufferAttachmentCreateInfo::new(SizedTextureFormat::Rgba16f,
//                                                                          AttachmentType::Texture)])
//                    .unwrap_or_else(|error| {
//                        panic!("Framebuffer creation error: {}", error)
//                    }) ];
//
//
//        let sampler = Sampler::new(MinificationFilter::LinearMipmapLinear,
//                                   MagnificationFilter::Linear,
//                                   WrappingMode::ClampToEdge,
//                                   WrappingMode::ClampToEdge,
//                                   WrappingMode::ClampToEdge,
//                                   Vec4::new(0.0, 0.0, 0.0, 0.0));
//
//        let sampler_nearest = Sampler::new(MinificationFilter::Nearest,
//                                           MagnificationFilter::Nearest,
//                                           WrappingMode::ClampToEdge,
//                                           WrappingMode::ClampToEdge,
//                                           WrappingMode::ClampToEdge,
//                                           Vec4::new(0.0, 0.0, 0.0, 0.0));
//
//        let projection = perspective(window.get_framebuffer_width(),
//                                     window.get_framebuffer_height(),
//                                                      60,
//                                                      0.1,
//                                                      100.0);

//        PbsScene {
//            camera,
//            model: Model{
//                mesh,
//                transform: translate(&Mat4::identity(), &Vec3::new(0.0, 0.0, 2.0))
//            },
//            skybox_mesh,
//            fullscreen_mesh: FullscreenMesh::new(),
//            material: Box::new(PbsMetallicRoughnessMaterial::new(
//                albedo,
//                metallic,
//                roughness,
//                normals,
//                ao,
//                ibl_brdf_lut
//            )),
//            environment_maps: EnvironmentMaps {
//                skybox,
//                irradiance,
//                radiance
//            },
//            skybox_program_pipeline: skybox_prog,
//            horizontal_gaussian_pipeline: horizontal_gaussian_prog,
//            vertical_gaussian_pipeline: vertical_gaussian_prog,
//            tonemapping_pipeline: tonemapping_prog,
//            framebuffer,
//            blur_framebuffers,
//            default_fb_size: UVec2::new(window.get_framebuffer_width(),
//                                        window.get_framebuffer_height()),
//            sampler,
//            sampler_nearest,
//            projection_matrix: projection,
//            asset_manager
//        }
//    }

//    fn geometry_pass(&self) {
//        self.framebuffer.bind();
//        self.framebuffer.clear(&Vec4::new(0.0, 0.0, 0.0, 1.0));
////        self.geometry_program_pipeline.bind();
////
////        self.geometry_program_pipeline
////            .set_texture_2d("albedoMap",
////                            &self.material.albedo,
////                            &self.sampler,
////                            ShaderStage::Fragment)
////            .set_texture_2d("metallicMap",
////                            &self.material.metallic,
////                            &self.sampler,
////                            ShaderStage::Fragment)
////            .set_texture_2d("roughnessMap",
////                            &self.material.roughness,
////                            &self.sampler,
////                            ShaderStage::Fragment)
////            .set_texture_2d("normalMap",
////                            &self.material.normals,
////                            &self.sampler,
////                            ShaderStage::Fragment)
////            .set_texture_2d("aoMap",
////                            &self.material.ao,
////                            &self.sampler,
////                            ShaderStage::Fragment)
////            .set_texture_2d("brdfLUT",
////                            &self.material.ibl_brdf_lut,
////                            &self.sampler,
////                            ShaderStage::Fragment)
////            .set_texture_cube("irradianceMap",
////                              &self.environment_maps.irradiance,
////                              &self.sampler,
////                              ShaderStage::Fragment)
////            .set_texture_cube("radianceMap",
////                              &self.environment_maps.radiance,
////                              &self.sampler,
////                              ShaderStage::Fragment)
////            .set_vector3f("wLightDirection",
////                          &Vec3::new(-0.2, 0.0, -1.0),
////                          ShaderStage::Fragment)
////            .set_vector3f("lightColor",
////                          &Vec3::new(3.0, 3.0, 3.0),
////                          ShaderStage::Fragment)
////            .set_matrix4f("model",
////                          &self.model.transform,
////                          ShaderStage::Vertex)
////            .set_matrix4f("view",
////                          &self.camera.get_transform(),
////                          ShaderStage::Vertex)
////            .set_vector3f("eyePosition",
////                          &self.camera.get_position(),
////                          ShaderStage::Vertex)
////            .set_matrix4f("projection",
////                          &self.projection_matrix,
////                          ShaderStage::Vertex);
//
//        self.model.mesh.draw();
//
//        self.framebuffer.unbind(false);
//        self.geometry_program_pipeline.unbind()
//    }

//    fn bloom_pass(&self) {
//        let blur_strength = 10;
//
//        for i in 0..blur_strength {
//            let ping_pong_index = i % 2;
//
//            let mut attachment_id: u32 = 0;
//            if ping_pong_index == 0 {
//
//                self.blur_framebuffers[ping_pong_index].bind();
//                self.vertical_gaussian_pipeline.bind();
//
//                if i == 0 {
//                    attachment_id = self.framebuffer.get_texture_attachment(1).get_id();
//                }
//                else {
//                    attachment_id = self.blur_framebuffers[1 - ping_pong_index].get_texture_attachment(0).get_id();
//                }
//
//                self.vertical_gaussian_pipeline.set_texture_2d_with_id("image",
//                                                                         attachment_id,
//                                                                         &self.sampler,
//                                                                       ShaderStage::Fragment);
//                StateManager::set_front_face(FrontFace::Clockwise);
//                self.fullscreen_mesh.draw();
//                StateManager::set_front_face(FrontFace::CounterClockwise);
//                self.blur_framebuffers[ping_pong_index].unbind(false);
//            }
//            else {
//                attachment_id = self.blur_framebuffers[1 - ping_pong_index].get_texture_attachment(0).get_id();
//                self.blur_framebuffers[ping_pong_index].bind();
//
//                self.horizontal_gaussian_pipeline.bind();
//                self.horizontal_gaussian_pipeline.set_texture_2d_with_id("image",
//                                                                       attachment_id,
//                                                                       &self.sampler,
//                                                                       ShaderStage::Fragment);
//
//                StateManager::set_front_face(FrontFace::Clockwise);
//                self.fullscreen_mesh.draw();
//                StateManager::set_front_face(FrontFace::CounterClockwise);
//                self.blur_framebuffers[ping_pong_index].unbind(false);
//            }
//
//        }
//    }
//
//    fn skybox_pass(&self) {
//        StateManager::set_depth_function(DepthFunction::LessOrEqual);
//        StateManager::set_face_culling(FaceCulling::Front);
//
//        self.framebuffer.bind();
//
//        self.skybox_program_pipeline.bind();
//
//        self.skybox_program_pipeline.set_matrix4f("view",
//                                                    &self.camera.get_transform(),
//                                                    ShaderStage::Vertex);
//
//        self.skybox_program_pipeline.set_matrix4f("projection",
//                                                    &self.projection_matrix,
//                                                    ShaderStage::Vertex);
//
//        self.skybox_mesh.draw();
//
//        self.framebuffer.unbind(true);
//        self.skybox_program_pipeline.unbind();
//
//        StateManager::set_depth_function(DepthFunction::Less);
//        StateManager::set_face_culling(FaceCulling::Back)
//    }
//
//    pub fn tonemap_pass(&self) {
//        clear_default_framebuffer(&Vec4::new(0.0, 0.0, 0.0, 1.0));
//
//        self.tonemapping_pipeline.bind();
//
//        let exposure: f32 = 1.0;
//        self.tonemapping_pipeline.set_texture_2d_with_id("image",
//                                                         self.framebuffer.get_texture_attachment(0).get_id(),
//                                                         &self.sampler_nearest,
//                                                         ShaderStage::Fragment)
//                                 .set_texture_2d_with_id("bloomImage",
//                                                         self.blur_framebuffers[1].get_texture_attachment(0).get_id(),
//                                                         &self.sampler,
//                                                         ShaderStage::Fragment)
//                                 .set_float("exposure",
//                                            exposure,
//                                            ShaderStage::Fragment);
//
//        StateManager::set_front_face(FrontFace::Clockwise);
//        self.fullscreen_mesh.draw();
//        StateManager::set_front_face(FrontFace::CounterClockwise);
//
//        self.tonemapping_pipeline.unbind()
//    }
}

//impl Scene<ApplicationData> for PbsScene {
//    fn name(&self) -> &str {
//        "PBS_SCENE"
//    }
//
//    fn setup(&mut self) {
//        self.skybox_program_pipeline.bind();
//        self.skybox_program_pipeline.set_texture_cube("skybox",
//                                                      &self.environment_maps.skybox,
//                                                      &self.sampler,
//                                                      ShaderStage::Fragment);
//        self.skybox_program_pipeline.unbind();
//    }
//
//    fn update(&mut self, dt: f32) {
//        let rotation_speed: f32 = 0.08;
//        self.model.transform = rotate(&self.model.transform,
//                                      2.0 * 180.0 * rotation_speed * dt,
//                                      Vec3::new(-1.0, 1.0, 1.0))
//    }
//
//    fn pre_draw(&mut self) {
//
//    }
//
//    fn draw(&mut self) {
//        self.geometry_pass();
//        self.bloom_pass();
//        self.skybox_pass();
//        self.tonemap_pass();
//    }
//
//    fn post_draw(&mut self) {
//    }
//}

impl Scene<ApplicationData> for PbsScene {
    fn start(&mut self, context: Context<ApplicationData>) {
    }

    fn stop(&mut self, context: Context<ApplicationData>) {

    }

    fn pause(&mut self, context: Context<ApplicationData>) {

    }

    fn resume(&mut self, context: Context<ApplicationData>) {

    }

    fn handle_event(&mut self, context: Context<ApplicationData>, event: Event) -> Transition<ApplicationData> {
        match event {
            Event::MouseButton(button, action, modifiers) => {
                println!("{:?} : {:?}", button, action)
            },
            Event::Key(key, action, _) => {
                println!("{:?} : {:?}", key, action)
            },
            Event::CursorPosition(x, y) => {
                println!("Cursor position: {}, {}", x, y)
            },
            _ => ()
        }
        Transition::None
    }

    fn update(&mut self, context: Context<ApplicationData>) -> Transition<ApplicationData> {
//        println!("{} {}", context.timer.get_delta(), context.user_data.foo);

        Transition::None
    }

    fn pre_draw(&mut self, context: Context<ApplicationData>) {

    }

    fn draw(&mut self, context: Context<ApplicationData>) {

    }

    fn post_draw(&mut self, context: Context<ApplicationData>) {

    }
}
