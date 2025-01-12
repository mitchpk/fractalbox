use std::time::Duration;

use cgmath::Point2;
use std::ops::*;
use winit::{
    event::{ElementState, MouseScrollDelta},
    keyboard::KeyCode,
};

#[derive(Debug)]
pub struct Camera {
    position: Point2<f64>,
    position_target: Point2<f64>,
    zoom: f32,
    zoom_target: f32,
    pub aspect: f32,
}

impl Camera {
    pub fn new(position: impl Into<Point2<f64>> + Clone, zoom: f32, aspect: f32) -> Self {
        Self {
            position: position.clone().into(),
            position_target: position.into(),
            zoom,
            zoom_target: zoom,
            aspect,
        }
    }
}

fn lerp<T, F>(start: T, end: T, percent: F) -> T
where
    T: Clone + Add<T, Output = T> + Sub<T, Output = T> + Mul<F, Output = T>,
{
    start.clone() + (end - start) * percent
}

#[derive(Debug)]
pub struct CameraController {
    amount_left: f32,
    amount_right: f32,
    amount_up: f32,
    amount_down: f32,
    amount_in: f32,
    amount_out: f32,
    speed: f32,
}

impl CameraController {
    pub fn new(speed: f32) -> Self {
        Self {
            amount_left: 0.0,
            amount_right: 0.0,
            amount_up: 0.0,
            amount_down: 0.0,
            amount_in: 0.0,
            amount_out: 0.0,
            speed,
        }
    }

    pub fn process_keyboard(&mut self, key: KeyCode, state: ElementState) -> bool {
        let amount = if state == ElementState::Pressed {
            1.0
        } else {
            0.0
        };

        match key {
            KeyCode::ArrowUp => {
                self.amount_up = amount;
                true
            }
            KeyCode::ArrowDown => {
                self.amount_down = amount;
                true
            }
            KeyCode::ArrowLeft => {
                self.amount_left = amount;
                true
            }
            KeyCode::ArrowRight => {
                self.amount_right = amount;
                true
            }
            KeyCode::KeyQ => {
                self.amount_out = amount;
                true
            }
            KeyCode::KeyW => {
                self.amount_in = amount;
                true
            }
            KeyCode::Equal => {
                self.speed *= 1.2;
                true
            }
            KeyCode::Minus => {
                self.speed /= 1.2;
                true
            }
            _ => false,
        }
    }

    pub fn process_mouse(&mut self, _scroll: MouseScrollDelta) {}

    pub fn update_camera(&mut self, camera: &mut Camera, dt: Duration) {
        let dt = dt.as_secs_f32();
        camera.position_target.x +=
            ((self.amount_right - self.amount_left) * self.speed * (-camera.zoom).exp() * dt)
                as f64;
        camera.position_target.y +=
            ((self.amount_down - self.amount_up) * self.speed * (-camera.zoom).exp() * dt) as f64;
        camera.position.x = lerp(camera.position.x, camera.position_target.x, 5.0 * dt as f64);
        camera.position.y = lerp(camera.position.y, camera.position_target.y, 5.0 * dt as f64);

        camera.zoom_target += (self.amount_in - self.amount_out) * self.speed * 0.5 * dt;
        camera.zoom = lerp(camera.zoom, camera.zoom_target, 5.0 * dt);
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
