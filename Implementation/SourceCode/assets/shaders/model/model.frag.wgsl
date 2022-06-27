let PI = 3.141592653589793;

struct Camera {
    eye: vec3<f32>;
};

struct VertexOutput {
	[[builtin(position)]] position: vec4<f32>;
	[[location(0)]] normal: vec3<f32>;
	[[location(1)]] world_pos: vec3<f32>;
};

[[group(0), binding(1)]] 
var<uniform> camera: Camera;

fn linearSample(texture: texture_2d<f32>, tex_sampler: sampler, uv: vec2<f32>) -> vec4<f32> {
    let color = textureSample(texture, tex_sampler, uv);
    return vec4<f32>(pow(color.rgb, vec3<f32>(2.2)), color.a);
}

fn blinnPhong(color: vec3<f32>, l: vec3<f32>, v: vec3<f32>, n: vec3<f32>) -> vec3<f32> {
    let spec_exp = 64.0;
    let intensity = 0.5;
    let ambient = 0.5;

    let diffuse = max(dot(n, l), 0.0);
    let specular = pow(max(dot(n, normalize(l + v)), 0.0), spec_exp);

    return color * ((diffuse + specular) * intensity + ambient);
}

fn brdf(color: vec3<f32>, metallic: f32, roughness: f32, l: vec3<f32>, v: vec3<f32>, n: vec3<f32>) -> vec3<f32> {
    let h = normalize(l + v);
    let ndotl = clamp(dot(n, l), 0.0, 1.0);
    let ndotv = abs(dot(n, v));
    let ndoth = clamp(dot(n, h), 0.0, 1.0);
    let vdoth = clamp(dot(v, h), 0.0, 1.0);

    let f0 = vec3<f32>(0.04);
    let diffuse_color = color * (1.0 - f0) * (1.0 - metallic);
    let specular_color = mix(f0, color, metallic);

    let reflectance = max(max(specular_color.r, specular_color.g), specular_color.b);
    let reflectance0 = specular_color;
    let reflectance9 = vec3<f32>(clamp(reflectance * 25.0, 0.0, 1.0));
    let f = reflectance0 + (reflectance9 - reflectance0) * pow(1.0 - vdoth, 5.0);

    let r2 = roughness * roughness;
    let r4 = r2 * r2;
    let attenuation_L = 2.0 * ndotl / (ndotl + sqrt(r4 + (1.0 - r4) * ndotl * ndotl));
    let attenuation_V = 2.0 * ndotv / (ndotv + sqrt(r4 + (1.0 - r4) * ndotv * ndotv));
    let g = attenuation_L * attenuation_V;

    let temp = ndoth * ndoth * (r2 - 1.0) + 1.0;
    let d = r2 / (PI * temp * temp);

    let diffuse = (1.0 - f) / PI * diffuse_color;
    let specular = max(f * g * d / (4.0 * ndotl * ndotv), vec3<f32>(0.0));

    return ndotl * (diffuse + specular) * 2.0 + color * 0.1;
}

[[stage(fragment)]]
fn main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    let light_direction = normalize(vec3<f32>(2.0, 4.0, 3.0));

    let color = vec4<f32>(1.0, 1.0, 1.0, 1.0);
    let metallic: f32 = 1.0;
    let roughness = clamp(1.0, 0.04, 1.0);

    var normal = normalize(in.normal);

    let ao = 1.0;
    var emissive = vec3<f32>(0.0, 0.0, 0.0);


    let view_direction = normalize(camera.eye - in.world_pos);

    let res = brdf(color.rgb, metallic, roughness, light_direction, view_direction, normal) * ao + emissive;
    let rgb = pow(res, vec3<f32>(1.0 / 2.2));

    return vec4<f32>(rgb, color.a);
}