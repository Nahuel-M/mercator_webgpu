use wgpu::{ShaderModule, TextureFormat, VertexState, ColorTargetState};

pub const DEFAULT_PRIMITIVE_STATE: wgpu::PrimitiveState = wgpu::PrimitiveState {
    topology: wgpu::PrimitiveTopology::TriangleList,
    strip_index_format: None,
    front_face: wgpu::FrontFace::Ccw,
    cull_mode: Some(wgpu::Face::Back),
    polygon_mode: wgpu::PolygonMode::Fill,
    unclipped_depth: false,
    conservative: false,
};

pub const DEFAULT_MULTISAMPLE_STATE: wgpu::MultisampleState = wgpu::MultisampleState {
    count: 1,
    mask: !0,
    alpha_to_coverage_enabled: false,
};

pub fn default_color_target_state(format: TextureFormat) -> ColorTargetState{
    ColorTargetState {
        format,
        blend: Some(wgpu::BlendState::REPLACE),
        write_mask: wgpu::ColorWrites::ALL,
    }
}

pub fn default_vertex_state(shader: &ShaderModule) -> VertexState{
    wgpu::VertexState {
        module: shader,
        entry_point: "vs_main",
        buffers: &[],
    }
}