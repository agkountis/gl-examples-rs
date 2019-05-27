#[macro_use]
macro_rules! offset_of {
    ($ty:ty, $field:ident) => {
        unsafe { &(*(0 as *const $ty)).$field as *const _ as usize }
    }
}

pub mod application;
pub mod math;
pub mod rendering;
pub mod window;
pub mod scene;
pub mod camera;
pub mod entity;
pub mod timer;

use self::math::vector::UVec2;
use crate::core::math::vector::Vec4;

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
    pub vsync: bool,
    pub default_clear_color: Vec4
}

#[derive(Debug)]
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
