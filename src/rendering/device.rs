use gl_bindings as gl;

pub struct DeviceLimits {
    max_framebuffer_width: i32,
    max_framebuffer_height: i32,
}

impl Default for DeviceLimits {
    fn default() -> Self {
        let mut max_framebuffer_width = 0;
        let mut max_framebuffer_height = 0;
        unsafe {
            // gl::GetIntegerv(gl::MAX_TEXTURE_SIZE, &mut max_framebuffer_width);
            // gl::GetIntegerv(gl::MAX_FRAMEBUFFER_HEIGHT, &mut max_framebuffer_height);
        }

        Self {
            max_framebuffer_width,
            max_framebuffer_height,
        }
    }
}

impl DeviceLimits {
    pub fn new() -> Self {
        Default::default()
    }
}

pub struct Device {
    limits: DeviceLimits,
}

impl Default for Device {
    fn default() -> Self {
        Self {
            limits: DeviceLimits::new(),
        }
    }
}

impl Device {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn limits(&self) -> &DeviceLimits {
        &self.limits
    }
}
