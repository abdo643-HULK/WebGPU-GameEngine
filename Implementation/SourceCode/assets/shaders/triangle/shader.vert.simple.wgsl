[[stage(vertex)]]
fn main(
    [[location(0)]] aPos: vec3<f32>,
) -> [[builtin(position)]] vec4<f32> {
    return vec4<f32>(aPos, 1.0);
}
