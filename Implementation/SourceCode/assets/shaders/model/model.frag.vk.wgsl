[[constant_id(0)]] 
let LIGHT_COUNT = 1u;
[[constant_id(1)]] 
let OUTPUT_MODE = 0u;
[[constant_id(2)]] 
let EMISSIVE_INTENSITY = 1.0;

let DIRECTIONAL_LIGHT_TYPE = 0u;
let NO_TEXTURE_ID = 255u;
let UNLIT_FLAG_UNLIT = 1u;

let DIELECTRIC_SPECULAR = vec3<f32>(0.04);
let BLACK = vec3<f32>(0.0);
let PI = 3.14159;

let ALPHA_MODE_MASK = 1u;
let ALPHA_MODE_BLEND = 2u;
let ALPHA_CUTOFF_BIAS = 0.0000001;

let METALLIC_ROUGHNESS_WORKFLOW = 0u;
let MAX_REFLECTION_LOD = 9.0; // last mip mips for 512 px res TODO: specializations ?


struct MaterialUniform {
    color: vec4<f32>;
    emissiveAndRoughnessGlossiness: vec4<f32>;
    metallicSpecularAndOcclusion: vec4<f32>;
    colorMetallicRoughnessEmissiveNormalTextureChannels: u32;
    occlusionTextureChannelAlphaModeUnlitFlagAndWorkflow: u32;
    alphaCutoff: f32;
};

struct PbrInfo {
     base_color: vec3<f32>;
     metallic: f32;
     specular: vec3<f32>;
     roughness: f32;
     metallicRoughnessWorkflow: bool;
};


struct Light {
    position: vec4<f32>;
    direction: vec4<f32>;
    color: vec4<f32>;
    intensity: f32;
    range: f32;
    angle_scale: f32;
    angle_offset: f32;
    light_type: u32;
};

struct TextureChannels {
    color: u32;
    material: u32;
    emissive: u32;
    normal: u32;
    occlusion: u32;
};

struct Camera {
    view: mat4x4<f32>;
    proj: mat4x4<f32>;
    inverted_proj: mat4x4<f32>;
    eye: vec4<f32>;
    z_near: f32;
    z_far: f32;
} ;

[[group(0), binding(0)]]
var<uniform> material:MaterialUniform;
[[group(0), binding(1)]]
var<uniform> camera:Camera;

[[group(1), binding(0)]]
var base_sampler : sampler;
[[group(1), binding(1)]]
var color_tex: texture_2d<f32>;
[[group(1), binding(2)]]
var normals_tex: texture_2d<f32>;
[[group(1), binding(3)]]
var material_tex: texture_2d<f32>;
[[group(1), binding(4)]]
var emissive_tex: texture_2d<f32>;
[[group(1), binding(5)]]
var brdfLookupSampler: texture_2d<f32>;
[[group(1), binding(6)]]
var irradianceMapSampler: texture_cube<f32>;
[[group(1), binding(7)]]
var preFilteredSampler: texture_cube<f32>;

[[group(2), binding(0)]]
var<storage> oColors: vec4<f32>;
[[group(2), binding(1)]]
var<storage> oTexcoords0: vec2<f32>;
[[group(2), binding(2)]]
var<storage> oTexcoords1: vec2<f32>;
[[group(2), binding(3)]]
var<storage> oNormals: vec3<f32>;
[[group(2), binding(4)]]
var<storage> oPositions: vec3<f32>;
[[group(2), binding(5)]]
var<storage> oTBN: vec3<f32>;

fn getTextureChannels() -> TextureChannels {
    return TextureChannels(
        (material.colorMetallicRoughnessEmissiveNormalTextureChannels >> 24u) & 255u,
        (material.colorMetallicRoughnessEmissiveNormalTextureChannels >> 16u) & 255u,
        (material.colorMetallicRoughnessEmissiveNormalTextureChannels >> 8u) & 255u,
        material.colorMetallicRoughnessEmissiveNormalTextureChannels & 255u,
        (material.occlusionTextureChannelAlphaModeUnlitFlagAndWorkflow >> 24u) & 255u
    );
}

fn getUV(tex_channel: u32) -> vec2<f32> {
    if (tex_channel == 0u) {
        return oTexcoords0;
    }
    return oTexcoords1;
}

fn getbase_color(textureChannels: TextureChannels) -> vec4<f32> {
    var color = material.color;
    if (textureChannels.color != NO_TEXTURE_ID) {
        let uv = getUV(textureChannels.color);
        let sampledColor = textureSample(color_tex, base_sampler, uv);
        color *= vec4<f32>(pow(sampledColor.rgb, vec3<f32>(2.2)), sampledColor.a);
    }
    return color * oColors;
}

fn getMetallic(textureChannels: TextureChannels) -> f32 {
    var metallic = material.metallicSpecularAndOcclusion.r;
    if (textureChannels.material != NO_TEXTURE_ID) {
        let uv = getUV(textureChannels.material);
        metallic *= textureSample(color_tex, base_sampler, uv).b;
    }

    return metallic;
}

fn getSpecular(textureChannels: TextureChannels) -> vec3<f32> {
    var specular = material.metallicSpecularAndOcclusion.rgb;
    if (textureChannels.material != NO_TEXTURE_ID) {
        let uv = getUV(textureChannels.material);
        let sampledColor = textureSample(material_tex, base_sampler, uv);
        specular *= pow(sampledColor.rgb, vec3<f32>(2.2));
    }
    return specular;
}

fn getEmissiveColor(textureChannels: TextureChannels) -> vec3<f32> {
    var emissive = material.emissiveAndRoughnessGlossiness.rgb;
    if (textureChannels.emissive != NO_TEXTURE_ID) {
        let uv = getUV(textureChannels.emissive);
        emissive *= pow(textureSample(emissive_tex, base_sampler, uv).rgb, vec3<f32>(2.2));
    }
    return emissive * EMISSIVE_INTENSITY;
}

fn getNormal(is_front_facing: bool, textureChannels: TextureChannels) -> vec3<f32> {
    var normal = normalize(oNormals);
    if (textureChannels.normal != NO_TEXTURE_ID) {
        let uv = getUV(textureChannels.normal);
        let normalMap = textureSample(normals_tex, base_sampler, uv).rgb * 2.0 - 1.0;
        normal = normalize(oTBN * normalMap);
    }

    if (!is_front_facing) {
        normal *= -1.0;
    }

    return normal;
}

fn getAlpha(base_color: vec4<f32>) -> f32 {
    if (getAlphaMode() == ALPHA_MODE_BLEND) {
        return base_color.a;
    }
    return 1.0;
}

fn getAlphaMode() -> u32 {
    return (material.occlusionTextureChannelAlphaModeUnlitFlagAndWorkflow >> 16u) & 255u;
}

fn isMasked(base_color: vec4<f32>) -> bool {
    return getAlphaMode() == ALPHA_MODE_MASK && base_color.a + ALPHA_CUTOFF_BIAS < material.alphaCutoff;
}

fn isUnlit() -> bool {
    let unlitFlag = (material.occlusionTextureChannelAlphaModeUnlitFlagAndWorkflow >> 8u) & 255u;
    if (unlitFlag == UNLIT_FLAG_UNLIT) {
        return true;
    }
    return false;
}

fn isMetallicRoughnessWorkflow() -> bool {
    let workflow = material.occlusionTextureChannelAlphaModeUnlitFlagAndWorkflow & 255u;
    if (workflow == METALLIC_ROUGHNESS_WORKFLOW) {
        return true;
    }
    return false;
}


fn convertMetallic(diffuse: vec3<f32>, specular: vec3<f32>, max_specular: f32) -> f32 {
    let c_MinRoughness = 0.04;
    let perceivedDiffuse = sqrt(0.299 * diffuse.r * diffuse.r + 0.587 * diffuse.g * diffuse.g + 0.114 * diffuse.b * diffuse.b);
    let perceivedSpecular = sqrt(0.299 * specular.r * specular.r + 0.587 * specular.g * specular.g + 0.114 * specular.b * specular.b);

    if (perceivedSpecular < c_MinRoughness) {
        return 0.0;
    }

    let a = c_MinRoughness;
    let b = perceivedDiffuse * (1.0 - max_specular) / (1.0 - c_MinRoughness) + perceivedSpecular - 2.0 * c_MinRoughness;
    let c = c_MinRoughness - perceivedSpecular;
    let D = max(b * b - 4.0 * a * c, 0.0);

    return clamp((-b + sqrt(D)) / (2.0 * a), 0.0, 1.0);
}

fn getRoughness(textureChannels: TextureChannels, metallicRoughnessWorkflow: bool) -> f32 {
    var roughness = material.emissiveAndRoughnessGlossiness.a;
    if (textureChannels.material != NO_TEXTURE_ID) {
        let uv = getUV(textureChannels.material);
        if (metallicRoughnessWorkflow) {
            roughness *= textureSample(material_tex, base_sampler, uv).g;
        } else {
            roughness *= textureSample(material_tex, base_sampler, uv).a;
        }
    }

    if (metallicRoughnessWorkflow) {
        return roughness;
    }

    return (1.0 - roughness);
}

fn f(f0: vec3<f32>, v: vec3<f32>, h: vec3<f32>) -> vec3<f32> {
    return f0 + (1.0 - f0) * pow(1.0 - max(dot(v, h), 0.0), 5.0);
}

fn f1(f0: vec3<f32>, v: vec3<f32>, n: vec3<f32>, roughness: f32) -> vec3<f32> {
    return f0 + (max(vec3<f32>(1.0 - roughness), f0) - f0) * pow(1.0 - max(dot(v, n), 0.0), 5.0);
}

fn vis(n: vec3<f32>, l: vec3<f32>, v: vec3<f32>, a: f32) -> f32 {
    let aa = a * a;
    let nl = max(dot(n, l), 0.0);
    let nv = max(dot(n, v), 0.0);
    let denom = ((nl * sqrt(nv * nv * (1.0 - aa) + aa)) + (nv * sqrt(nl * nl * (1.0 - aa) + aa))
    );

    if (denom < 0.0) {
        return 0.0;
    }
    return 0.5 / denom;
}

fn d(a: f32, n: vec3<f32>, h: vec3<f32>) -> f32 {
    let aa = a * a;
    let nh = max(dot(n, h), 0.0);
    let denom = nh * nh * (aa - 1.0) + 1.0;

    return aa / (PI * denom * denom);
}

fn computeAttenuation(distance: f32, range: f32) -> f32 {
    if (range < 0.0) {
        return 1.0;
    }
    return max(min(1.0 - pow(distance / range, 4.0), 1.0), 0.0) / pow(distance, 2.0);
}

fn computeColor(
    pbrInfo: PbrInfo,
    n: vec3<f32>,
    l: vec3<f32>,
    v: vec3<f32>,
    h: vec3<f32>,
    lightColor: vec3<f32>,
    lightIntensity: f32
) -> vec3<f32> {
    var color = vec3<f32>(0.0);
    if (dot(n, l) > 0.0 || dot(n, v) > 0.0) {
        var cDiffuse: vec3<f32>;
        var f0: vec3<f32>;
        if (pbrInfo.metallicRoughnessWorkflow) {
            cDiffuse = mix(pbrInfo.base_color * (1.0 - DIELECTRIC_SPECULAR.r), BLACK, pbrInfo.metallic);
            f0 = mix(DIELECTRIC_SPECULAR, pbrInfo.base_color, pbrInfo.metallic);
        } else {
            cDiffuse = pbrInfo.base_color * (1.0 - max(pbrInfo.specular.r, max(pbrInfo.specular.g, pbrInfo.specular.b)));
            f0 = pbrInfo.specular;
        }

        let a = pbrInfo.roughness * pbrInfo.roughness;

        let f_1 = f(f0, v, h);
        let vis_1 = vis(n, l, v, a);
        let d_1 = d(a, n, h);

        let diffuse = cDiffuse / PI;
        let fDiffuse = (1.0 - f_1) * diffuse;
        let fSpecular = max(f_1 * vis_1 * d_1, vec3<f32>(0.0));
        color = max(dot(n, l), 0.0) * (fDiffuse + fSpecular) * lightColor * lightIntensity;
    }
    return color;
}


fn computeDirectionalLight(light: Light, pbrInfo: PbrInfo, n: vec3<f32>, v: vec3<f32>) {
    let l = -normalize(light.direction.xyz);
    let h = normalize(l + v);
    return computeColor(pbrInfo, n, l, v, h, light.color.rgb, light.intensity);
}

fn computePointLight(light: Light, pbrInfo: PbrInfo, n: vec3<f32>, v: vec3<f32>) {
    let toLight = light.position.xyz - oPositions;
    let distance = length(toLight);
    let l = normalize(toLight);
    let h = normalize(l + v);

    let attenuation = computeAttenuation(distance, light.range);

    return computeColor(pbrInfo, n, l, v, h, light.color.rgb, light.intensity * attenuation);
}

fn computeSpotLight(light: Light, pbrInfo: PbrInfo, n: vec3<f32>, v: vec3<f32>) {
    let invLightDir = -normalize(light.direction.xyz);

    let toLight = light.position.xyz - oPositions;
    let distance = length(toLight);
    let l = normalize(toLight);
    let h = normalize(l + v);

    let attenuation = computeAttenuation(distance, light.range);

    let cd = dot(invLightDir, l);
    var angularAttenuation = max(0.0, cd * light.angle_scale + light.angle_offset);
    angularAttenuation *= angularAttenuation;

    return computeColor(pbrInfo, n, l, v, h, light.color.rgb, light.intensity * attenuation * angularAttenuation);
}

fn prefilteredReflection(R: vec3<f32>, roughness: f32) -> vec3<f32> {
    let lod = roughness * MAX_REFLECTION_LOD;
    return textureSampleBias(preFilteredSampler, base_sampler, R, lod).rgb;
}

fn computeIBL(pbrInfo: PbrInfo, v: vec3<f32>, n: vec3<f32>) -> vec3<f32> {

    var f0 = pbrInfo.specular;
    if (pbrInfo.metallicRoughnessWorkflow) {
        f0 = mix(DIELECTRIC_SPECULAR, pbrInfo.base_color, pbrInfo.metallic);
    }

    let f_1 = f1(f0, v, n, pbrInfo.roughness);
    var kD = 1.0 - f_1;
    kD *= 1.0 - pbrInfo.metallic;

    let irradiance = textureSample(irradianceMapSampler, base_sampler, n).rgb;
    let diffuse = irradiance * pbrInfo.base_color;

    let r = normalize(reflect(-v, n));
    let reflection = prefilteredReflection(r, pbrInfo.roughness);
    let envBRDF = textureSample(brdfLookupSampler, base_sampler, vec2(max(dot(n, v), 0.0), pbrInfo.roughness)).rg;
    let specular = reflection * (f * envBRDF.x + envBRDF.y);

    return kD * diffuse + specular;
}

struct FragmentInput {
    [[builtin(front_facing)]] is_front_facing: bool;
};

[[stage(fragment)]]
fn main(
    in: FragmentInput
) -> [[location(0)]] vec4<f32> {
    let textureChannels = getTextureChannels();

    let base_color = getbase_color(textureChannels);
    if (isMasked(base_color)) {
        discard;
    }
    let alpha = getAlpha(base_color);

    if (isUnlit()) {
        return vec4<f32>(base_color.rgb, alpha);
    }

    let metallic_roughness_workflow = isMetallicRoughnessWorkflow();
    let specular = getSpecular(textureChannels);
    let roughness = getRoughness(textureChannels, metallic_roughness_workflow);

    var metallic: f32;
    if (metallic_roughness_workflow) {
        metallic = getMetallic(textureChannels);
    } else {
        let max_specular = max(specular.r, max(specular.g, specular.b));
        metallic = convertMetallic(base_color.rgb, specular, max_specular);
    }

    let pbrInfo = PbrInfo(base_color.rgb, metallic, specular, roughness, metallic_roughness_workflow);

    let emissive = getEmissiveColor(textureChannels);

    let n = getNormal(in.is_front_facing, textureChannels);
    let v = normalize(camera.eye.xyz - oPositions);


    var color = vec3<f32>(0.0);

    for (var i = 0u; i < LIGHT_COUNT; i++) {
        let light = lights.lights[i];
        let lightType = light.light_type;

        if (lightType == DIRECTIONAL_LIGHT_TYPE) {
            color += computeDirectionalLight(light, pbrInfo, n, v);
        } else if (lightType == POINT_LIGHT_TYPE) {
            color += computePointLight(light, pbrInfo, n, v);
        } else if (lightType == SPOT_LIGHT_TYPE) {
            color += computeSpotLight(light, pbrInfo, n, v);
        }
    }

    let ambient = computeIBL(pbrInfo, v, n);

    color += emissive + occludeAmbientColor(ambient, textureChannels);

    return vec4(color, alpha);
}