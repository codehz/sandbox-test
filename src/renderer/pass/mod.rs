use super::{buffers::SurfaceProvider, camera::Camera};
use crate::world::Map;

use bevy_app::App;
use bevy_ecs::prelude::*;
use hlist::{Cons, Nil};

pub mod cube;
pub mod debug;
pub mod outline;
pub mod sprite;
pub mod strengthen;

mod pp;

pub struct PassContext<'a> {
    pub world: &'a mut World,
    pub view_model: [[f32; 4]; 4],
    pub perspective: [[f32; 4]; 4],
    pub aspect_ratio: f32,
}

impl<'a> PassContext<'a> {
    pub fn get_thread_local_res<T: 'static>(&'a self) -> &'a T {
        self.world.get_non_send_resource().unwrap()
    }
    pub fn get_res<T: 'static + Send + Sync>(&'a self) -> &'a T {
        self.world.get_resource().unwrap()
    }
    // pub fn for_each<Q, F>(&mut self, func: F)
    // where
    //     Q: WorldQuery,
    //     Q::Fetch: ReadOnlyFetch,
    //     F: FnMut(<Q::Fetch as Fetch<'_>>::Item),
    // {
    //     let mut query = self.world.query::<Q>();
    //     query.for_each(self.world, func)
    // }
}

impl<'a> PassContext<'a> {
    pub fn camera(&'a self) -> &'a Camera {
        self.get_res()
    }
    pub fn map(&'a self) -> &'a Map {
        self.get_res()
    }
}

impl<'a> PassContext<'a> {
    pub fn create(app: &'a mut App, display: &glium::Display) -> Self {
        let aspect_ratio = {
            let dim = display.get_framebuffer_dimensions();
            dim.0 as f32 / dim.1 as f32
        };
        let camera: &Camera = app.world.get_resource().unwrap();
        let view_model = camera.view_model().to_cols_array_2d();
        let perspective = camera.perspective(aspect_ratio).to_cols_array_2d();
        Self {
            world: &mut app.world,
            view_model,
            perspective,
            aspect_ratio,
        }
    }
}

pub trait Pass
where
    Self: Sized,
{
    fn new(display: &glium::Display) -> anyhow::Result<Self>;

    fn prepare(&mut self, context: &mut PassContext<'_>, display: &glium::Display);

    fn process(
        &self,
        context: &mut PassContext<'_>,
        provider: &SurfaceProvider,
        display: &glium::Display,
    ) -> anyhow::Result<()>;
}

impl Pass for Nil {
    #[inline(always)]
    fn new(_display: &glium::Display) -> anyhow::Result<Self> {
        Ok(Nil)
    }

    #[inline(always)]
    fn prepare(&mut self, _context: &mut PassContext<'_>, _display: &glium::Display) {}

    #[inline(always)]
    fn process(
        &self,
        _context: &mut PassContext<'_>,
        _provider: &SurfaceProvider,
        _display: &glium::Display,
    ) -> anyhow::Result<()> {
        Ok(())
    }
}

impl<T, R> Pass for Cons<T, R>
where
    T: Pass,
    R: Pass,
{
    #[inline(always)]
    fn new(display: &glium::Display) -> anyhow::Result<Self> {
        Ok(Cons(T::new(display)?, R::new(display)?))
    }

    #[inline(always)]
    fn prepare(&mut self, context: &mut PassContext<'_>, display: &glium::Display) {
        self.0.prepare(context, display);
        self.1.prepare(context, display);
    }

    #[inline(always)]
    fn process(
        &self,
        context: &mut PassContext<'_>,
        provider: &SurfaceProvider,
        display: &glium::Display,
    ) -> anyhow::Result<()> {
        self.0.process(context, provider, display)?;
        self.1.process(context, provider, display)
    }
}
