use crate::core::Context;
use glutin::event::WindowEvent;
use imgui::Ui;

pub enum Transition {
    Push(Box<dyn Scene>),
    Switch(Box<dyn Scene>),
    Pop,
    None,
    Quit,
}

pub trait Scene {
    fn start(&mut self, context: Context) {}
    fn stop(&mut self, context: Context) {}
    fn pause(&mut self, context: Context) {}
    fn resume(&mut self, context: Context) {}
    fn handle_event(&mut self, context: Context, event: WindowEvent) -> Transition {
        Transition::None
    }
    fn update(&mut self, context: Context) -> Transition {
        Transition::None
    }
    fn pre_draw(&mut self, context: Context) {}
    fn draw(&mut self, context: Context) {}
    fn gui(&mut self, ui: &Ui) {}
    fn post_draw(&mut self, context: Context) {}
}

pub struct SceneManager {
    scenes: Vec<Box<dyn Scene>>,
    is_running: bool,
}

impl SceneManager {
    pub(crate) fn new<S: Scene + 'static>(initial_scene: S) -> SceneManager {
        Self {
            scenes: vec![Box::new(initial_scene)],
            is_running: false,
        }
    }

    pub fn initialize(&mut self, context: Context) {
        self.scenes.last_mut().unwrap().start(context);
        self.is_running = true
    }

    pub(crate) fn handle_event(&mut self, context: Context, event: WindowEvent) {
        let Context {
            window,
            asset_manager,
            timer,
            framebuffer_cache,
            settings,
        } = context;

        if self.is_running {
            let transition = match self.scenes.last_mut() {
                Some(scene) => scene.handle_event(
                    Context::new(window, asset_manager, timer, framebuffer_cache, settings),
                    event,
                ),
                None => Transition::None,
            };

            self.handle_transition(
                transition,
                Context::new(window, asset_manager, timer, framebuffer_cache, settings),
            );
        }
    }

    pub(crate) fn update(&mut self, context: Context) {
        let Context {
            window,
            asset_manager,
            timer,
            framebuffer_cache,
            settings,
        } = context;

        if self.is_running {
            let transition = match self.scenes.last_mut() {
                Some(scene) => scene.update(Context::new(
                    window,
                    asset_manager,
                    timer,
                    framebuffer_cache,
                    settings,
                )),
                None => Transition::None,
            };

            self.handle_transition(
                transition,
                Context::new(window, asset_manager, timer, framebuffer_cache, settings),
            )
        }
    }

    pub(crate) fn pre_draw(&mut self, context: Context) {
        let Context {
            window,
            asset_manager,
            timer,
            framebuffer_cache,
            settings,
        } = context;

        if self.is_running {
            if let Some(scene) = self.scenes.last_mut() {
                scene.pre_draw(Context::new(
                    window,
                    asset_manager,
                    timer,
                    framebuffer_cache,
                    settings,
                ))
            }
        }
    }

    pub(crate) fn draw(&mut self, context: Context) {
        let Context {
            window,
            asset_manager,
            timer,
            framebuffer_cache,
            settings,
        } = context;

        if self.is_running {
            if let Some(scene) = self.scenes.last_mut() {
                scene.draw(Context::new(
                    window,
                    asset_manager,
                    timer,
                    framebuffer_cache,
                    settings,
                ))
            }
        }
    }

    pub(crate) fn gui(&mut self, ui: &Ui) {
        if self.is_running {
            if let Some(scene) = self.scenes.last_mut() {
                scene.gui(ui)
            }
        }
    }

    pub(crate) fn post_draw(&mut self, context: Context) {
        let Context {
            window,
            asset_manager,
            timer,
            framebuffer_cache,
            settings,
        } = context;

        if self.is_running {
            if let Some(scene) = self.scenes.last_mut() {
                scene.post_draw(Context::new(
                    window,
                    asset_manager,
                    timer,
                    framebuffer_cache,
                    settings,
                ))
            }
        }
    }

    pub fn is_running(&self) -> bool {
        self.is_running
    }

    fn handle_transition(&mut self, transition: Transition, context: Context) {
        let Context {
            window,
            asset_manager,
            timer,
            framebuffer_cache,
            settings,
        } = context;

        match transition {
            Transition::Push(scene) => self.push(
                scene,
                Context::new(window, asset_manager, timer, framebuffer_cache, settings),
            ),
            Transition::Switch(_) => {}
            Transition::Pop => {}
            Transition::None => {}
            Transition::Quit => self.stop(Context::new(
                window,
                asset_manager,
                timer,
                framebuffer_cache,
                settings,
            )),
        }
    }

    fn push(&mut self, scene: Box<dyn Scene>, context: Context) {
        let Context {
            window,
            asset_manager,
            timer,
            framebuffer_cache,
            settings,
        } = context;

        if let Some(current) = self.scenes.last_mut() {
            current.pause(Context::new(
                window,
                asset_manager,
                timer,
                framebuffer_cache,
                settings,
            ))
        }

        self.scenes.push(scene);
        self.scenes.last_mut().unwrap().start(Context::new(
            window,
            asset_manager,
            timer,
            framebuffer_cache,
            settings,
        ))
    }

    pub(crate) fn stop(&mut self, context: Context) {
        if self.is_running {
            let Context {
                window,
                asset_manager,
                timer,
                framebuffer_cache,
                settings,
            } = context;

            while let Some(mut scene) = self.scenes.pop() {
                scene.stop(Context::new(
                    window,
                    asset_manager,
                    timer,
                    framebuffer_cache,
                    settings,
                ))
            }

            self.is_running = false;
        }
    }

    pub(crate) fn pause(&mut self, context: Context) {
        let Context {
            window,
            asset_manager,
            timer,
            framebuffer_cache,
            settings,
        } = context;

        if let Some(scene) = self.scenes.last_mut() {
            scene.pause(Context::new(
                window,
                asset_manager,
                timer,
                framebuffer_cache,
                settings,
            ))
        }
    }

    pub(crate) fn resume(&mut self, context: Context) {
        let Context {
            window,
            asset_manager,
            timer,
            framebuffer_cache,
            settings,
        } = context;

        if let Some(scene) = self.scenes.last_mut() {
            scene.resume(Context::new(
                window,
                asset_manager,
                timer,
                framebuffer_cache,
                settings,
            ))
        }
    }
}
