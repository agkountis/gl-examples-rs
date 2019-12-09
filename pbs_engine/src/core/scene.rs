use crate::core::engine::{Context, event::Event};
use std::borrow::BorrowMut;
use std::ops::DerefMut;

pub enum Transition<T> {
    Push(Box<dyn Scene<T>>),
    Switch(Box<dyn Scene<T>>),
    Pop,
    None,
    Quit
}

pub trait Scene<T> {
    fn start(&mut self, context: Context<T>) {}
    fn stop(&mut self, context: Context<T>) {}
    fn pause(&mut self, context: Context<T>) {}
    fn resume(&mut self, context: Context<T>) {}
    fn handle_event(&mut self, context: Context<T>, event: Event) -> Transition<T> { Transition::None }
    fn update(&mut self, context: Context<T>) -> Transition<T> { Transition::None }
    fn pre_draw(&mut self, context: Context<T>) {}
    fn draw(&mut self, context: Context<T>) {}
    fn post_draw(&mut self, context: Context<T>) {}
}

pub struct SceneManager<T> {
    scenes: Vec<Box<dyn Scene<T>>>,
    active_scene_index: usize,
    is_running: bool
}

impl<T> SceneManager<T> {
    pub fn new(initial_scene: Box<dyn Scene<T>>) -> Self {
        Self {
            scenes: vec![initial_scene],
            active_scene_index: 0,
            is_running: false
        }
    }

    pub fn initialize(&mut self, context: Context<T>) {
        self.scenes.last_mut().unwrap().start(context);
        self.is_running = true
    }

    pub fn handle_event(&mut self, context: Context<T>, event: Event) {
        let Context {
            window,
            asset_manager,
            timer,
            settings,
            user_data
        } = context;

        let transition = self.scenes[self.active_scene_index].handle_event(Context::new(window,
                                                                                        asset_manager,
                                                                                        timer,
                                                                                        settings,
                                                                                        user_data),
                                                                           event);

        match transition {
            Transition::Quit => self.stop(Context::new(window,
                                                       asset_manager,
                                                       timer,
                                                       settings,
                                                       user_data)),
            _ => {
                self.handle_transition(transition, Context::new(window,
                                                                asset_manager,
                                                                timer,
                                                                settings,
                                                                user_data))
            }
        }
    }

    pub fn update(&mut self, context: Context<T>) {
        let Context {
            window,
            asset_manager,
            timer,
            settings,
            user_data
        } = context;

        let transition = self.scenes[self.active_scene_index].update(Context::new(window,
                                                                                  asset_manager,
                                                                                  timer,
                                                                                  settings,
                                                                                  user_data));
        self.handle_transition(transition, Context::new(window,
                                                        asset_manager,
                                                        timer,
                                                        settings,
                                                        user_data));

        self.scenes[self.active_scene_index].pre_draw(Context::new(window,
                                                                   asset_manager,
                                                                   timer,
                                                                   settings,
                                                                   user_data));

        self.scenes[self.active_scene_index].draw(Context::new(window,
                                                               asset_manager,
                                                               timer,
                                                               settings,
                                                               user_data));

        self.scenes[self.active_scene_index].post_draw(Context::new(window,
                                                                    asset_manager,
                                                                    timer,
                                                                    settings,
                                                                    user_data))
    }

    pub fn is_running(&self) -> bool {
        self.is_running
    }

    fn handle_transition(&mut self, transition: Transition<T>, context: Context<T>) {
        match transition {
            Transition::Push(mut scene) => self.push(scene, context),
            Transition::Switch(_) => {},
            Transition::Pop => {},
            Transition::None => {},
            Transition::Quit => {},
        }
    }

    fn push(&mut self, scene: Box<dyn Scene<T>>, context: Context<T>) {
        let Context {
            window,
            asset_manager,
            timer,
            settings,
            user_data
        } = context;

        if let Some(current) = self.scenes.last_mut() {
            current.pause(Context::new(window,
                                       asset_manager,
                                       timer,
                                       settings,
                                       user_data))
        }

        self.scenes.push(scene);
        self.scenes.last_mut().unwrap().start(Context::new(window,
                                                           asset_manager,
                                                           timer,
                                                           settings,
                                                           user_data))
    }

    pub(crate) fn stop(&mut self, context: Context<T>) {
        if self.is_running {
            let Context {
                window,
                asset_manager,
                timer,
                settings,
                user_data
            } = context;

            while let Some(mut scene) = self.scenes.pop() {
                scene.stop(Context::new(window, asset_manager, timer, settings, user_data))
            }

            self.is_running = false;
        }
    }
}
