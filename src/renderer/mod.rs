use self::{buffers::SurfaceProvider, events::*, pass::Pass};
use bevy_app::{App, AppExit, EventReader, EventWriter, Events, ManualEventReader, Plugin};
use bevy_ecs::prelude::*;
use glium::glutin::{self, event::KeyboardInput};
use pass::PassContext;

pub mod buffers;
pub mod camera;
pub mod events;
pub mod pass;

#[derive(Debug)]
pub enum Action {
    Exit,
    CaptureMouse(bool),
}

#[derive(Debug, Clone, Copy)]
pub struct RenderingConfig {}

#[macro_export]
macro_rules! pipeline {
    () => (::hlist::Nil);
    ($a:ty) => (::hlist::Cons<$a, ::hlist::Nil>);
    ($a:ty, $($rest:tt)*) => (::hlist::Cons<$a, $crate::pipeline!($($rest)*)>);
}

fn convert_appexit_to_action(
    mut appexit_event: EventReader<AppExit>,
    mut action_event: EventWriter<Action>,
) {
    for _ in appexit_event.iter() {
        action_event.send(Action::Exit);
    }
}

#[derive(Debug)]
pub struct RenderPlugin<P: Pass + 'static>(std::marker::PhantomData<fn(P)>);

impl<P: Pass + 'static> Plugin for RenderPlugin<P> {
    fn build(&self, appb: &mut bevy_app::AppBuilder) {
        appb.add_event::<FocusedEvent>()
            .add_event::<MouseButtonEvent>()
            .add_event::<MouseMotionEvent>()
            .add_event::<KeyboardInput>()
            .add_event::<Action>()
            .add_system(convert_appexit_to_action.system())
            .set_runner(RenderPlugin::<P>::run);
    }
}

impl<P: Pass + 'static> Default for RenderPlugin<P> {
    fn default() -> Self {
        Self(std::marker::PhantomData)
    }
}

fn send_event<E: 'static + Sync + Send>(app: &mut App, e: E) {
    if let Some(mut sender) = app.world.get_resource_mut::<Events<_>>() {
        sender.send(e);
    }
}

impl<P: Pass + 'static> RenderPlugin<P> {
    fn run(mut app: App) {
        let event_loop = glutin::event_loop::EventLoop::new();
        let mut wb = glutin::window::WindowBuilder::new();
        if let Some(data) = app
            .world
            .get_non_send_resource::<glutin::window::WindowAttributes>()
        {
            wb.window = (*data).to_owned();
        }
        let cb = glutin::ContextBuilder::new();
        let display = glium::Display::new(wb, cb, &event_loop).unwrap();

        let mut context = PassContext::create(&mut app, &display);
        let mut pass = P::new(&mut context, &display).unwrap();
        let mut provider = SurfaceProvider::new(&display).unwrap();
        let mut action_reader = ManualEventReader::<Action>::default();

        event_loop.run(move |event, _, control_flow| {
            *control_flow = glutin::event_loop::ControlFlow::Poll;
            use glutin::event::*;
            match event {
                Event::MainEventsCleared => {
                    app.update();
                    let window = display.gl_window();
                    let window = window.window();
                    let action_events = app.world.get_resource_mut().unwrap();
                    for action in action_reader.iter(&action_events) {
                        match action {
                            Action::Exit => {
                                *control_flow = glutin::event_loop::ControlFlow::Exit;
                                return;
                            }
                            Action::CaptureMouse(capture) => {
                                window
                                    .set_cursor_grab(*capture)
                                    .expect("failed to capture mouse");
                                window.set_cursor_visible(!*capture);
                                let size = window.inner_size();
                                window
                                    .set_cursor_position(glutin::dpi::PhysicalPosition::new(
                                        size.width / 2,
                                        size.height / 2,
                                    ))
                                    .expect("failed to set cursor position");
                            }
                        }
                    }
                    window.request_redraw();
                }
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => {
                        *control_flow = glutin::event_loop::ControlFlow::Exit;
                    }
                    WindowEvent::Focused(focused) => {
                        send_event(&mut app, FocusedEvent(focused));
                    }
                    WindowEvent::MouseInput { state, button, .. } => {
                        send_event(&mut app, MouseButtonEvent::new(button, state));
                    }
                    _ => {}
                },
                Event::RedrawRequested(_) => {
                    provider
                        .verify(&display)
                        .expect("Failed to resize framebuffer");
                    let mut context = PassContext::create(&mut app, &display);
                    pass.prepare(&mut context, &display);
                    pass.process(&mut context, &provider, &display).unwrap();
                }
                Event::DeviceEvent {
                    device_id: _,
                    event,
                } => {
                    match event {
                        DeviceEvent::MouseMotion { delta } => {
                            send_event(&mut app, MouseMotionEvent(delta.0 as f32, delta.1 as f32));
                        }
                        DeviceEvent::Key(keyboard) => {
                            send_event(&mut app, keyboard);
                        }
                        _ => (),
                    }
                    return;
                }
                _ => return,
            }
        });
    }
}
