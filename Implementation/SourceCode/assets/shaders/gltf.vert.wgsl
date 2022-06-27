struct CameraUniform {
    view: mat4x4<f32>;
    projection: mat4x4<f32>;
    view_proj: mat4x4<f32>;
};

[[group(0), binding(0)]] 
var<uniform> camera: CameraUniform;

struct VertexInput {
    [[location(0)]] normal: vec3<f32>;
    [[location(1)]] position: vec3<f32>;
    [[location(2)]] tex_coords: vec2<f32>;
};

struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] tex_coords: vec2<f32>;
};

[[stage(vertex)]]
fn main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.clip_position = camera.view_proj * vec4<f32>(model.position, 1.0);
    return out;
}