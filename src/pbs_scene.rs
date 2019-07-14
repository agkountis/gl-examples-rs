use pbs_engine::core::scene::Scene;
use pbs_engine::core::camera::Camera;
use pbs_engine::core::rendering::mesh::{Mesh, MeshUtilities};
use pbs_engine::core::rendering::program_pipeline::ProgramPipeline;
use pbs_engine::core::math::vector::{Vec3, UVec2, Vec4};
use pbs_engine::core::rendering::shader::{ShaderStage, Shader};
use pbs_engine::core::rendering::framebuffer::{Framebuffer, FramebufferAttachmentCreateInfo, AttachmentType};
use pbs_engine::core::rendering::texture::{Texture2D, SizedTextureFormat, TextureCube};
use pbs_engine::core::rendering::sampler::{Sampler, MinificationFilter, MagnificationFilter, WrappingMode};
use pbs_engine::core::rendering::material::Material;
use pbs_engine::core::window::Window;
use pbs_engine::core::math::matrix::{perspective, Mat4, rotate, translate};
use pbs_engine::core::rendering::Draw;
use pbs_engine::core::rendering::state::{StateManager, DepthFunction, FaceCulling};



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
    camera: Camera,
    model: Model,
    skybox_mesh: Mesh,
    material: Material,
    environment_maps: EnvironmentMaps,
    geometry_program_pipeline: ProgramPipeline,
    skybox_program_pipeline: ProgramPipeline,
    horizontal_gaussian_pipeline: ProgramPipeline,
    vertical_gaussian_pipeline: ProgramPipeline,
    framebuffer: Framebuffer,
    blur_framebuffers: [Framebuffer; 2],
    default_fb_size: UVec2,
    sampler: Sampler,
    projection_matrix: Mat4
}

impl PbsScene {
    pub fn new(window: &Window) -> Self {
        let mut camera = Camera::new();
        camera.look_at(Vec3::new(0.0, 0.0, 0.0),
                       Vec3::new(0.0, 0.0, 1.0),
                       Vec3::new(0.0, 1.0, 0.0));

        let geometry_prog = ProgramPipeline::new()
            .add_shader(&Shader::new_from_text(ShaderStage::Vertex,
                                               "sdr/pbs.vert").unwrap())
            .add_shader(&Shader::new_from_text(ShaderStage::Fragment,
                                               "sdr/pbs.frag").unwrap())
            .build()
            .unwrap();

        let skybox_prog = ProgramPipeline::new()
            .add_shader(&Shader::new_from_text(ShaderStage::Vertex,
                                               "sdr/skybox.vert").unwrap())
            .add_shader(&Shader::new_from_text(ShaderStage::Fragment,
                                               "sdr/skybox.frag").unwrap())
            .build()
            .unwrap();

        let fullscreen_shader = Shader::new_from_text(ShaderStage::Vertex,
                                                      "sdr/fullscreen.vert").unwrap();
        let horizontal_gaussian_prog = ProgramPipeline::new()
            .add_shader(&fullscreen_shader)
            .add_shader(&Shader::new_from_text(ShaderStage::Fragment,
                                               "sdr/gaussian_blur_horizontal.frag").unwrap())
            .build()
            .unwrap();

        let vertical_gaussian_prog = ProgramPipeline::new().add_shader(&fullscreen_shader)
            .add_shader(&Shader::new_from_text(ShaderStage::Fragment,
                                               "sdr/gaussian_blur_vertical.frag").unwrap())
            .build()
            .unwrap();


        let mesh = MeshUtilities::generate_cube(1.0);
        let skybox_mesh = MeshUtilities::generate_cube(1.0);

        let albedo = Texture2D::new_from_file("assets/textures/pbs/rusted_iron/albedo.png", true, true)
            .expect("Failed to load albedo texture");

        let metallic = Texture2D::new_from_file("assets/textures/pbs/rusted_iron/metallic.png", false, true)
            .expect("Failed to load metallic texture");

        let roughness = Texture2D::new_from_file("assets/textures/pbs/rusted_iron/roughness.png", false, true)
            .expect("Failed to load roughness texture");

        let normals = Texture2D::new_from_file("assets/textures/pbs/rusted_iron/normal.png", false, true)
            .expect("Failed to load normals texture");

        let ao = Texture2D::new_from_file("assets/textures/pbs/rusted_iron/ao.png", false, true)
            .expect("Failed to load ao texture");

        let ibl_brdf_lut = Texture2D::new_from_file("assets/textures/pbs/ibl_brdf_lut.png", false, false)
            .expect("Failed to load BRDF LUT texture");

        let skybox = TextureCube::new_from_file("assets/textures/pbs/ktx/skybox/ibl_skybox.ktx")
            .expect("Failed to load Skybox");

        let irradiance = TextureCube::new_from_file("assets/textures/pbs/ktx/irradiance/ibl_irradiance.ktx")
            .expect("Failed to load Irradiance map");

        let radiance = TextureCube::new_from_file("assets/textures/pbs/ktx/radiance/ibl_radiance.ktx")
            .expect("Failed to load Radiance map");

        let framebuffer = Framebuffer::new(UVec2::new(window.get_framebuffer_width(),
                                                      window.get_framebuffer_height()),
                                  vec![
                                      FramebufferAttachmentCreateInfo::new(SizedTextureFormat::Rgba16f,
                                                                           AttachmentType::Texture),
                                      FramebufferAttachmentCreateInfo::new(SizedTextureFormat::Rgba16f,
                                                                           AttachmentType::Texture),
                                      FramebufferAttachmentCreateInfo::new(SizedTextureFormat::Depth24Stencil8,
                                                                           AttachmentType::Renderbuffer)
                                  ])
            .unwrap_or_else(|error| {
                panic!("Framebuffer creation error: {}", error)
            });

        let blur_framebuffers: [Framebuffer; 2] =
            [ Framebuffer::new(UVec2::new(window.get_framebuffer_width(),
                                          window.get_framebuffer_height()),
                               vec![
                                   FramebufferAttachmentCreateInfo::new(SizedTextureFormat::Rgba16f,
                                                                        AttachmentType::Texture)])
                .unwrap_or_else(|error| {panic!("Framebuffer creation error: {}", error)}),
                Framebuffer::new(UVec2::new(window.get_framebuffer_width(),
                                            window.get_framebuffer_height()),
                                 vec![
                                     FramebufferAttachmentCreateInfo::new(SizedTextureFormat::Rgba16f,
                                                                          AttachmentType::Texture)])
                    .unwrap_or_else(|error| {
                        panic!("Framebuffer creation error: {}", error)
                    }) ];


        let sampler = Sampler::new(MinificationFilter::LinearMipmapLinear,
                                   MagnificationFilter::Linear,
                                   WrappingMode::ClampToEdge,
                                   WrappingMode::ClampToEdge,
                                   WrappingMode::ClampToEdge,
                                   Vec4::new(0.0, 0.0, 0.0, 0.0));

        let projection = perspective(window.get_framebuffer_width(),
                                     window.get_framebuffer_height(),
                                                      60,
                                                      0.1,
                                                      100.0);

        PbsScene {
            camera,
            model: Model{
                mesh,
                transform: translate(&Mat4::identity(), &Vec3::new(0.0, 0.0, 2.0))
            },
            skybox_mesh,
            material: Material{
                albedo,
                metallic,
                roughness,
                normals,
                ao,
                ibl_brdf_lut
            },
            environment_maps: EnvironmentMaps {
                skybox,
                irradiance,
                radiance
            },
            geometry_program_pipeline: geometry_prog,
            skybox_program_pipeline: skybox_prog,
            horizontal_gaussian_pipeline: horizontal_gaussian_prog,
            vertical_gaussian_pipeline: vertical_gaussian_prog,
            framebuffer,
            blur_framebuffers,
            default_fb_size: UVec2::new(window.get_framebuffer_width(),
                                        window.get_framebuffer_height()),
            sampler,
            projection_matrix: projection
        }
    }

    fn geometry_pass(&self) {
        self.geometry_program_pipeline.bind();

        self.geometry_program_pipeline.set_matrix4f("model",
                                                    &self.model.transform,
                                                    ShaderStage::Vertex);
        self.geometry_program_pipeline.set_matrix4f("view",
                                                    &self.camera.get_transform(),
                                                    ShaderStage::Vertex);

        self.geometry_program_pipeline.set_vector3f("eyePosition",
                                                    &self.camera.get_position(),
                                                    ShaderStage::Vertex);

        self.geometry_program_pipeline.set_matrix4f("projection",
                                                    &self.projection_matrix,
                                                    ShaderStage::Vertex);

        self.model.mesh.draw();

        self.geometry_program_pipeline.unbind()
    }

    fn bloom_pass(&self) {
        //TODO
        let blur_strength = 10;

        for i in 0..blur_strength {
            let ping_pong_index = i % 2;

            self.horizontal_gaussian_pipeline.bind();

            let mut attachment_id: u32 = 0;
            if i == 0 {
                attachment_id = self.framebuffer.get_texture_attachment(1).get_id();
            }
            else if (ping_pong_index == 1) {
//                attachment_id =
            }

            self.horizontal_gaussian_pipeline.set_texture_2d_with_id("image",
                                                                     self.framebuffer.get_texture_attachment(1).get_id(),
                                                                     &self.sampler,
                                                                     ShaderStage::Fragment);
            self.blur_framebuffers[ping_pong_index].bind();
            Mesh::draw_fullscreen();

        }



        //self.b
    }

    fn skybox_pass(&self) {
        StateManager::set_depth_function(DepthFunction::LessOrEqual);
        StateManager::set_face_culling(FaceCulling::Front);

        self.skybox_program_pipeline.bind();
        // TODO: render

        self.skybox_program_pipeline.set_matrix4f("view",
                                                    &self.camera.get_transform(),
                                                    ShaderStage::Vertex);

        self.skybox_program_pipeline.set_matrix4f("projection",
                                                    &self.projection_matrix,
                                                    ShaderStage::Vertex);

        self.skybox_mesh.draw();

        self.skybox_program_pipeline.unbind();

        StateManager::set_depth_function(DepthFunction::Less);
        StateManager::set_face_culling(FaceCulling::Back)
    }

    pub fn tonemap_pass(&self) {
        //TODO
    }

    pub fn display_pass(&self) {
        //TODO
    }
}

impl Scene for PbsScene {

    fn setup(&mut self) {
        self.geometry_program_pipeline.bind();
        self.geometry_program_pipeline.set_texture_2d("albedoMap",
                                                      &self.material.albedo,
                                                      &self.sampler,
                                                      ShaderStage::Fragment)
                             .set_texture_2d("metallicMap",
                                             &self.material.metallic,
                                             &self.sampler,
                                             ShaderStage::Fragment)
                             .set_texture_2d("roughnessMap",
                                             &self.material.roughness,
                                             &self.sampler,
                                             ShaderStage::Fragment)
                             .set_texture_2d("normalMap",
                                             &self.material.normals,
                                             &self.sampler,
                                             ShaderStage::Fragment)
                             .set_texture_2d("aoMap",
                                      &self.material.ao,
                                             &self.sampler,
                                             ShaderStage::Fragment)
                             .set_texture_2d("brdfLUT",
                                             &self.material.ibl_brdf_lut,
                                             &self.sampler,
                                             ShaderStage::Fragment)
                             .set_texture_cube("irradianceMap",
                                               &self.environment_maps.irradiance,
                                               &self.sampler,
                                               ShaderStage::Fragment)
                             .set_texture_cube("radianceMap",
                                               &self.environment_maps.radiance,
                                               &self.sampler,
                                               ShaderStage::Fragment)
                             .set_vector3f("wLightDirection",
                                           &Vec3::new(-0.5, 0.0, -1.0),
                                           ShaderStage::Fragment)
                             .set_vector3f("lightColor",
                                           &Vec3::new(3.5, 3.5, 3.5),
                                           ShaderStage::Fragment);

        self.geometry_program_pipeline.unbind();

        self.skybox_program_pipeline.bind();
        self.skybox_program_pipeline.set_texture_cube("skybox",
                                                      &self.environment_maps.skybox,
                                                      &self.sampler,
                                                      ShaderStage::Fragment);
        self.skybox_program_pipeline.unbind();
    }

    fn update(&mut self, dt: f32) {
        let rotation_speed: f32 = 0.08;
        self.model.transform = rotate(&self.model.transform,
                                      2.0 * 180.0 * rotation_speed * dt,
                                      Vec3::new(-1.0, 1.0, 1.0))
    }

    fn pre_draw(&mut self) {
        self.framebuffer.bind();
        self.framebuffer.clear(&Vec4::new(1.0, 0.0, 0.0, 1.0));
        self.geometry_program_pipeline.bind()
    }

    fn draw(&mut self) {
        self.geometry_pass();
        //self.bloom_pass();
        self.skybox_pass();
        self.tonemap_pass();
        self.display_pass();
    }

    fn post_draw(&mut self) {
        self.framebuffer.unbind();
        Framebuffer::blit_to_default(&self.framebuffer,
                                     self.default_fb_size);
    }
}
