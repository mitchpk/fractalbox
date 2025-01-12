mod state;
use std::time::Instant;

use state::*;

mod camera;

use winit::{
    application::ApplicationHandler,
    event::*,
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::Window,
};

enum App {
    Uninitialised,
    Initialised {
        state: State,
        last_render_time: Instant,
        focused: bool,
    },
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window = event_loop
            .create_window(Window::default_attributes())
            .unwrap();
        let state = pollster::block_on(State::new(window));
        *self = App::Initialised {
            state,
            last_render_time: Instant::now(),
            focused: true,
        };
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        if let App::Initialised {
            state,
            last_render_time,
            focused,
        } = self
        {
            if state.input(&event) {
                return;
            }

            match event {
                WindowEvent::CloseRequested
                | WindowEvent::KeyboardInput {
                    event:
                        KeyEvent {
                            state: ElementState::Pressed,
                            physical_key: PhysicalKey::Code(KeyCode::Escape),
                            ..
                        },
                    ..
                } => event_loop.exit(),
                WindowEvent::Resized(physical_size) => {
                    state.resize(physical_size);
                }
                WindowEvent::Focused(focus) => *focused = focus,
                WindowEvent::RedrawRequested => {
                    state.window.request_redraw();
                    let now = Instant::now();
                    let dt = now - *last_render_time;
                    *last_render_time = now;
                    println!("{:?}", dt);
                    //std::thread::sleep(std::time::Duration::from_millis(30));
                    state.update(dt);
                    match state.render() {
                        Ok(_) => {}
                        // Reconfigure the surface if lost
                        Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                        // The system is out of memory, we should probably quit
                        Err(wgpu::SurfaceError::OutOfMemory) => event_loop.exit(),
                        // All other errors (Outdated, Timeout) should be resolved by the next frame
                        Err(e) => eprintln!("{:?}", e),
                    }
                }
                _ => {}
            }
        }
    }
}

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new().expect("failed to create event loop");
    let mut app = App::Uninitialised;
    event_loop.run_app(&mut app).expect("failure while running event loop");
}
