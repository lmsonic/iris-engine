
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
    direction: vec4f,
    color: vec3f,
}


struct Camera {
    proj: mat4x4<f32>,
    view: mat4x4<f32>,
    inv_view: mat4x4<f32>,
    position: vec3f,
}

@group(0)
@binding(0)
var<uniform> camera: Camera;

@group(0) @binding(1) var<uniform> directional_light: Light;
@group(0) @binding(2) var<uniform> point_light: Light;
@group(0) @binding(3) var texture: texture_2d<f32>;
@group(0) @binding(4) var s_texture: sampler;
fn lighting(input: ptr<function, LightingInput>, material: ptr<function, Material>, light: Light) -> vec3f {
    // Lighting is calculate_ in clip space
    var diffuse = vec3f(0.0);
    var specular = vec3f(0.0);
    let P = (*input).position;
    let N = (*input).normal;
    let V = (*input).view;
    let ambient = (*input).ambient_light;
    let diffuse_color = (*material).diffuse_color;
    let specular_color = (*material).specular_color;
    let specular_exponent = (*material).specular_exponent;

    let light_direction = (camera.view * light.direction).xyz - P * light.direction.w;
    let L = normalize(light_direction);
    var intensity = vec3f(0.0);
    if light.direction.w == 0.0 {
        intensity = light.color;
    } else {
        // if distance < light.range {
        let distance = length(light_direction);
        // let attenuation = light.attenuation[0] + light.attenuation[1] * distance + light.attenuation[2] * distance * distance;
        intensity = light.color / distance * distance;
        // }
    }


    let NdotL = max(dot(N, L), 0.0);
    diffuse += intensity * NdotL;
    let H = normalize(V + L);
    let NdotH = max(dot(N, H), 0.0);

    specular += intensity * pow(NdotH, specular_exponent) * select(0.0, 1.0, NdotL > 0.0);

    return diffuse_color * ambient + diffuse_color * diffuse + specular_color * specular;
}



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
    position: vec3f, 
    normal: vec3f,
    view: vec3f, 
    ambient_light: vec3f,
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
    lighting_input.view = - in.position;
    lighting_input.ambient_light = vec3f(0.01);
    var material: Material;
    material.diffuse_color = textureSample(texture, s_texture, in.uv).rgb;
    material.specular_color = vec3f(1.0);
    material.specular_exponent = 100.0;
    let lighting = lighting(&lighting_input, &material, directional_light) + lighting(&lighting_input, &material, point_light);
    return vec4<f32>(lighting, 1.0);
}

@fragment
fn fs_wire(vertex: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(0.0, 0.5, 0.0, 0.5);
}