use crate::rendering::framebuffer::Framebuffer;
use crate::{AsAny, AsAnyMut};

pub mod bloom;

pub trait PostprocessingEffect: AsAny + AsAnyMut {
    fn name(&self) -> &str;

    fn apply(&self, input: &Framebuffer);
}

pub struct PostprocessingStack {
    post_effects: Vec<Box<dyn PostprocessingEffect>>,
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
        self.post_effects
            .iter()
            .for_each(|effect| effect.apply(&input));
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

pub struct PostprocessingStackBuilder {
    post_effects: Vec<Box<dyn PostprocessingEffect>>,
}

impl PostprocessingStackBuilder {
    pub fn new() -> Self {
        Self {
            post_effects: vec![],
        }
    }

    pub fn with_effect<T>(mut self, effect: T) -> Self
    where
        T: PostprocessingEffect + 'static,
    {
        self.post_effects.push(Box::new(effect));
        self
    }

    pub fn build(self) -> PostprocessingStack {
        PostprocessingStack {
            post_effects: self.post_effects,
        }
    }
}
