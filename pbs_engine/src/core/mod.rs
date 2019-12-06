pub mod application;
pub mod math;
pub mod window;
pub mod scene;
pub mod camera;
pub mod entity;
pub mod timer;
pub mod asset;
pub mod engine;

mod model_loader;
mod messaging;

use self::math::{UVec2, Vec4};

#[derive(Debug, Clone, Copy)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32
}

#[derive(Debug, Clone, Copy)]
pub enum WindowMode {
    Windowed,
    Fullscreen
}

#[derive(Debug, Clone, Copy)]
pub enum Msaa {
    None,
    X4,
    X8,
    X16
}

#[derive(Debug)]
pub struct Settings{
    pub name: String,
    pub version: Version,
    pub graphics_api_version: Version,
    pub window_size: UVec2,
    pub window_mode: WindowMode,
    pub msaa: Msaa,
    pub vsync: bool,
    pub default_clear_color: Vec4
}

#[derive(Debug, Clone, Copy)]
pub struct Rectangle {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32
}

impl Rectangle {
    pub fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Rectangle {
            x,
            y,
            width,
            height
        }
    }
}
