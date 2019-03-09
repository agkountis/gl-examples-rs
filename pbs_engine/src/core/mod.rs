pub mod application;
pub mod math;
mod window;

use self::math::vector::UVec2;

pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32
}

pub enum WindowMode {
    Windowed,
    Fullscreen
}

pub enum Msaa {
    None,
    X4,
    X16
}

pub struct Settings {
    pub name: String,
    pub version: Version,
    pub graphics_api_version: Version,
    pub window_size: UVec2,
    pub window_mode: WindowMode,
    pub msaa: Msaa,
    pub vsync: bool
}
