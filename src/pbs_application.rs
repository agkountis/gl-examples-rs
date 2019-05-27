use std::rc::Rc;
use std::cell::Cell;

use pbs_engine::core::Settings;
use pbs_engine::core::application::{RenderingApplication, clear_default_framebuffer};
use pbs_engine::core::rendering::state::StateManager;
use pbs_engine::core::window::Window;
use pbs_engine::core::timer::Timer;
use pbs_engine::core::rendering::Draw;
use pbs_engine::core::rendering::shader::{Shader, ShaderStage};
use pbs_engine::core::rendering::program_pipeline::ProgramPipeline;
use pbs_engine::core::rendering::mesh::{Mesh, MeshUtilities};
use pbs_engine::core::math::matrix::{Mat4, translate, perspective, rotate};
use pbs_engine::core::math::vector::{Vec3, Vec4, UVec2};
use pbs_engine::core::rendering::texture::{Texture2D, TextureCube, SizedTextureFormat};
use pbs_engine::core::rendering::sampler::{Sampler, MinificationFilter, MagnificationFilter, WrappingMode};
use pbs_engine::core::rendering::framebuffer::{Framebuffer, FramebufferAttachmentCreateInfo};


struct RenderingData {
    mesh: Mesh,
    prog: ProgramPipeline,
    model: Cell<Mat4>,
    view: Cell<Mat4>,
    proj: Cell<Mat4>,
    albedo: Texture2D,
    specular: Texture2D,
    normals: Texture2D,
    ao: Texture2D,
    sampler: Sampler
}

impl RenderingData {

    pub fn new(mesh: Mesh,
               vert: Shader,
               frag: Shader,
               model: Mat4,
               view: Mat4,
               proj: Mat4,
               albedo: Texture2D,
               specular: Texture2D,
               normals: Texture2D,
               ao: Texture2D,
               sampler: Sampler) -> RenderingData {

        let prog = ProgramPipeline::new().add_shader(&vert)
            .add_shader(&frag)
            .build().unwrap();

        RenderingData {
            mesh,
            prog,
            model: Cell::new(model),
            view: Cell::new(view),
            proj: Cell::new(proj),
            albedo,
            specular,
            normals,
            ao,
            sampler
        }
    }

}

pub struct Application<'a> {
    window: Window,
    settings: Settings<'a>,
    timer: Timer,
    data: Rc<RenderingData>
}

impl<'a> Application<'a> {

    pub fn new(settings: Settings) -> Application {

        let window = Window::new(&settings.name,
                                 settings.window_size,
                                 &settings.graphics_api_version,
                                 &settings.window_mode,
                                 &settings.msaa);

        let vertex_shader = Shader::new_from_text(ShaderStage::Vertex,
                                              "sdr/pbs.vert").unwrap();

        let fragment_shader = Shader::new_from_text(ShaderStage::Fragment,
                                                "sdr/pbs.frag").unwrap();

        let mesh = MeshUtilities::generate_cube(1.0);

        let m = translate(&Mat4::identity(),
                          Vec3::new(0.0, 0.0, -2.0));
        let p = perspective(window.get_width(),
                            window.get_height(),
                            45,
                            0.1,
                            500.0);

        let albedo = Texture2D::new_from_file("assets/textures/brickwall.jpg", true, true)
            .expect("Failed to load texture");

        let specular = Texture2D::new_from_file("assets/textures/brickwall_spec.png", false, true)
            .expect("Failed to load texture");

        let normals = Texture2D::new_from_file("assets/textures/brickwall_normal.jpg", false, true)
            .expect("Failed to load texture");

        let ao = Texture2D::new_from_file("assets/textures/brickwall_ao.png", false, true)
            .expect("Failed to load texture");

        let skybox = TextureCube::new_from_file("assets/textures/ktx/skybox/ibl_skybox.ktx")
            .expect("Failed to load Skybox");

        let irradiance = TextureCube::new_from_file("assets/textures/ktx/irradiance/ibl_irradiance.ktx")
            .expect("Failed to load Irradiance map");

        let radiance = TextureCube::new_from_file("assets/textures/ktx/radiance/ibl_radiance.ktx")
            .expect("Failed to load Radiance map");

        let sampler = Sampler::new(MinificationFilter::LinearMipmapLinear,
                                   MagnificationFilter::Linear,
                                   WrappingMode::ClampToEdge,
                                   WrappingMode::ClampToEdge,
                                   WrappingMode::ClampToEdge,
                                   Vec4::new(0.0, 0.0, 0.0, 0.0));

        let fb = Framebuffer::new(vec![
            FramebufferAttachmentCreateInfo::new(UVec2::new(512, 512),
                                                 SizedTextureFormat::Rgba16f),
            FramebufferAttachmentCreateInfo::new(UVec2::new(512, 512),
                                                 SizedTextureFormat::Depth24Stencil8)
        ]).unwrap_or_else(|error|{
            panic!("Framebuffer creation error: {}", error)
        });

        Application {
            window,
            settings,
            timer: Timer::new(),
            data: Rc::new(RenderingData::new(mesh,
                                             vertex_shader,
                                             fragment_shader,
                                             m,
                                             Mat4::identity(),
                                             p,
                                             albedo,
                                             specular,
                                             normals,
                                             ao,
                                             sampler))
        }
    }

    pub fn handle_events(&mut self) {
        self.window.handle_events()
    }

    pub fn should_close(&self) -> bool {
        self.window.should_close()
    }

    pub fn get_settings(&self) -> &Settings {
        &self.settings
    }

    pub fn swap_buffers(&mut self) {
        self.window.swap_buffers()
    }
}

impl<'a> RenderingApplication for Application<'a> {
    fn run(&mut self) {

        self.setup();

        while !self.should_close() {
            let delta = self.timer.get_delta();
            self.update(delta);
            self.pre_draw();
            self.draw();
            self.post_draw();
        }
    }

    fn setup(&mut self) {
        self.window.set_resize_callback({
            let closure_data = Rc::clone(&self.data);

            move |w, h| {
                StateManager::set_viewport(0, 0, w, h);
                closure_data.proj.set(perspective(w as u32,
                                                  h as u32,
                                                  60,
                                                  0.1,
                                                  100.0))
            }
        });

        self.data.prog.bind();
        self.data.prog.set_texture_2d("diffuse",
                                      &self.data.albedo,
                                      &self.data.sampler,
                                      ShaderStage::Fragment);

        self.data.prog.set_texture_2d("specular",
                                      &self.data.specular,
                                      &self.data.sampler,
                                      ShaderStage::Fragment);

        self.data.prog.set_texture_2d("normal",
                                      &self.data.normals,
                                      &self.data.sampler,
                                      ShaderStage::Fragment);

        self.data.prog.set_texture_2d("ao",
                                      &self.data.ao,
                                      &self.data.sampler,
                                      ShaderStage::Fragment);
    }

    fn update(&mut self, dt: f32) {
        self.handle_events();

        self.data.model.set(rotate(&self.data.model.get(),
                                   2.0 * 360.0 * dt * 0.01,
                                   Vec3::new(1.0, 1.0, 0.0)));
    }

    fn pre_draw(&mut self) {

    }

    fn draw(&mut self) {
        clear_default_framebuffer(&self.get_settings().default_clear_color);

        self.data.prog.set_matrix4f("model", &self.data.model.get(), ShaderStage::Vertex);
        self.data.prog.set_matrix4f("view", &self.data.view.get(), ShaderStage::Vertex);
        self.data.prog.set_matrix4f("projection", &self.data.proj.get(), ShaderStage::Vertex);

        self.data.mesh.draw();
    }

    fn post_draw(&mut self) {
        self.swap_buffers()
    }
}
