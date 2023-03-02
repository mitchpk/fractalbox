use wgpu::util::DeviceExt;
use winit::{window::Window, event::{Event, DeviceEvent, KeyboardInput, WindowEvent, ElementState}};
use crate::camera::{Camera, CameraController, CameraUniform};
use anyhow::{Result, Context};

const FULLSCREEN_VERTICES: &[[f32; 3]] = &[
    [-1.0, 1.0, 0.0],
    [1.0, 1.0, 0.0],
    [-1.0, -1.0, 0.0],
    [1.0, -1.0, 0.0],
];
const FULLSCREEN_INDICES: &[u16] = &[0, 2, 1, 1, 2, 3];

pub struct State {
    instance: wgpu::Instance,
    adapter: wgpu::Adapter,
    surface: wgpu::Surface,
    pub device: wgpu::Device,
    queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    camera: Camera,
    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    camera_controller: CameraController,
    mouse_pressed: bool,
    fullscreen_pipeline: wgpu::RenderPipeline,
    fullscreen_bind_group: wgpu::BindGroup,
    fullscreen_vertex_buffer: wgpu::Buffer,
    fullscreen_index_buffer: wgpu::Buffer,
    frame_count: f32,
    frame_count_buffer: wgpu::Buffer,
    utils_bind_group: wgpu::BindGroup,
}

impl State {
    pub async fn new(window: &Window) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        let surface = unsafe { instance.create_surface(window).expect("failed to create surface") };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::SHADER_FLOAT64,
                    limits: wgpu::Limits::default(),
                    label: None,
                },
                None,
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats.iter()
            .copied()
            .filter(|f| f.describe().srgb)
            .next()
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        println!("Output config: {:#?}", config);

        let fullscreen_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[],
                label: Some("fullscreen_bind_group_layout"),
            }
        );

        let fullscreen_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &fullscreen_bind_group_layout,
            entries: &[],
            label: Some("fullscreen_bind_group"),
        });

        let camera = Camera::new((0.0, 0.0), 1.0);
        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update(&camera);

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("camera_buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("camera_bind_group_layout"),
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        let camera_controller = CameraController::new(1.0);

        let frame_count_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("frame_count_buffer"),
            contents: bytemuck::bytes_of(&0.0f32),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let utils_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("utils_bind_group_layout"),
            });

        let utils_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &utils_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: frame_count_buffer.as_entire_binding(),
            }],
            label: Some("utils_bind_group"),
        });

        let fullscreen_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("fullscreen_shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("fullscreen.wgsl").into()),
        });

        let fullscreen_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Fullscreen Pipeline Layout"),
                bind_group_layouts: &[
                    &fullscreen_bind_group_layout,
                    &camera_bind_group_layout,
                    &utils_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });

        let fullscreen_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Fullscreen Render Pipeline"),
            layout: Some(&fullscreen_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &fullscreen_shader,
                entry_point: "vs_main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: (std::mem::size_of::<f32>() * 3) as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[wgpu::VertexAttribute {
                        offset: 0,
                        shader_location: 0,
                        format: wgpu::VertexFormat::Float32x3,
                    }],
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &fullscreen_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    // Final view
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        let fullscreen_vertex_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Fullscreen Vertex Buffer"),
                contents: bytemuck::cast_slice(FULLSCREEN_VERTICES),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let fullscreen_index_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Fullscreen Index buffer"),
                contents: bytemuck::cast_slice(FULLSCREEN_INDICES),
                usage: wgpu::BufferUsages::INDEX,
            });

        return Self {
            instance,
            adapter,
            surface,
            device,
            queue,
            config,
            size,
            camera,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            camera_controller,
            mouse_pressed: false,
            fullscreen_pipeline,
            fullscreen_bind_group,
            fullscreen_vertex_buffer,
            fullscreen_index_buffer,
            frame_count: 0.0,
            frame_count_buffer,
            utils_bind_group,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn input<T: std::fmt::Debug>(&mut self, event: &Event<T>) -> bool {
        match event {
            Event::DeviceEvent { event, .. } => match event {
                DeviceEvent::MouseWheel { delta } => {
                    self.camera_controller.process_mouse(*delta);
                    true
                }

                _ => false,
            },

            Event::WindowEvent { event, .. } => match event {
                WindowEvent::MouseInput {
                    button: winit::event::MouseButton::Left,
                    state,
                    ..
                } => {
                    self.mouse_pressed = *state == ElementState::Pressed;
                    true
                }

                WindowEvent::MouseWheel { delta, .. } => {
                    self.camera_controller.process_mouse(*delta);
                    true
                }

                WindowEvent::KeyboardInput {
                    input: KeyboardInput {
                        virtual_keycode: Some(key),
                        state,
                        ..
                    },
                    ..
                } => self.camera_controller.process_keyboard(*key, *state),

                _ => false,
            },

            _ => false,
        }
    }

    pub fn update(&mut self, dt: std::time::Duration) {
        self.camera_controller.update_camera(&mut self.camera, dt);
        self.camera_uniform
            .update(&self.camera);
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
        self.queue.write_buffer(
            &self.frame_count_buffer,
            0,
            bytemuck::bytes_of(&self.frame_count),
        )
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        {
            let mut fullscreen_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Fullscreen Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            fullscreen_pass.set_pipeline(&self.fullscreen_pipeline);
            fullscreen_pass.set_bind_group(0, &self.fullscreen_bind_group, &[]);
            fullscreen_pass.set_bind_group(1, &self.camera_bind_group, &[]);
            fullscreen_pass.set_bind_group(2, &self.utils_bind_group, &[]);
            fullscreen_pass.set_vertex_buffer(0, self.fullscreen_vertex_buffer.slice(..));
            fullscreen_pass.set_index_buffer(
                self.fullscreen_index_buffer.slice(..),
                wgpu::IndexFormat::Uint16,
            );
            fullscreen_pass.draw_indexed(0..FULLSCREEN_INDICES.len() as u32, 0, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        self.frame_count += 1.0;
        Ok(())
    }
}
