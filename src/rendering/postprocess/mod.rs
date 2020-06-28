use crate::imgui::{im_str, Gui, Ui};
use crate::rendering::framebuffer::Framebuffer;
use crate::{AsAny, AsAnyMut};

pub mod bloom;

pub trait PostprocessingEffect: Gui + AsAny + AsAnyMut {
    fn name(&self) -> &str;

    fn enable(&mut self);

    fn disable(&mut self);

    fn enabled(&self) -> bool;

    fn apply(&self, input: &Framebuffer);
}

pub struct PostprocessingStack {
    post_effects: Vec<Box<dyn PostprocessingEffect>>,
    enabled: bool,
}

impl PostprocessingStack {
    pub fn add_effect<T>(&mut self, effect: T) -> &mut Self
    where
        T: PostprocessingEffect + 'static,
    {
        self.post_effects.push(Box::new(effect));
        self
    }

    pub fn apply(&self, input: &Framebuffer) {
        if self.enabled {
            self.post_effects
                .iter()
                .filter(|&effect| effect.enabled())
                .for_each(|effect| effect.apply(&input));
        }
    }

    pub fn get_mut<T>(&mut self) -> Option<&mut T>
    where
        T: PostprocessingEffect + 'static,
    {
        self.post_effects
            .iter_mut()
            .filter_map(|effect| effect.as_any_mut().downcast_mut::<T>())
            .next()
    }

    pub fn get<T>(&self) -> Option<&T>
    where
        T: PostprocessingEffect + 'static,
    {
        self.post_effects
            .iter()
            .filter_map(|effect| effect.as_any().downcast_ref::<T>())
            .next()
    }
}

impl Gui for PostprocessingStack {
    fn gui(&mut self, ui: &Ui) {
        ui.checkbox(im_str!(""), &mut self.enabled);
        ui.same_line_with_spacing(30.0, 3.0);

        if imgui::CollapsingHeader::new(im_str!("Post-processing"))
            .default_open(true)
            .open_on_arrow(true)
            .open_on_double_click(true)
            .build(ui)
        {
            ui.spacing();
            ui.indent();

            self.post_effects
                .iter_mut()
                .for_each(|effect| effect.gui(ui));

            ui.unindent();
        }
    }
}

pub struct PostprocessingStackBuilder {
    post_effects: Vec<Box<dyn PostprocessingEffect>>,
    enabled: bool,
}

impl PostprocessingStackBuilder {
    pub fn new() -> Self {
        Self {
            post_effects: vec![],
            enabled: true,
        }
    }

    pub fn with_effect<T>(mut self, effect: T) -> Self
    where
        T: PostprocessingEffect + 'static,
    {
        self.post_effects.push(Box::new(effect));
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn build(self) -> PostprocessingStack {
        PostprocessingStack {
            post_effects: self.post_effects,
            enabled: self.enabled,
        }
    }
}
