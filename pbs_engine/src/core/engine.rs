use crate::core::scene::{SceneManager, Transition};
use crate::core::asset::AssetManager;
use crate::core::window::Window;
use crate::core::Settings;
use crate::core::timer::Timer;
use std::path::PathBuf;

pub mod event {
    use glfw;
    use std::path::PathBuf;
    use crate::engine::input::{MouseButton, Action, Modifiers};
    use crate::engine::input::Key;

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
        Quit
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
                glfw::WindowEvent::MouseButton(button, action, modifiers) => Event::MouseButton(button.into(), action.into(), modifiers),
                glfw::WindowEvent::CursorPos(x, y) => Event::CursorPosition(x, y),
                glfw::WindowEvent::CursorEnter(cursor_enter) => Event::CursorEnter(cursor_enter),
                glfw::WindowEvent::Scroll(x, y) => Event::Scroll(x, y),
                glfw::WindowEvent::Key(vk, _, action, modifiers) => Event::Key(vk.into(), action.into(), modifiers),
                glfw::WindowEvent::Char(c) => Event::Char(c),
                glfw::WindowEvent::CharModifiers(c, modifiers) => Event::CharModifiers(c, modifiers),
                glfw::WindowEvent::FileDrop(file_drop) => Event::WindowFileDrop(file_drop),
                glfw::WindowEvent::Maximize(maximize) => Event::WindowMaximize(maximize),
                glfw::WindowEvent::ContentScale(x, y) => Event::WindowContentScale(x, y)
            }
        }
    }
}

pub mod input {
    use glfw;

    #[derive(Debug, Copy, Clone)]
    pub enum MouseButton {
        Left,
        Right,
        Middle,
        Unsupported
    }

    #[derive(Debug, Copy, Clone)]
    pub enum Key {
        Space,
        Apostrophe,
        Comma,
        Minus,
        Period,
        Slash,
        Num0,
        Num1,
        Num2,
        Num3,
        Num4,
        Num5,
        Num6,
        Num7,
        Num8,
        Num9,
        Semicolon,
        Equal,
        A,
        B,
        C,
        D,
        E,
        F,
        G,
        H,
        I,
        J,
        K,
        L,
        M,
        N,
        O,
        P,
        Q,
        R,
        S,
        T,
        U,
        V,
        W,
        X,
        Y,
        Z,
        LeftBracket,
        Backslash,
        RightBracket,
        GraveAccent,
        World1,
        World2,
        Escape,
        Enter,
        Tab,
        Backspace,
        Insert,
        Delete,
        RightArrow,
        LeftArrow,
        DownArrow,
        UpArrow,
        PageUp,
        PageDown,
        Home,
        End,
        CapsLock,
        ScrollLock,
        NumLock,
        PrintScreen,
        Pause,
        F1,
        F2,
        F3,
        F4,
        F5,
        F6,
        F7,
        F8,
        F9,
        F10,
        F11,
        F12,
        F13,
        F14,
        F15,
        F16,
        F17,
        F18,
        F19,
        F20,
        F21,
        F22,
        F23,
        F24,
        F25,
        NumPad0,
        NumPad1,
        NumPad2,
        NumPad3,
        NumPad4,
        NumPad5,
        NumPad6,
        NumPad7,
        NumPad8,
        NumPad9,
        NumPadDecimal,
        NumPadDivide,
        NumPadMultiply,
        NumPadSubtract,
        NumPadAdd,
        NumPadEnter,
        NumPadEqual,
        LeftShift,
        LeftControl,
        LeftAlt,
        LeftSuper,
        RightShift,
        RightControl,
        RightAlt,
        RightSuper,
        Menu,
        Unknown
    }

    #[derive(Debug, Copy, Clone)]
    pub enum Action {
        Press,
        Release,
        Repeat
    }

    pub type Modifiers = glfw::Modifiers;

    impl From<glfw::MouseButton> for MouseButton {
        fn from(button: glfw::MouseButton) -> Self {
            match button {
                glfw::MouseButton::Button1 => MouseButton::Left,
                glfw::MouseButton::Button2 => MouseButton::Right,
                glfw::MouseButton::Button3 => MouseButton::Middle,
                _ => MouseButton::Unsupported
            }
        }
    }

    impl From<glfw::Key> for Key {
        fn from(key: glfw::Key) -> Self {
            match key {
                glfw::Key::Space => Key::Space,
                glfw::Key::Apostrophe => Key::Apostrophe,
                glfw::Key::Comma => Key::Comma,
                glfw::Key::Minus => Key::Minus,
                glfw::Key::Period => Key::Period,
                glfw::Key::Slash => Key::Slash,
                glfw::Key::Num0 => Key::Num0,
                glfw::Key::Num1 => Key::Num1,
                glfw::Key::Num2 => Key::Num2,
                glfw::Key::Num3 => Key::Num3,
                glfw::Key::Num4 => Key::Num4,
                glfw::Key::Num5 => Key::Num5,
                glfw::Key::Num6 => Key::Num6,
                glfw::Key::Num7 => Key::Num7,
                glfw::Key::Num8 => Key::Num8,
                glfw::Key::Num9 => Key::Num9,
                glfw::Key::Semicolon => Key::Semicolon,
                glfw::Key::Equal => Key::Equal,
                glfw::Key::A => Key::A,
                glfw::Key::B => Key::B,
                glfw::Key::C => Key::C,
                glfw::Key::D => Key::D,
                glfw::Key::E => Key::E,
                glfw::Key::F => Key::F,
                glfw::Key::G => Key::G,
                glfw::Key::H => Key::H,
                glfw::Key::I => Key::I,
                glfw::Key::J => Key::J,
                glfw::Key::K => Key::K,
                glfw::Key::L => Key::L,
                glfw::Key::M => Key::M,
                glfw::Key::N => Key::N,
                glfw::Key::O => Key::O,
                glfw::Key::P => Key::P,
                glfw::Key::Q => Key::Q,
                glfw::Key::R => Key::R,
                glfw::Key::S => Key::S,
                glfw::Key::T => Key::T,
                glfw::Key::U => Key::U,
                glfw::Key::V => Key::V,
                glfw::Key::W => Key::W,
                glfw::Key::X => Key::X,
                glfw::Key::Y => Key::Y,
                glfw::Key::Z => Key::Z,
                glfw::Key::LeftBracket => Key::LeftBracket,
                glfw::Key::Backslash => Key::Backslash,
                glfw::Key::RightBracket => Key::RightBracket,
                glfw::Key::GraveAccent => Key::GraveAccent,
                glfw::Key::World1 => Key::World1,
                glfw::Key::World2 => Key::World2,
                glfw::Key::Escape => Key::Escape,
                glfw::Key::Enter => Key::Enter,
                glfw::Key::Tab => Key::Tab,
                glfw::Key::Backspace => Key::Backspace,
                glfw::Key::Insert => Key::Insert,
                glfw::Key::Delete => Key::Delete,
                glfw::Key::Right => Key::RightArrow,
                glfw::Key::Left => Key::LeftArrow,
                glfw::Key::Down => Key::DownArrow,
                glfw::Key::Up => Key::UpArrow,
                glfw::Key::PageUp => Key::PageUp,
                glfw::Key::PageDown => Key::PageDown,
                glfw::Key::Home => Key::Home,
                glfw::Key::End => Key::End,
                glfw::Key::CapsLock => Key::CapsLock,
                glfw::Key::ScrollLock => Key::ScrollLock,
                glfw::Key::NumLock => Key::NumLock,
                glfw::Key::PrintScreen => Key::PrintScreen,
                glfw::Key::Pause => Key::Pause,
                glfw::Key::F1 => Key::F1,
                glfw::Key::F2 => Key::F2,
                glfw::Key::F3 => Key::F3,
                glfw::Key::F4 => Key::F4,
                glfw::Key::F5 => Key::F5,
                glfw::Key::F6 => Key::F6,
                glfw::Key::F7 => Key::F7,
                glfw::Key::F8 => Key::F8,
                glfw::Key::F9 => Key::F9,
                glfw::Key::F10 => Key::F10,
                glfw::Key::F11 => Key::F11,
                glfw::Key::F12 => Key::F12,
                glfw::Key::F13 => Key::F13,
                glfw::Key::F14 => Key::F14,
                glfw::Key::F15 => Key::F15,
                glfw::Key::F16 => Key::F16,
                glfw::Key::F17 => Key::F17,
                glfw::Key::F18 => Key::F18,
                glfw::Key::F19 => Key::F19,
                glfw::Key::F20 => Key::F20,
                glfw::Key::F21 => Key::F21,
                glfw::Key::F22 => Key::F22,
                glfw::Key::F23 => Key::F23,
                glfw::Key::F24 => Key::F24,
                glfw::Key::F25 => Key::F25,
                glfw::Key::Kp0 => Key::NumPad0,
                glfw::Key::Kp1 => Key::NumPad1,
                glfw::Key::Kp2 => Key::NumPad2,
                glfw::Key::Kp3 => Key::NumPad3,
                glfw::Key::Kp4 => Key::NumPad4,
                glfw::Key::Kp5 => Key::NumPad5,
                glfw::Key::Kp6 => Key::NumPad6,
                glfw::Key::Kp7 => Key::NumPad7,
                glfw::Key::Kp8 => Key::NumPad8,
                glfw::Key::Kp9 => Key::NumPad9,
                glfw::Key::KpDecimal => Key::NumPadDecimal,
                glfw::Key::KpDivide => Key::NumPadDivide,
                glfw::Key::KpMultiply => Key::NumPadMultiply,
                glfw::Key::KpSubtract => Key::NumPadSubtract,
                glfw::Key::KpAdd => Key::NumPadAdd,
                glfw::Key::KpEnter => Key::NumPadEnter,
                glfw::Key::KpEqual => Key::NumPadEqual,
                glfw::Key::LeftShift => Key::LeftShift,
                glfw::Key::LeftControl => Key::LeftControl,
                glfw::Key::LeftAlt => Key::LeftAlt,
                glfw::Key::LeftSuper => Key::LeftSuper,
                glfw::Key::RightShift => Key::RightShift,
                glfw::Key::RightControl => Key::RightControl,
                glfw::Key::RightAlt => Key::RightAlt,
                glfw::Key::RightSuper => Key::RightSuper,
                glfw::Key::Menu => Key::Menu,
                glfw::Key::Unknown => Key::Unknown,
            }
        }
    }

    impl From<glfw::Action> for Action {
        fn from(action: glfw::Action) -> Self {
            match action {
                glfw::Action::Release => Action::Release,
                glfw::Action::Press => Action::Press,
                glfw::Action::Repeat => Action::Repeat,
            }
        }
    }
}

pub struct Context<'a, T> {
    pub window: &'a mut Window,
    pub asset_manager: &'a mut AssetManager,
    pub timer: &'a mut Timer,
    pub settings: &'a mut Settings,
    pub user_data: &'a mut T
}


impl<'a, T> Context<'a, T> where T: 'a {
    pub fn new(window: &'a mut Window,
               asset_manager: &'a mut AssetManager,
               timer: &'a mut Timer,
               settings: &'a mut Settings,
               user_data: &'a mut T) -> Self {
        Self {
            window,
            asset_manager,
            timer,
            settings,
            user_data
        }
    }
}
