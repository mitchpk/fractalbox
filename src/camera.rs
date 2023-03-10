use std::time::Duration;

use cgmath::Point2;
use winit::event::{VirtualKeyCode, ElementState, MouseScrollDelta};

#[derive(Debug)]
pub struct Camera {
    pub position: Point2<f64>,
    zoom: f32,
    zoom_target: f32,
    pub aspect: f32,
}

impl Camera {
    pub fn new(position: impl Into<Point2<f64>>, zoom: f32, aspect: f32) -> Self {
        Self {
            position: position.into(),
            zoom,
            zoom_target: zoom,
            aspect,
        }
    }
}

fn lerp(start: f32, end: f32, percent: f32) -> f32 {
    start + (end - start) * percent
}

#[derive(Debug)]
pub struct CameraController {
    amount_right: f32,
    amount_down: f32,
    amount_in: f32,
    speed: f64,
}

impl CameraController {
    pub fn new(speed: f64) -> Self {
        Self {
            amount_right: 0.0,
            amount_down: 0.0,
            amount_in: 0.0,
            speed,
        }
    }

    pub fn process_keyboard(&mut self, key: VirtualKeyCode, state: ElementState) -> bool {
        let amount = if state == ElementState::Pressed {
            1.0
        } else {
            0.0
        };

        match key {
            VirtualKeyCode::Up => {
                self.amount_down = -amount;
                true
            }
            VirtualKeyCode::Down => {
                self.amount_down = amount;
                true
            }
            VirtualKeyCode::Left => {
                self.amount_right = -amount;
                true
            }
            VirtualKeyCode::Right => {
                self.amount_right = amount;
                true
            }
            _ => false,
        }
    }

    pub fn process_mouse(&mut self, scroll: MouseScrollDelta) {
        self.amount_in += match scroll {
            MouseScrollDelta::LineDelta(_, y) => y,
            MouseScrollDelta::PixelDelta(p) => p.y as f32 * 0.0001,
        }
    }

    pub fn update_camera(&mut self, camera: &mut Camera, dt: Duration) {
        let dt = dt.as_secs_f32();
        camera.position.x += self.amount_right as f64 * self.speed * (-camera.zoom as f64).exp() * dt as f64;
        camera.position.y += self.amount_down as f64 * self.speed * (-camera.zoom as f64).exp() * dt as f64;

        camera.zoom_target += self.amount_in * 0.2;
        camera.zoom = lerp(camera.zoom, camera.zoom_target, 5.0 * dt);
        self.amount_in = 0.0;
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub pos: [f64; 2],
    pub zoom: f32,
    pub aspect: f32,
    pub _padding: f64,
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            pos: [0.0; 2],
            zoom: 0.0,
            aspect: 1.0,
            _padding: 0.0,
        }
    }

    pub fn update(&mut self, camera: &Camera) {
        self.pos = camera.position.into();
        self.zoom = camera.zoom;
        self.aspect = camera.aspect;
    }
}
