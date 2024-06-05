
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) camera_position: vec3<f32>,
    @location(1) camera_normal: vec3<f32>,
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
// @group(0) @binding(2) var<uniform> point_light: Light;
fn lighting(input: ptr<function, LightingInput>, material: ptr<function, Material>) -> vec3f {
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

    let light = directional_light;
    let L = -normalize((camera.view * light.direction).xyz - P * light.direction.w);
    let intensity = light.color;

    let NdotL = dot(N, L);
    diffuse += intensity * max(NdotL, 0.0);
    let H = normalize(V + L);
    // let R = reflect(L, N);
    let NdotH = max(dot(N, H), 0.0);
    // let RdotV = max(dot(R, V), 0.0);
    specular += intensity * pow(NdotH, specular_exponent) * select(1.0, 0.0, NdotL < 0.0);


    // let light = point_light;
    // let delta = light.position - position;
    // let distance = length(delta);
    // if distance < light.range {
    //     let direction_to_light = normalize(delta);
    //     let attenuation = light.attenuation[0] + light.attenuation[1] * distance + light.attenuation[2] * distance * distance;
    //     let intensity = light.color / attenuation;
    // } else {
    //     let intensity = vec3f(0.0);
    // }


    // let ndotl = dot(normal, direction_to_light);
    // diffuse += intensity * saturate(ndotl);
    // let half = normalize(direction_to_light + view);
    // specular += intensity * pow(saturate(dot(normal, half)), specular_exponent) * select(1.0, 0.0, ndotl > 0.0);

    return diffuse_color * ambient + diffuse_color * diffuse + specular_color * specular;
}



@vertex
fn vs_main(
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
) -> VertexOutput {
    var result: VertexOutput;
    result.clip_position = camera.proj * camera.view * vec4f(position, 1.0);
    result.camera_position = (camera.view * vec4f(position, 1.0)).xyz;
    result.camera_normal = (camera.inv_view * vec4f(normal, 0.0)).xyz;
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
    lighting_input.normal = normalize(in.camera_normal);
    lighting_input.position = in.camera_position.xyz;
    lighting_input.view = -in.camera_position.xyz;
    lighting_input.ambient_light = vec3f(0.01);
    var material: Material;
    material.diffuse_color = vec3f(1.0);
    material.specular_color = vec3f(1.0, 0.0, 0.0);
    material.specular_exponent = 100.0;
    let lighting = lighting(&lighting_input, &material);
    return vec4<f32>(lighting, 1.0);
}

@fragment
fn fs_wire(vertex: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(0.0, 0.5, 0.0, 0.5);
}