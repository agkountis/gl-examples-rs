pub mod application;
pub mod math;
pub mod rendering;

mod window;

use self::math::vector::UVec2;

#[derive(Debug)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32
}

#[derive(Debug)]
pub enum WindowMode {
    Windowed,
    Fullscreen
}

#[derive(Debug)]
pub enum Msaa {
    None,
    X4,
    X16
}

#[derive(Debug)]
pub struct Settings<'a> {
    pub name: &'a str,
    pub version: Version,
    pub graphics_api_version: Version,
    pub window_size: UVec2,
    pub window_mode: WindowMode,
    pub msaa: Msaa,
    pub vsync: bool
}
