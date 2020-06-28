use glutin::window::Window;
use imgui_winit_support::{HiDpiMode, WinitPlatform};

pub use ::imgui::*;

pub(crate) struct ImGui {
    pub(crate) context: imgui::Context,
    pub(crate) platform: imgui_winit_support::WinitPlatform,
    pub(crate) renderer: imgui_opengl_renderer::Renderer,
}

impl ImGui {
    pub(crate) fn new<F>(window: &Window, load_fn: F) -> Self
    where
        F: FnMut(&'static str) -> *const ::std::os::raw::c_void,
    {
        let mut context = imgui::Context::create();
        context.set_ini_filename(None);
        let mut platform = WinitPlatform::init(&mut context);
        platform.attach_window(context.io_mut(), window, HiDpiMode::Default);

        let renderer = imgui_opengl_renderer::Renderer::new(&mut context, load_fn);

        Self {
            context,
            platform,
            renderer,
        }
    }
}

pub trait Gui {
    fn gui(&mut self, ui: &Ui);
}
