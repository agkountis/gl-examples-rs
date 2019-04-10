use pbs_engine::core::Settings;
use pbs_engine::core::application::{RenderingApplication, clear_default_framebuffer};
use pbs_engine::core::window::Window;
use pbs_engine::core::rendering::shader::{Shader, ShaderStage};
use pbs_engine::core::rendering::program_pipeline::ProgramPipeline;
use pbs_engine::core::rendering::mesh::{Mesh, MeshUtilities};
use pbs_engine::core::math::matrix::{Mat4, Vec3, translate, perspective, rotate};

use std::time::Instant;

struct RenderingData {
    mesh: Mesh,
    prog: ProgramPipeline,
    model: Mat4,
    view: Mat4,
    proj: Mat4
}

impl RenderingData {
    pub fn new(mesh: Mesh, vert: Shader, frag: Shader, model: Mat4, view: Mat4, proj: Mat4) -> RenderingData {

        let prog = ProgramPipeline::new().add_shader(&vert)
            .add_shader(&frag)
            .build().unwrap();

        RenderingData {
            mesh,
            prog,
            model,
            view,
            proj
        }
    }
}

pub struct Application<'a> {
    window: Window,
    settings: Settings<'a>,
    data: RenderingData
}

impl<'a> Application<'a> {

    pub fn new(settings: Settings) -> Application {

        let window = Window::new(&settings.name,
                                 settings.window_size,
                                 &settings.graphics_api_version,
                                 &settings.window_mode,
                                 &settings.msaa);

        let vertex_shader = Shader::new_from_text(ShaderStage::Vertex,
                                              "sdr/simple_blinn_phong.vert").unwrap();

        let fragment_shader = Shader::new_from_text(ShaderStage::Fragment,
                                                "sdr/simple_blinn_phong.frag").unwrap();

        let mesh = MeshUtilities::generate_cube(1.0);

        let m = translate(&Mat4::identity(),
                          Vec3::new(0.0, 0.0, -4.0));
        let p = perspective(window.get_width(),
                            window.get_height(),
                            45,
                            0.1,
                            500.0);

        Application {
            window,
            settings,
            data: RenderingData::new(mesh,
                                     vertex_shader,
                                     fragment_shader,
                                     m,
                                     Mat4::identity(), p)
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
        let start = Instant::now();
        let mut prev_time = start.elapsed().as_secs() as f32 + start.elapsed().subsec_nanos() as f32 / 1_000_000_000.0;

        self.data.prog.bind();

        while !self.should_close() {
            let delta =  start.elapsed().as_secs() as f32 + start.elapsed().subsec_nanos() as f32 / 1_000_000_000.0 - prev_time;
            prev_time = start.elapsed().as_secs() as f32 + start.elapsed().subsec_nanos() as f32 / 1_000_000_000.0;
            self.update(delta); //TODO: fix timer
            self.draw();
        }
    }

    fn draw(&mut self) {
        clear_default_framebuffer(&self.get_settings().default_clear_color);

        self.data.prog.set_matrix4f("model", &self.data.model, ShaderStage::Vertex);
        self.data.prog.set_matrix4f("view", &self.data.view, ShaderStage::Vertex);
        self.data.prog.set_matrix4f("projection", &self.data.proj, ShaderStage::Vertex);

        self.data.mesh.draw();

        self.swap_buffers()
    }

    fn update(&mut self, dt: f32) {
        self.handle_events();

        self.data.model = rotate(&self.data.model, 2.0 * 360.0 * dt * 0.1, Vec3::new(1.0, 1.0, 0.0));
    }
}
