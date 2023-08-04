mod earth_texture;
pub mod map_cylinder;
mod substates;

use wgpu::{Adapter, Surface, SurfaceConfiguration};
use winit::{dpi::PhysicalSize, event::*, window::Window};

use crate::state::substates::{default_color_target_state, default_vertex_state};

use self::{
    earth_texture::EarthTexture,
    map_cylinder::MapCylinderManager,
    substates::{DEFAULT_MULTISAMPLE_STATE, DEFAULT_PRIMITIVE_STATE},
};

pub struct State {
    #[allow(dead_code)]
    instance: wgpu::Instance,
    #[allow(dead_code)]
    adapter: wgpu::Adapter,
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub window: Window,
    render_pipeline: wgpu::RenderPipeline,
    earth_texture: EarthTexture,
    map_cylinder: MapCylinderManager,
}

impl State {
    pub async fn new(window: Window) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
        });

        let surface = unsafe { instance.create_surface(&window) }.unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::LowPower,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::downlevel_webgl2_defaults(),
                },
                None,
            )
            .await
            .unwrap();

        let config = Self::get_surface_configuration(&surface, &size, &adapter);
        surface.configure(&device, &config);

        let earth_texture = EarthTexture::new(&device, &queue);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let map_cylinder = MapCylinderManager::new(&device);

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &earth_texture.bind_group_layout,
                    &map_cylinder.bind_group_layout,
                ],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: default_vertex_state(&shader),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(default_color_target_state(config.format))],
            }),
            primitive: DEFAULT_PRIMITIVE_STATE,
            depth_stencil: None,
            multisample: DEFAULT_MULTISAMPLE_STATE,
            multiview: None,
        });
        Self {
            instance,
            adapter,
            surface,
            device,
            queue,
            config,
            size,
            window,
            render_pipeline,
            earth_texture,
            map_cylinder,
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

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        self.map_cylinder
            .update_buffers(&self.queue, self.window.inner_size());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: false,
                    },
                })],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.earth_texture.bind_group, &[]);
            render_pass.set_bind_group(1, &self.map_cylinder.bind_group, &[]);
            render_pass.draw(0..3, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    pub fn handle_event(&mut self, event: &Event<()>) -> bool {
        match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                self.resize(*size);
                true
            }
            Event::WindowEvent {
                event: WindowEvent::CursorMoved { position, .. },
                ..
            } => {
                log::warn!("{:?}", position);
                if self.window.has_focus() {
                    self.map_cylinder
                        .handle_cursor_moved(*position, self.window.inner_size());
                }
                true
            }
            Event::WindowEvent {
                event: WindowEvent::MouseInput { state, .. },
                ..
            } => {
                if self.window.has_focus() {
                    self.map_cylinder.handle_mouse_input(*state);
                }
                true
            }
            Event::WindowEvent {
                event: WindowEvent::Touch(touch),
                ..
            } => {
                log::warn!("{:?}", touch);
                if touch.phase == TouchPhase::Started {
                    self.map_cylinder.reset_cursor_position(touch.location);
                    self.map_cylinder.handle_mouse_input(ElementState::Pressed);
                } else if touch.phase == TouchPhase::Ended || touch.phase == TouchPhase::Cancelled {
                    self.map_cylinder.handle_mouse_input(ElementState::Released);
                } else if touch.phase == TouchPhase::Moved {
                    self.map_cylinder
                        .handle_cursor_moved(touch.location, self.window.inner_size());
                }
                true
            }
            _ => false,
        }
    }

    fn get_surface_configuration(
        surface: &Surface,
        size: &PhysicalSize<u32>,
        adapter: &Adapter,
    ) -> SurfaceConfiguration {
        let surface_caps = surface.get_capabilities(adapter);

        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(wgpu::TextureFormat::is_srgb)
            .unwrap_or(surface_caps.formats[0]);

        wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        }
    }
}
