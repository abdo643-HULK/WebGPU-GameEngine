struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>
};

@vertex
fn main(
    @location(0) aPos: vec3<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    out.color = vec4<f32>(0.5, 0.0, 0.0, 1.0);
    out.position = vec4<f32>(aPos, 1.0);

    return out;
}
