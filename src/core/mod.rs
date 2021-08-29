pub mod application;
pub mod asset;
pub mod camera;
pub mod entity;
pub mod math;
pub mod scene;
pub mod timer;

use self::math::{UVec2, Vec4};
use crate::asset::AssetManager;
use crate::rendering::device::Device;
use crate::rendering::framebuffer::TemporaryFramebufferPool;
use crate::timer::Timer;
use glutin::window::Window;
use std::any::Any;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

#[derive(Debug, Clone, Copy)]
#[repr(u32)]
pub enum Msaa {
    None = 1,
    X2 = 2,
    X4 = 4,
    X8 = 8,
    X16 = 16,
}

#[derive(Debug)]
pub struct Settings {
    pub name: String,
    pub asset_path: PathBuf,
    pub version: Version,
    pub graphics_api_version: Version,
    pub window_size: UVec2,
    pub fullscreen: bool,
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
    pub window: &'a Window,
    pub device: &'a Device,
    pub asset_manager: &'a mut AssetManager,
    pub timer: &'a mut Timer,
    pub framebuffer_cache: &'a mut TemporaryFramebufferPool,
    pub settings: &'a Settings,
}

impl<'a> Context<'a> {
    pub fn new(
        window: &'a Window,
        device: &'a Device,
        asset_manager: &'a mut AssetManager,
        timer: &'a mut Timer,
        framebuffer_cache: &'a mut TemporaryFramebufferPool,
        settings: &'a Settings,
    ) -> Self {
        Self {
            window,
            device,
            asset_manager,
            timer,
            framebuffer_cache,
            settings,
        }
    }
}

pub trait AsAny {
    fn as_any(&self) -> &dyn Any;
}

pub trait AsAnyMut {
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub fn slice_as_bytes<T>(slice: &[T]) -> &[u8] {
    unsafe {
        std::slice::from_raw_parts(
            slice.as_ptr() as *const u8,
            slice.len() * std::mem::size_of::<T>(),
        )
    }
}
