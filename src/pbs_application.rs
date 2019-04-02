use pbs_engine::core::Settings;
use pbs_engine::core::application::{RenderingApplication, Draw, Update, Run, clear_default_framebuffer};
use pbs_engine::core::window::Window;
use pbs_engine::core::rendering::shader::{Shader, ShaderType};
use pbs_engine::core::rendering::program_pipeline::ProgramPipeline;
use pbs_engine::core::rendering::mesh::{Mesh, MeshUtilities};

pub struct Application<'a> {
    window: Window,
    settings: Settings<'a>
}

impl<'a> Application<'a> {

    pub fn new(settings: Settings) -> Application {

        let window = Window::new(&settings.name,
                                 settings.window_size,
                                 &settings.graphics_api_version,
                                 &settings.window_mode,
                                 &settings.msaa);

        let vertex_shader: Shader;
        let fragment_shader: Shader;
        let program_pipeline: ProgramPipeline;

        if pbs_engine::Capabilities::spirv_supported() {

            vertex_shader = Shader::new_from_spirv(ShaderType::Vertex,
                                                   "main",
                                                   "sdr/pbs.vert.spv").unwrap();

            fragment_shader = Shader::new_from_spirv(ShaderType::Fragment,
                                                     "main",
                                                     "sdr/pbs.frag.spv").unwrap();
        }
        else {
            vertex_shader = Shader::new_from_text(ShaderType::Vertex,
                                                  "sdr/pbs.vert").unwrap();

            fragment_shader = Shader::new_from_text(ShaderType::Fragment,
                                                    "sdr/pbs.frag").unwrap();
        }


        let program_pipeline = ProgramPipeline::new().add_shader(&vertex_shader)
                                                                   .add_shader(&fragment_shader)
                                                                   .build().unwrap();

        let mesh = MeshUtilities::generate_cube(1.0);

        Application {
            window,
            settings
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
        while !self.should_close() {
            self.update(0.0); //TODO: fix timer
            self.draw();
        }
    }

    fn draw(&mut self) {
        clear_default_framebuffer(&self.get_settings().default_clear_color);

        self.swap_buffers()
    }

    fn update(&mut self, dt: f32) {
        self.handle_events()
    }
}
