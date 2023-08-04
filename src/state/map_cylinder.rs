use std::f32::consts::PI;

use bytemuck::{Pod, Zeroable};
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutEntry,
    BindingType, Buffer, BufferBindingType, BufferDescriptor, BufferSize, BufferUsages, Device,
    Queue, ShaderStages,
};
use winit::{dpi::{PhysicalPosition, PhysicalSize}, event::ElementState};


const MERCATOR_SCALE : f32 = 2.794219;

#[repr(C)]
#[derive(Copy, Clone, Debug, Zeroable, Pod)]
struct MapCylinderUniform {
    rotation_matrix: [f32; 12],
    aspect_ratio: f32,
    padding: [f32; 3],
}

struct MouseState{
    position: PhysicalPosition<f64>,
    button_down: bool,
}

pub struct MapCylinderManager {
    latitude: f32,
    longitude: f32,
    mouse_state: MouseState,
    uniform: Buffer,
    pub bind_group_layout: BindGroupLayout,
    pub bind_group: BindGroup,
}

impl MapCylinderManager {
    pub fn new(device: &Device) -> Self {
        let uniform = Self::init_buffers(device);
        let bind_group_layout = Self::bind_group_layout(device);

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Map cylinder bind group"),
            layout: &bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: uniform.as_entire_binding(),
            }],
        });

        let mouse_state = MouseState{
            position: PhysicalPosition::new(0.0, 0.0),
            button_down: false,
        };

        Self {
            latitude: 0.0,
            longitude: 1.717,
            mouse_state,
            uniform,
            bind_group_layout,
            bind_group,
        }
    }

    fn bind_group_layout(device: &Device) -> wgpu::BindGroupLayout {
        let bind_group_layout = wgpu::BindGroupLayoutDescriptor {
            label: Some("ShapesBindGroupLayout"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: BufferSize::new(
                        std::mem::size_of::<MapCylinderUniform>() as u64
                    ),
                },
                count: None,
            }],
        };
        device.create_bind_group_layout(&bind_group_layout)
    }

    fn init_buffers(device: &Device) -> wgpu::Buffer {
        device.create_buffer(&BufferDescriptor {
            label: Some("Camera Uniform"),
            size: std::mem::size_of::<MapCylinderUniform>() as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }

    pub fn update_buffers(&mut self, queue: &Queue, window_size: PhysicalSize<u32>) {
        queue.write_buffer(
            &self.uniform,
            0,
            bytemuck::bytes_of(&self.generate_uniform(window_size)),
        );
    }

    fn generate_uniform(&self, window_size: PhysicalSize<u32>) -> MapCylinderUniform {
        let lat = self.latitude;
        let lon = self.longitude;
        MapCylinderUniform {
            aspect_ratio: window_size.width as f32 / window_size.height as f32,
            #[rustfmt::skip]
            rotation_matrix: [ 
                lon.cos()*lat.cos(), -lon.sin(), lat.sin()*lon.cos(), 0.0,
                lon.sin()*lat.cos(), lon.cos(),  lat.sin()*lon.sin(), 0.0,
                -lat.sin(),          0.0,        lat.cos(),           0.0,
            ],
            padding: [0.0; 3],
        }
    }

    pub fn handle_cursor_moved(&mut self, position: PhysicalPosition<f64>, window_size: PhysicalSize<u32>) {
        if self.mouse_state.button_down {
            let mercator_y = (position.y as f32 / window_size.height as f32 - 0.5) * 2.0;
            let latitude = (mercator_y/MERCATOR_SCALE).exp().atan() * 2.0 - PI/2.0;
            let scale = 1.0/latitude.cos();
            let delta_x = position.x - self.mouse_state.position.x;
            let delta_y = -position.y + self.mouse_state.position.y;
            self.longitude += delta_x as f32 / window_size.width as f32 * 2.0 * PI;
            self.latitude += delta_y as f32 * scale / window_size.height as f32 * 2.0 * PI;
        }
        self.mouse_state.position = position;
    }

    pub fn handle_mouse_input(&mut self, state: ElementState){
        match state {
            ElementState::Pressed => {
                self.mouse_state.button_down = true;
            }
            ElementState::Released => {
                self.mouse_state.button_down = false;
            }
        }
    }

    pub fn reset_cursor_position(&mut self, position: PhysicalPosition<f64>) {
        self.mouse_state.position = position;
    }

}
