
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) tangent: vec3<f32>,
    @location(4) bitangent: vec3<f32>,
};


@group(0) @binding(0) var<uniform> camera: Camera;
@group(0) @binding(1) var<storage> lights: array<Light>;
@group(1) @binding(0) var texture: texture_2d<f32>;
@group(1) @binding(1) var s_texture: sampler;
@group(1) @binding(2) var<uniform> diffuse_color: vec3f;
@group(1) @binding(3) var normal_map: texture_2d<f32>;
@group(1) @binding(4) var s_normal_map: sampler;
@group(1) @binding(5) var<uniform> specular_color: vec3f;
@group(1) @binding(6) var<uniform> specular_exponent: f32;


@vertex
fn vs_main(
    in: VertexInput,
) -> VertexOutput {
    var result: VertexOutput;
    result.clip_position = camera.proj * camera.view * vec4f(in.position, 1.0);
    result.position = in.position;
    result.normal = in.normal;
    result.uv = in.uv;
    result.tangent = in.tangent;
    result.bitangent = in.bitangent;
    return result;
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) tangent: vec3<f32>,
    @location(4) bitangent: vec3<f32>,
};

struct ProcessedLightInput {
    L: vec3f,
    intensity: vec3f,
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let normal_map = textureSample(normal_map, s_normal_map, in.uv).rgb;
    let tangent_normal = normal_map * 2.0 - 1.0;
    let tangent_to_world = mat3x3<f32>(
        normalize(in.tangent),
        normalize(in.bitangent),
        normalize(in.normal),
    );
    let world_normal = tangent_to_world * tangent_normal;


    var lighting = vec3f(0.0);
    // Lighting is calculated in view space
    let P = normalize(world_normal.xyz);
    let N = in.position;
    let V = camera.position - in.position;

    let diffuse_texture = textureSample(texture, s_texture, in.uv).rgb;
    let diffuse_color = diffuse_texture * diffuse_color;
    let len = arrayLength(&lights);
    for (var i = 0u; i < len; i++) {
        let light = lights[i];
        let light_direction = (light.position).xyz - P * light.position.w;
        var processed: ProcessedLightInput;
        if light.position.w == 0.0 {
            processed = directional_light(light_direction, light);
        } else if light.custom_data.w == -1.0 {
            processed = point_light(light_direction, light);
        } else {
            processed = spot_light(light_direction, light);
        }
        let L = processed.L;
        let intensity = processed.intensity;
        let NdotL = max(dot(N, L), 0.0);
        let H = normalize(V + L);
        let NdotH = max(dot(N, H), 0.0);

        let diffuse = intensity * NdotL;
        let specular = intensity * pow(NdotH, specular_exponent) * select(0.0, 1.0, NdotL > 0.0);
        lighting += diffuse_color * diffuse + specular_color * specular;
    }
    let ambient = vec3f(0.01) * diffuse_color;

    return vec4<f32>(lighting + ambient, 1.0);
}

fn calculate_lighting() {
}

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



fn directional_light(light_direction: vec3f, light: Light) -> ProcessedLightInput {
    var processed: ProcessedLightInput;
    processed.L = normalize(light_direction);
    processed.intensity = light.color_range.rgb;
    return processed;
}
fn point_light(light_direction: vec3f, light: Light) -> ProcessedLightInput {
    var processed: ProcessedLightInput;
    processed.L = normalize(light_direction);
    let light_color = light.color_range.rgb;

    let range = light.color_range.w;
    let distance = length(light_direction);
    let attenuation_consts = light.custom_data.xyz;

    let attenuation = attenuation_consts[0] + attenuation_consts[1] * distance + attenuation_consts[2] * distance * distance;
    processed.intensity = light_color / distance * distance;

    if distance > range {
        processed.intensity = vec3f(0.0);
    }
    return  processed;
}
fn spot_light(light_direction: vec3f, light: Light) -> ProcessedLightInput {
    let L = normalize(light_direction);
    let light_color = light.color_range.rgb;

    let range = light.color_range.w;
    let distance = length(light_direction);
    let attenuation_consts = light.custom_data.xyz;
    let direction = vec4f(light.custom_data.xyz, 0.0);
    let outer_cutoff = light.custom_data.w;
    let spot_direction = normalize((-direction).xyz);
    let dot = dot(L, spot_direction);
    let delta = 1.0 - outer_cutoff;
    let intensity = light_color * saturate((dot - outer_cutoff) / delta);

    var processed: ProcessedLightInput;
    processed.L = L;
    processed.intensity = intensity;
    if distance > range || dot < outer_cutoff {
        processed.intensity = vec3f(0.0);
    }
    return processed;
}



