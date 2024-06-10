
struct VertexInput {
    @location(0) position: vec3f,
    @location(1) normal: vec3f,
    @location(2) uv: vec2f,
    @location(3) tangent: vec4f,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4f,
    @location(0) position: vec3f,
    @location(1) uv: vec2f,    
    @location(2) view: vec3f,    
    @location(3) @interpolate(flat) normal: vec3f,    
    @location(4) @interpolate(flat) tangent: vec3f,    
    @location(5) @interpolate(flat) bitangent: vec3f,    
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
    // let tangent_space = (*input).tangent_space;
    let diffuse_color = (*material).diffuse_color;
    let specular_color = (*material).specular_color;
    let specular_exponent = (*material).specular_exponent;

    let light_direction = (camera.view * light.position).xyz - P * light.position.w;
    // let light_direction = tangent_space * light.position.xyz - P * light.position.w;
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
    // let tangent_space = (*input).tangent_space;

    let diffuse_color = (*material).diffuse_color;
    let specular_color = (*material).specular_color;
    let specular_exponent = (*material).specular_exponent;

    let light_direction = (camera.view * light.position).xyz - P * light.position.w;

    // let light_direction = tangent_space * light.position.xyz - P * light.position.w;
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
    // let tangent_space = (*input).tangent_space;

    let diffuse_color = (*material).diffuse_color;
    let specular_color = (*material).specular_color;
    let specular_exponent = (*material).specular_exponent;

    let light_direction = (camera.view * light.position).xyz - P * light.position.w;
    // let light_direction = tangent_space * light.position.xyz - P * light.position.w;
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
@group(0) @binding(1) var<storage> lights: array<Light>;
@group(1) @binding(0) var texture: texture_2d<f32>;
@group(1) @binding(1) var s_texture: sampler;
@group(1) @binding(2) var<uniform> diffuse_color: vec3f;
@group(1) @binding(3) var normal_map: texture_2d<f32>;
@group(1) @binding(4) var s_normal_map: sampler;
@group(1) @binding(2) var<uniform> specular_color: vec3f;
@group(1) @binding(2) var<uniform> specular_exponent: f32;


@vertex
fn vs_main(
    in: VertexInput,
) -> VertexOutput {
    var result: VertexOutput;
    result.clip_position = camera.proj * camera.view * vec4f(in.position, 1.0);
    result.position = (camera.view * vec4f(in.position, 1.0)).xyz;
    result.normal = (camera.inv_view * vec4f(in.normal, 0.0)).xyz;
    result.uv = in.uv;
    
    // let bitangent = cross(in.normal, in.tangent.xyz) * in.tangent.w;
    // let tangent_space = transpose(mat3x3<f32>(in.tangent.xyz, bitangent, in.normal));
    // result.position = vec3f(camera.view * in.position);
    // result.tangent = in.tangent.xyz;
    // result.bitangent = bitangent;
    // result.normal = in.normal;
    return result;
}

struct LightingInput {
    position: vec3f,
    normal: vec3f,
    view: vec3f,
    tangent_space: mat3x3<f32>,
}
struct ProcessedLightingInput {
    L: vec3f,
    NdotL: f32,
    NdotH: f32
}
struct Material {
    diffuse_color: vec3f,
    specular_color: vec3f,
    specular_exponent: f32,
}
    @fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let tangent_space = transpose(mat3x3<f32>(in.tangent, in.bitangent, in.normal));
    // Lighting is calculate_ in view space space
    var lighting_input: LightingInput;
    lighting_input.normal = normalize(in.normal);
    lighting_input.position = in.position;
    lighting_input.view = -in.position;
    let diffuse_color = textureSample(texture, s_texture, in.uv).rgb;
    var material: Material;
    material.diffuse_color = diffuse_color;
    material.specular_color = vec3f(1.0);
    material.specular_exponent = 100.0;
    let len = arrayLength(&lights);
    var lighting = vec3f(0.0);
    for (var i = 0u; i < len; i++) {
        let light = lights[i];
        if light.position.w == 0.0 {
            lighting += directional_light(&lighting_input, &material, light);
        } else if light.custom_data.w == -1.0 {
            lighting += point_light(&lighting_input, &material, light);
        } else {
            lighting += spot_light(&lighting_input, &material, light);
        }
    }
    let ambient = vec3f(0.1);
    return vec4<f32>(lighting + ambient * diffuse_color, 1.0);
}

