use crate::core::engine::{event::Event, Context};

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
    fn handle_event(&mut self, context: Context, event: Event) -> Transition {
        Transition::None
    }
    fn update(&mut self, context: Context) -> Transition {
        Transition::None
    }
    fn pre_draw(&mut self, context: Context) {}
    fn draw(&mut self, context: Context) {}
    fn post_draw(&mut self, context: Context) {}
}

pub struct SceneManager<'a> {
    scenes: Vec<Box<dyn Scene + 'a>>,
    active_scene_index: usize,
    is_running: bool,
}

impl<'a> SceneManager<'a> {
    pub fn new<S: Scene + 'a>(initial_scene: S) -> SceneManager<'a> {
        Self {
            scenes: vec![Box::new(initial_scene)],
            active_scene_index: 0,
            is_running: false,
        }
    }

    pub fn initialize(&mut self, context: Context) {
        self.scenes.last_mut().unwrap().start(context);
        self.is_running = true
    }

    pub fn handle_event(&mut self, context: Context, event: Event) {
        let Context {
            window,
            asset_manager,
            timer,
            settings,
        } = context;

        if self.is_running {
            let transition = match self.scenes.last_mut() {
                Some(scene) => scene.handle_event(
                    Context::new(window, asset_manager, timer, settings),
                    event,
                ),
                None => Transition::None,
            };

            self.handle_transition(
                transition,
                Context::new(window, asset_manager, timer, settings),
            );
        }
    }

    pub fn update(&mut self, context: Context) {
        let Context {
            window,
            asset_manager,
            timer,
            settings,
        } = context;

        if self.is_running {
            let transition = match self.scenes.last_mut() {
                Some(scene) => scene.update(Context::new(
                    window,
                    asset_manager,
                    timer,
                    settings,
                )),
                None => Transition::None,
            };

            self.handle_transition(
                transition,
                Context::new(window, asset_manager, timer, settings),
            )
        }
    }

    pub fn pre_draw(&mut self, context: Context) {
        let Context {
            window,
            asset_manager,
            timer,
            settings,
        } = context;

        if self.is_running {
            if let Some(scene) = self.scenes.last_mut() {
                scene.pre_draw(Context::new(
                    window,
                    asset_manager,
                    timer,
                    settings,
                ))
            }
        }
    }

    pub fn draw(&mut self, context: Context) {
        let Context {
            window,
            asset_manager,
            timer,
            settings,
        } = context;

        if self.is_running {
            if let Some(scene) = self.scenes.last_mut() {
                scene.draw(Context::new(
                    window,
                    asset_manager,
                    timer,
                    settings,
                ))
            }
        }
    }

    pub fn post_draw(&mut self, context: Context) {
        let Context {
            window,
            asset_manager,
            timer,
            settings,
        } = context;

        if self.is_running {
            if let Some(scene) = self.scenes.last_mut() {
                scene.post_draw(Context::new(
                    window,
                    asset_manager,
                    timer,
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
            settings,
        } = context;

        match transition {
            Transition::Push(mut scene) => self.push(
                scene,
                Context::new(window, asset_manager, timer, settings),
            ),
            Transition::Switch(_) => {}
            Transition::Pop => {}
            Transition::None => {}
            Transition::Quit => self.stop(Context::new(
                window,
                asset_manager,
                timer,
                settings,
            )),
        }
    }

    fn push(&mut self, scene: Box<dyn Scene>, context: Context) {
        let Context {
            window,
            asset_manager,
            timer,
            settings,
        } = context;

        if let Some(current) = self.scenes.last_mut() {
            current.pause(Context::new(
                window,
                asset_manager,
                timer,
                settings,
            ))
        }

        self.scenes.push(scene);
        self.scenes.last_mut().unwrap().start(Context::new(
            window,
            asset_manager,
            timer,
            settings,
        ))
    }

    pub(crate) fn stop(&mut self, context: Context) {
        if self.is_running {
            let Context {
                window,
                asset_manager,
                timer,
                settings,
            } = context;

            while let Some(mut scene) = self.scenes.pop() {
                scene.stop(Context::new(
                    window,
                    asset_manager,
                    timer,
                    settings,
                ))
            }

            self.is_running = false;
        }
    }
}
