use std::time::Instant;
use std::rc::Rc;
use std::cell::RefCell;

use pbs_engine::core::Settings;
use pbs_engine::core::application::{RenderingApplication, clear_default_framebuffer, set_vieport};
use pbs_engine::core::window::Window;
use pbs_engine::core::rendering::Draw;
use pbs_engine::core::rendering::shader::{Shader, ShaderStage};
use pbs_engine::core::rendering::program_pipeline::ProgramPipeline;
use pbs_engine::core::rendering::mesh::{Mesh, MeshUtilities};
use pbs_engine::core::math::matrix::{Mat4, Vec3, translate, perspective, rotate};


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
    data: Rc<RefCell<RenderingData>>
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
            data: Rc::new(RefCell::new(RenderingData::new(mesh,
                                                          vertex_shader,
                                                          fragment_shader,
                                                          m,
                                                          Mat4::identity(), p)))
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
        let closure_data = Rc::clone(&self.data);
        self.window.set_resize_callback(move |w, h| {
            set_vieport(0, 0, w, h);
            closure_data.borrow_mut().proj = perspective(w as u32,
                                    h as u32,
                                    60,
                                    0.1,
                                    100.0)
        });

        let start = Instant::now();
        let mut prev_time = start.elapsed().as_secs() as f32 + start.elapsed().subsec_nanos() as f32 / 1_000_000_000.0;

        self.data.borrow().prog.bind();

        while !self.should_close() {
            let delta =  start.elapsed().as_secs() as f32 + start.elapsed().subsec_nanos() as f32 / 1_000_000_000.0 - prev_time;
            prev_time = start.elapsed().as_secs() as f32 + start.elapsed().subsec_nanos() as f32 / 1_000_000_000.0;
            self.update(delta); //TODO: fix timer
            self.draw();
        }
    }

    fn draw(&mut self) {
        clear_default_framebuffer(&self.get_settings().default_clear_color);

        let data = Rc::clone(&self.data);

        data.borrow().prog.set_matrix4f("model", &data.borrow().model, ShaderStage::Vertex);
        data.borrow().prog.set_matrix4f("view", &data.borrow().view, ShaderStage::Vertex);
        data.borrow().prog.set_matrix4f("projection", &data.borrow().proj, ShaderStage::Vertex);

        data.borrow().mesh.draw();

        self.swap_buffers()
    }

    fn update(&mut self, dt: f32) {
        self.handle_events();

        let m = rotate(&self.data.borrow().model,
                       2.0 * 360.0 * dt * 0.1,
                       Vec3::new(1.0, 1.0, 0.0));
        self.data.borrow_mut().model = m;
    }
}
