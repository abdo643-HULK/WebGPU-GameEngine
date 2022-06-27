struct VertexOutput {
    [[builtin(position)]] position: vec4<f32>;
    [[location(0)]] color: vec4<f32>;
};

struct Locals {
    color: vec4<f32>;
};

[[group(0), binding(0)]]
var<uniform> locals: Locals;

[[stage(fragment)]]
fn main(
    in: VertexOutput
) -> [[location(0)]] vec4<f32> {
    return locals.color;
}