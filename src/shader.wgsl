struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) screen_position: vec2<f32>,
};

struct MapCylinderUniform {
    rotation_matrix: mat3x3<f32>,
    aspect_ratio: f32,
    // padding: vec3<f32>,
}

const PI = 3.14159265;
const HALF_PI = 1.570797325;
const TWO_PI = 6.283185307;
/// ln(tan(π÷4+83÷180×π÷2)) = 2.79
const MERCATOR_SCALE = 2.794219058;

@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
) -> VertexOutput {
    var out: VertexOutput;
    let x = f32(1 - i32(in_vertex_index)) * 5.;
    let y = f32(i32(in_vertex_index & 1u) * 2 - 1) * 5.;
    out.clip_position = vec4<f32>(x, y, 0.0, 1.0);
    out.screen_position = vec2<f32>(x, y);
    return out;
}


@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;
@group(1) @binding(0)
var<uniform> map_cylinder: MapCylinderUniform;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let gcs = mercator_to_gcs(in.screen_position, map_cylinder.aspect_ratio);
    let n_vector = gcs_to_n_vector(gcs);
    let rotated_n_vector = map_cylinder.rotation_matrix * n_vector;
    let rotated_gcs = n_vector_to_gcs(rotated_n_vector);
    let texture_position = gcs_to_texture_position(rotated_gcs);
    return textureSample(t_diffuse, s_diffuse, texture_position);
}

/// Maps a normalized mercator coordinate [-1,1] to latitude and longitude
fn mercator_to_gcs(mercator: vec2<f32>, aspect_ratio: f32) -> vec2<f32>{
    let lon = (mercator.x*PI + PI) % TWO_PI - PI;
    let lat = (2.0 * atan(exp(-mercator.y*MERCATOR_SCALE))) % PI - HALF_PI;
    return vec2<f32>(lon*aspect_ratio, lat);
}

fn gcs_to_n_vector(gcs: vec2<f32>) -> vec3<f32> {
    let cos_lat = cos(gcs.y);
    return vec3<f32>(cos_lat * cos(gcs.x), cos_lat * sin(gcs.x), sin(gcs.y));
}

fn n_vector_to_gcs(n_vector: vec3<f32>) -> vec2<f32> {
    let lon = atan2(n_vector.y, n_vector.x);
    let lat = asin(n_vector.z);
    return vec2<f32>(lon, lat);
}

/// Maps a geographic coordinate system (GCS) coordinate to a texture coordinate of a equirectangular world map
fn gcs_to_texture_position(gcs: vec2<f32>) -> vec2<f32> {
    return vec2<f32>(gcs.x / TWO_PI + 0.5, gcs.y / PI + 0.5);
}