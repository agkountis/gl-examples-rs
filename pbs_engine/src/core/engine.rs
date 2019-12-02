use crate::core::scene::{SceneManager, Transition};
use crate::core::asset::AssetManager;
use crate::core::window::Window;
use crate::core::Settings;
use crate::core::timer::Timer;
use std::path::PathBuf;

pub enum Event {
    Window(WindowEvent),
    Input(InputEvent),
    Quit
}

pub enum WindowEvent {
    Position(i32, i32),
    Size(i32, i32),
    Close,
    Refresh,
    Focus(bool),
    Iconify(bool),
    FramebufferSize(i32, i32),
    FileDrop(Vec<PathBuf>),
    Maximize(bool),
    ContentScale(f32, f32),
    CursorPosition(f64, f64),
    CursorEnter(bool)
}

pub enum InputEvent {
    MouseButton(MouseButton, Action, Vec<Modifier>),
    Key(Key, Action, Vec<Modifier>)
}

pub enum MouseButton {
    Left,
    Right,
    Middle
}

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
    CapsLock
    //TODO Continue
}

pub enum Action {
    Press,
    Release,
    Repeat
}

pub enum Modifier {
    Shift,
    Control,
    Alt,
    Super,
    CapsLock,
    NumLock
}

pub struct Context<'a, T> {
    pub window: &'a mut Window,
    pub asset_manager: &'a mut AssetManager,
    pub timer: &'a mut Timer,
    pub settings: &'a mut Settings,
    pub user_data: &'a mut T
}


impl<'a, T> Context<'a, T> {
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
