struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] tex_coords: vec2<f32>;
};

[[stage(fragment)]]
fn main(
    in: VertexOutput,
) -> [[location(0)]] vec4<f32> {
    // return textureLoad(texture_diffuse1, tex_coord);
    return vec4<f32>(in.tex_coords, 0.0, 0.0);
}