use crate::input::{Action, Key, Modifiers, MouseButton};
use glfw;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum Event {
    WindowPosition(i32, i32),
    WindowSize(i32, i32),
    WindowClose,
    WindowRefresh,
    WindowFocus(bool),
    WindowIconify(bool),
    WindowFramebufferSize(i32, i32),
    WindowFileDrop(Vec<PathBuf>),
    WindowMaximize(bool),
    WindowContentScale(f32, f32),
    CursorPosition(f64, f64),
    CursorEnter(bool),
    Scroll(f64, f64),
    MouseButton(MouseButton, Action, Modifiers),
    Key(Key, Action, Modifiers),
    Char(char),
    CharModifiers(char, Modifiers),
    Quit,
}

impl From<glfw::WindowEvent> for Event {
    fn from(event: glfw::WindowEvent) -> Self {
        match event {
            glfw::WindowEvent::Pos(x, y) => Event::WindowPosition(x, y),
            glfw::WindowEvent::Size(x, y) => Event::WindowSize(x, y),
            glfw::WindowEvent::Close => Event::WindowClose,
            glfw::WindowEvent::Refresh => Event::WindowRefresh,
            glfw::WindowEvent::Focus(focus) => Event::WindowFocus(focus),
            glfw::WindowEvent::Iconify(iconify) => Event::WindowIconify(iconify),
            glfw::WindowEvent::FramebufferSize(x, y) => Event::WindowFramebufferSize(x, y),
            glfw::WindowEvent::MouseButton(button, action, modifiers) => {
                Event::MouseButton(button.into(), action.into(), modifiers)
            }
            glfw::WindowEvent::CursorPos(x, y) => Event::CursorPosition(x, y),
            glfw::WindowEvent::CursorEnter(cursor_enter) => Event::CursorEnter(cursor_enter),
            glfw::WindowEvent::Scroll(x, y) => Event::Scroll(x, y),
            glfw::WindowEvent::Key(vk, _, action, modifiers) => {
                Event::Key(vk.into(), action.into(), modifiers)
            }
            glfw::WindowEvent::Char(c) => Event::Char(c),
            glfw::WindowEvent::CharModifiers(c, modifiers) => Event::CharModifiers(c, modifiers),
            glfw::WindowEvent::FileDrop(file_drop) => Event::WindowFileDrop(file_drop),
            glfw::WindowEvent::Maximize(maximize) => Event::WindowMaximize(maximize),
            glfw::WindowEvent::ContentScale(x, y) => Event::WindowContentScale(x, y),
        }
    }
}
