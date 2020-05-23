pub mod application;
pub mod asset;
pub mod camera;
pub mod entity;
pub mod event;
pub mod input;
pub mod math;
pub mod scene;
pub mod timer;
pub mod window;

mod messaging;
mod model_loader;

use self::math::{UVec2, Vec4};
use crate::asset::AssetManager;
use crate::timer::Timer;
use crate::window::Window;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

#[derive(Debug, Clone, Copy)]
pub enum WindowMode {
    Windowed,
    Fullscreen,
}

#[derive(Debug, Clone, Copy)]
pub enum Msaa {
    None,
    X4,
    X8,
    X16,
}

#[derive(Debug)]
pub struct Settings {
    pub name: String,
    pub asset_path: PathBuf,
    pub version: Version,
    pub graphics_api_version: Version,
    pub window_size: UVec2,
    pub window_mode: WindowMode,
    pub msaa: Msaa,
    pub vsync: bool,
    pub default_clear_color: Vec4,
}

#[derive(Debug, Clone, Copy)]
pub struct Rectangle {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl Rectangle {
    pub fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Rectangle {
            x,
            y,
            width,
            height,
        }
    }
}

pub struct Context<'a> {
    pub window: &'a mut Window,
    pub asset_manager: &'a mut AssetManager,
    pub timer: &'a mut Timer,
    pub settings: &'a mut Settings,
}

impl<'a> Context<'a> {
    pub fn new(
        window: &'a mut Window,
        asset_manager: &'a mut AssetManager,
        timer: &'a mut Timer,
        settings: &'a mut Settings,
    ) -> Self {
        Self {
            window,
            asset_manager,
            timer,
            settings,
        }
    }
}
