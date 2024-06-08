
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};


struct Light {
    // In directional lights, w==0 and in point and spot lights w==1
    position: vec4f,
    // In point lights, w is the range
    color_range: vec4f,
    // In point lights, these are 3 attenuation constants
    // In spot lights, this is direction and cutoff
    custom_data: vec4f,
    
}


struct Camera {
    proj: mat4x4<f32>,
    view: mat4x4<f32>,
    inv_view: mat4x4<f32>,
    position: vec3f,
}



fn directional_light(input: ptr<function, LightingInput>, material: ptr<function, Material>, light: Light) -> vec3f {
    // Lighting is calculated in clip space
    var diffuse = vec3f(0.0);
    var specular = vec3f(0.0);
    let P = (*input).position;
    let N = (*input).normal;
    let V = (*input).view;
    let diffuse_color = (*material).diffuse_color;
    let specular_color = (*material).specular_color;
    let specular_exponent = (*material).specular_exponent;

    let light_direction = (camera.view * light.position).xyz - P * light.position.w;
    let light_color = light.color_range.rgb;
    let L = normalize(light_direction);

    let intensity = light_color;

    let NdotL = max(dot(N, L), 0.0);
    diffuse += intensity * NdotL;
    let H = normalize(V + L);
    let NdotH = max(dot(N, H), 0.0);
    specular += intensity * pow(NdotH, specular_exponent) * select(0.0, 1.0, NdotL > 0.0);

    return diffuse_color * diffuse + specular_color * specular;
}
fn point_light(input: ptr<function, LightingInput>, material: ptr<function, Material>, light: Light) -> vec3f {
    // Lighting is calculated in clip space
    var diffuse = vec3f(0.0);
    var specular = vec3f(0.0);
    let P = (*input).position;
    let N = (*input).normal;
    let V = (*input).view;
    let diffuse_color = (*material).diffuse_color;
    let specular_color = (*material).specular_color;
    let specular_exponent = (*material).specular_exponent;

    let light_direction = (camera.view * light.position).xyz - P * light.position.w;
    let light_color = light.color_range.rgb;
    let L = normalize(light_direction);

    let range = light.color_range.w;
    let distance = length(light_direction);
    let attenuation_consts = light.custom_data.xyz;
    if distance > range {
        return vec3f(0.0);
    }
    let attenuation = attenuation_consts[0] + attenuation_consts[1] * distance + attenuation_consts[2] * distance * distance;
    let intensity = light_color / distance * distance;

    let NdotL = max(dot(N, L), 0.0);
    diffuse += intensity * NdotL;
    let H = normalize(V + L);
    let NdotH = max(dot(N, H), 0.0);
    specular += intensity * pow(NdotH, specular_exponent) * select(0.0, 1.0, NdotL > 0.0);

    return diffuse_color * diffuse + specular_color * specular;
}
fn spot_light(input: ptr<function, LightingInput>, material: ptr<function, Material>, light: Light) -> vec3f {
    // Lighting is calculated in clip space
    var diffuse = vec3f(0.0);
    var specular = vec3f(0.0);
    let P = (*input).position;
    let N = (*input).normal;
    let V = (*input).view;
    let diffuse_color = (*material).diffuse_color;
    let specular_color = (*material).specular_color;
    let specular_exponent = (*material).specular_exponent;

    let light_direction = (camera.view * light.position).xyz - P * light.position.w;
    let light_color = light.color_range.rgb;
    let L = normalize(light_direction);

    let range = light.color_range.w;
    let distance = length(light_direction);
    let attenuation_consts = light.custom_data.xyz;
    if distance > range {
        return vec3f(0.0);
    }
    let direction = vec4f(light.custom_data.xyz, 0.0);
    let outer_cutoff = light.custom_data.w;
    let spot_direction = normalize((camera.view * -direction).xyz);
    let dot = dot(L, spot_direction);
    // return vec3f(dot);
    if dot < outer_cutoff {
        return vec3f(0.0);
    }
    let delta = 1.0 - outer_cutoff;
    let intensity = light_color * saturate((dot - outer_cutoff) / delta);

    let NdotL = max(dot(N, L), 0.0);
    diffuse += intensity * NdotL;
    let H = normalize(V + L);
    let NdotH = max(dot(N, H), 0.0);
    specular += intensity * pow(NdotH, specular_exponent) * select(0.0, 1.0, NdotL > 0.0);

    return diffuse_color * diffuse + specular_color * specular;
}
@group(0) @binding(0) var<uniform> camera: Camera;
@group(0) @binding(1) var<uniform> directional_light1: Light;
@group(0) @binding(2) var<uniform> point_light1: Light;
@group(0) @binding(3) var<uniform> spot_light1: Light;
@group(0) @binding(4) var texture: texture_2d<f32>;
@group(0) @binding(5) var s_texture: sampler;

@vertex
fn vs_main(
    in: VertexInput,
) -> VertexOutput {
    var result: VertexOutput;
    result.clip_position = camera.proj * camera.view * vec4f(in.position, 1.0);
    result.position = (camera.view * vec4f(in.position, 1.0)).xyz;
    result.normal = (camera.inv_view * vec4f(in.normal, 1.0)).xyz;
    result.uv = in.uv;
    return result;
}

struct LightingInput {
    // TODO: process light input into the dot products needed before passing it to the functions
    position: vec3f,
    normal: vec3f,
    view: vec3f,
}
struct Material {
    diffuse_color: vec3f,
    specular_color: vec3f,
    specular_exponent: f32,
}
    @fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Lighting is calculate_ in clip space
    var lighting_input: LightingInput;
    lighting_input.normal = normalize(in.normal);
    lighting_input.position = in.position;
    lighting_input.view = -in.position;
    let ambient = vec3f(0.1);
    let diffuse_color = textureSample(texture, s_texture, in.uv).rgb;
    var material: Material;
    material.diffuse_color = diffuse_color;
    material.specular_color = vec3f(1.0);
    material.specular_exponent = 100.0;
    let lighting = directional_light(&lighting_input, &material, directional_light1) + point_light(&lighting_input, &material, point_light1) + spot_light(&lighting_input, &material, spot_light1) + ambient * diffuse_color;
    return vec4<f32>(lighting, 1.0);
}

    @fragment
fn fs_wire(vertex: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(0.0, 0.5, 0.0, 0.5);
}