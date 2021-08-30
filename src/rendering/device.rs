use crate::imgui::{im_str, Condition, Gui, Ui};
use gl_bindings as gl;
use std::ffi::CStr;

#[derive(Debug)]
pub struct DeviceInfo {
    version: String,
    max_framebuffer_width: i32,
    max_framebuffer_height: i32,
    max_texture_size: i32,
    max_cube_map_texture_size: i32,
    max_texture_image_units: i32,
    max_uniform_buffer_bindings: i32,
    max_uniform_block_size: i32,
    max_uniform_locations: i32,
}

impl Default for DeviceInfo {
    fn default() -> Self {
        let version = unsafe {
            let data = CStr::from_ptr(gl::GetString(gl::VERSION) as *const _)
                .to_bytes()
                .to_vec();
            String::from_utf8(data).unwrap()
        };

        let mut max_framebuffer_width = 0;
        let mut max_framebuffer_height = 0;

        let mut max_texture_size = 0;
        let mut max_cube_map_texture_size = 0;
        let mut max_texture_image_units = 0;

        let mut max_uniform_buffer_bindings = 0;
        let mut max_uniform_block_size = 0;
        let mut max_uniform_locations = 0;

        unsafe {
            gl::GetIntegerv(gl::MAX_FRAMEBUFFER_WIDTH, &mut max_texture_size);
            gl::GetIntegerv(gl::MAX_FRAMEBUFFER_HEIGHT, &mut max_framebuffer_height);

            gl::GetIntegerv(gl::MAX_TEXTURE_SIZE, &mut max_framebuffer_width);
            gl::GetIntegerv(
                gl::MAX_CUBE_MAP_TEXTURE_SIZE,
                &mut max_cube_map_texture_size,
            );
            gl::GetIntegerv(gl::MAX_TEXTURE_IMAGE_UNITS, &mut max_texture_image_units);

            gl::GetIntegerv(
                gl::MAX_UNIFORM_BUFFER_BINDINGS,
                &mut max_uniform_buffer_bindings,
            );
            gl::GetIntegerv(gl::MAX_UNIFORM_BLOCK_SIZE, &mut max_uniform_block_size);
            gl::GetIntegerv(gl::MAX_UNIFORM_LOCATIONS, &mut max_uniform_locations);
        }

        Self {
            version,
            max_framebuffer_width,
            max_framebuffer_height,
            max_texture_size,
            max_cube_map_texture_size,
            max_texture_image_units,
            max_uniform_buffer_bindings,
            max_uniform_block_size,
            max_uniform_locations,
        }
    }
}

impl DeviceInfo {
    pub fn new() -> Self {
        Default::default()
    }
}

pub struct Device {
    info: DeviceInfo,
}

impl Default for Device {
    fn default() -> Self {
        Self {
            info: DeviceInfo::new(),
        }
    }
}

impl Device {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn info(&self) -> &DeviceInfo {
        &self.info
    }
}

impl Gui for Device {
    fn gui(&mut self, ui: &Ui) {
        let info = self.info();

        imgui::Window::new(im_str!("Bloom Debug"))
            .focus_on_appearing(true)
            .bring_to_front_on_focus(true)
            .size([256.0f32, 500.0f32], Condition::Appearing)
            .build(ui, || {
                ui.text(&info.version);
                ui.text(format!(
                    "Max framebuffer width: {}",
                    info.max_framebuffer_width
                ));
                ui.text(format!(
                    "Max framebuffer height: {}",
                    info.max_framebuffer_height
                ));
            });
    }
}
