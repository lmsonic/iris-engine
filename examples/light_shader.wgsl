
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(1) normal: vec3<f32>,
};
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
struct DirectionalLight {
    direction: vec3f,
    color: vec3f,
}
struct PointLight {
    position: vec3f,
    color: vec3f,
    range: f32,
    attenuation: vec3f,
}
struct Camera {
    view_proj: mat4x4<f32>,
    position: vec3f,
}
@group(0)
@binding(0)
var<uniform> camera: Camera;

const MAX_DIRECTIONAL_LIGHTS = 1u;
@group(0) @binding(1) var<uniform> directional_lights: array<DirectionalLight,MAX_DIRECTIONAL_LIGHTS>;
@group(0) @binding(2) var<uniform> point_lights: array<PointLight,MAX_DIRECTIONAL_LIGHTS>;
fn lighting(input: ptr<function, LightingInput>, material: ptr<function, Material>) -> vec3f {
    var diffuse = vec3f(0.0);
    var specular = vec3f(0.0);
    let d_length = MAX_DIRECTIONAL_LIGHTS;
    let position = (*input).position;
    let normal = (*input).normal;
    let view = (*input).view;
    let ambient = (*input).ambient_light;

    let diffuse_color = (*material).diffuse_color;
    let specular_color = (*material).specular_color;
    let specular_exponent = (*material).specular_exponent;

    for (var i: u32 = 0u; i < 0u; i++) {
        let light = directional_lights[i];
        let direction_to_light = -light.direction;
        let intensity = light.color;

        let ndotl = dot(normal, direction_to_light);
        diffuse += intensity * saturate(ndotl);
        let half = normalize(direction_to_light + view);
        specular += intensity * pow(saturate(dot(normal, half)), specular_exponent) * select(1.0, 0.0, ndotl > 0.0);
    }
    let p_length = MAX_DIRECTIONAL_LIGHTS;

    for (var i: u32 = 0u; i < p_length; i++) {
        let light = point_lights[i];
        let delta = light.position - position;
        let distance = length(delta);
        // if distance > light.range {
        //     continue;
        // }
        let direction_to_light = normalize(delta);
        let attenuation = light.attenuation[0] + light.attenuation[1] * distance + light.attenuation[2] * distance * distance;
        let intensity = light.color / attenuation;

        let ndotl = dot(normal, direction_to_light);
        diffuse += intensity * saturate(ndotl);
        let half = normalize(direction_to_light + view);
        specular += intensity * pow(saturate(dot(normal, half)), specular_exponent) * select(1.0, 0.0, ndotl > 0.0);
    }
    return diffuse_color * ambient + diffuse_color * diffuse + specular_color * specular;
}



@vertex
fn vs_main(
    @location(0) position: vec4<f32>,
    @location(1) normal: vec3<f32>,
) -> VertexOutput {
    var result: VertexOutput;
    result.normal = (camera.view_proj * vec4f(normal, 0.0)).xyz;
    result.position = camera.view_proj * position;
    return result;
}


@fragment
fn fs_main(out: VertexOutput) -> @location(0) vec4<f32> {
    var lighting_input: LightingInput;
    lighting_input.normal = normalize(out.normal);
    lighting_input.position = out.position.xyz;
    lighting_input.view = camera.position;
    lighting_input.ambient_light = vec3f(0.0);
    var material: Material;
    material.diffuse_color = vec3f(1.0);
    material.specular_color = vec3f(1.0);
    material.specular_exponent = 10.0;
    let lighting = lighting(&lighting_input, &material);
    return vec4<f32>(lighting, 1.0);
}

@fragment
fn fs_wire(vertex: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(0.0, 0.5, 0.0, 0.5);
}