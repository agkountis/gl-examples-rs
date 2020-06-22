use gl::types::*;
use gl_bindings as gl;

use crate::{
    core::{
        asset::AssetManager,
        math::Vec4,
        scene::{Scene, SceneManager},
        timer::Timer,
        Context, Settings,
    },
    Msaa,
};
use glutin::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Fullscreen, Window, WindowBuilder},
    Api, ContextBuilder, ContextWrapper, GlProfile, GlRequest, PossiblyCurrent,
};
use std::{error::Error, ffi::CStr, ptr};

pub struct Application;

impl Application {
    pub fn run<Cons, S>(settings: Settings, mut scene_constructor: Cons)
    where
        S: Scene + 'static,
        Cons: FnMut(Context) -> S,
    {
        let mut asset_manager = AssetManager::new();
        let mut timer = Timer::new();

        let (event_loop, mut windowed_context) = Self::create_windowed_context(&settings).unwrap();

        let initial_scene = scene_constructor(Context::new(
            windowed_context.window(),
            &mut asset_manager,
            &mut timer,
            &settings,
        ));

        let mut scene_manager = SceneManager::new(initial_scene);
        scene_manager.initialize(Context::new(
            windowed_context.window(),
            &mut asset_manager,
            &mut timer,
            &settings,
        ));

        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Poll;
            match event {
                Event::NewEvents(_) => {}
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => *control_flow = ControlFlow::Exit,
                Event::WindowEvent { event, .. } => scene_manager.handle_event(
                    Context::new(
                        windowed_context.window(),
                        &mut asset_manager,
                        &mut timer,
                        &settings,
                    ),
                    event,
                ),
                Event::DeviceEvent { .. } => {}
                Event::UserEvent(_) => {}
                Event::Suspended => scene_manager.pause(Context::new(
                    windowed_context.window(),
                    &mut asset_manager,
                    &mut timer,
                    &settings,
                )),
                Event::Resumed => scene_manager.resume(Context::new(
                    windowed_context.window(),
                    &mut asset_manager,
                    &mut timer,
                    &settings,
                )),
                Event::MainEventsCleared => {
                    scene_manager.update(Context::new(
                        windowed_context.window(),
                        &mut asset_manager,
                        &mut timer,
                        &settings,
                    ));

                    scene_manager.draw(Context::new(
                        windowed_context.window(),
                        &mut asset_manager,
                        &mut timer,
                        &settings,
                    ));
                    windowed_context.swap_buffers().unwrap()
                }
                Event::RedrawRequested(_) => {}
                Event::RedrawEventsCleared => {}
                Event::LoopDestroyed => scene_manager.stop(Context::new(
                    windowed_context.window(),
                    &mut asset_manager,
                    &mut timer,
                    &settings,
                )),
            }
        });
    }

    fn create_windowed_context(
        settings: &Settings,
    ) -> Result<(EventLoop<()>, ContextWrapper<PossiblyCurrent, Window>), Box<dyn Error>> {
        assert!(
            settings.graphics_api_version.major > 3 && settings.graphics_api_version.minor > 2,
            "Only OpenGL version greater than 3.2 are supported"
        );

        assert!(
            settings.graphics_api_version.major <= 4 && settings.graphics_api_version.minor <= 6,
            "OpenGL versions greater than 4.6 are not supported"
        );

        let event_loop = EventLoop::new();
        let mut window_builder = WindowBuilder::new()
            .with_title(&settings.name)
            .with_inner_size(LogicalSize::new(
                settings.window_size.x,
                settings.window_size.y,
            ))
            .with_resizable(false);

        if settings.fullscreen {
            let monitor = (&event_loop).available_monitors().next().unwrap();
            let video_mode = monitor.video_modes().next().unwrap();
            window_builder = window_builder.with_fullscreen(Some(Fullscreen::Exclusive(video_mode)))
        }

        let windowed_context = ContextBuilder::new()
            .with_double_buffer(Some(true))
            .with_gl_profile(GlProfile::Core)
            .with_srgb(true)
            .with_multisampling(match settings.msaa {
                Msaa::None => 0,
                Msaa::X4 => 4,
                Msaa::X8 => 8,
                Msaa::X16 => 16,
            })
            .with_vsync(settings.vsync)
            .with_gl(GlRequest::Specific(
                Api::OpenGl,
                (
                    settings.graphics_api_version.major as u8,
                    settings.graphics_api_version.minor as u8,
                ),
            ))
            .build_windowed(window_builder, &event_loop)?;

        let windowed_context = unsafe { windowed_context.make_current().unwrap() };

        gl::load_with(|s| windowed_context.get_proc_address(s) as *const _);

        unsafe {
            gl::Enable(gl::DEPTH_TEST);
            gl::Enable(gl::CULL_FACE);
            gl::Enable(gl::MULTISAMPLE);
            gl::Enable(gl::FRAMEBUFFER_SRGB);
            gl::Enable(gl::TEXTURE_CUBE_MAP_SEAMLESS);

            gl::Viewport(
                0,
                0,
                settings.window_size.x as i32,
                settings.window_size.y as i32,
            );

            if cfg!(debug_assertions) {
                gl::Enable(gl::DEBUG_OUTPUT);
                gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
                gl::DebugMessageCallback(Self::debug_callback, ptr::null());
            }
        }

        Ok((event_loop, windowed_context))
    }

    extern "system" fn debug_callback(
        source: GLenum,
        message_type: GLenum,
        id: GLuint,
        severity: GLenum,
        length: GLsizei,
        message: *const GLchar,
        user_param: *mut GLvoid,
    ) {
        let msg = unsafe { CStr::from_ptr(message) };

        eprintln!(
            "GL CALLBACK: type = {}, severity = {}, message = {:#?}",
            Self::message_type_to_str(message_type),
            Self::severity_to_str(severity),
            msg
        )
    }

    fn message_type_to_str(message_type: GLenum) -> &'static str {
        match message_type {
            gl::DEBUG_TYPE_DEPRECATED_BEHAVIOR => "DEPRECATED BEHAVIOR",
            gl::DEBUG_TYPE_ERROR => "ERROR",
            gl::DEBUG_TYPE_MARKER => "MARKER",
            gl::DEBUG_TYPE_OTHER => "OTHER",
            gl::DEBUG_TYPE_PERFORMANCE => "PERFORMANCE",
            gl::DEBUG_TYPE_POP_GROUP => "POP GROUP",
            gl::DEBUG_TYPE_PORTABILITY => "PORTABILITY",
            gl::DEBUG_TYPE_PUSH_GROUP => "PUSH GROUP",
            gl::DEBUG_TYPE_UNDEFINED_BEHAVIOR => "UNDEFINED BEHAVIOR",
            _ => "!UNDEFINED ENUM!",
        }
    }

    fn severity_to_str(severity: GLenum) -> &'static str {
        match severity {
            gl::DEBUG_SEVERITY_HIGH => "HIGH",
            gl::DEBUG_SEVERITY_MEDIUM => "MEDIUM",
            gl::DEBUG_SEVERITY_LOW => "LOW",
            gl::DEBUG_SEVERITY_NOTIFICATION => "NOTIFICATION",
            _ => "!UNDEFINED ENUM!",
        }
    }
}

pub fn clear_default_framebuffer(color: &Vec4) {
    unsafe {
        gl::ClearColor(color.x, color.y, color.z, color.w);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
    }
}
