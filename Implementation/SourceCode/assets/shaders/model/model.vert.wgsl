struct Camera {
	proj_view: mat4x4<f32>;
};

struct Model {
	matrix: mat4x4<f32>;
	inv_tr: mat4x4<f32>;
};

[[group(0), binding(0)]]
var<uniform> camera: Camera;

[[group(1), binding(0)]] 
var<uniform> model: Model;

struct VertexOutput {
	[[builtin(position)]] position: vec4<f32>;
	[[location(0)]] normal: vec3<f32>;
	[[location(1)]] world_pos: vec3<f32>;
};

[[stage(vertex)]]
fn main(
    [[location(0)]] pos: vec3<f32>,
    [[location(1)]] normal: vec3<f32>
) -> VertexOutput {
    // var v: VertexOutput;
    // v.position = camera.proj_view * model.matrix * vec4<f32>(pos, 1.0);
    // v.normal = normalize((model.inv_tr * vec4<f32>(normal, 0.0)).xyz);
    // v.world_pos = (model.matrix * vec4<f32>(pos, 1.0)).xyz;

    let v_position = camera.proj_view * model.matrix * vec4<f32>(pos, 1.0);
    let v_normal = normalize((model.inv_tr * vec4<f32>(normal, 0.0)).xyz);
    let v_world_pos = (model.matrix * vec4<f32>(pos, 1.0)).xyz;

    let v = VertexOutput(v_position, v_normal, v_world_pos);

    return v;
}