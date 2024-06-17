
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) tangent: vec3<f32>,
    @location(4) bitangent: vec3<f32>,
};
struct InstanceInput {
    @location(5) x_axis: vec4<f32>,
    @location(6) y_axis: vec4<f32>,
    @location(7) z_axis: vec4<f32>,
    @location(8) w_axis: vec4<f32>,
};


@group(0) @binding(0) var texture: texture_2d<f32>;
@group(0) @binding(1) var s_texture: sampler;
@group(0) @binding(2) var<uniform> diffuse_color: vec3f;
@group(0) @binding(3) var normal_map: texture_2d<f32>;
@group(0) @binding(4) var s_normal_map: sampler;
@group(0) @binding(5) var<uniform> specular: f32;
@group(0) @binding(6) var<uniform> ior: f32;
@group(0) @binding(7) var<uniform> roughness: f32;
@group(0) @binding(8) var<uniform> ambient: vec3f;

@group(1) @binding(0) var<uniform> transform: mat4x4<f32>;

@group(2) @binding(0) var<uniform> camera: Camera;
@group(2) @binding(1) var<storage,read> lights: array<Light>;

@vertex
fn vs_main(
    v: VertexInput,
    i: InstanceInput,
) -> VertexOutput {
    var result: VertexOutput;
    let instance_transform = mat4x4<f32>(
        i.x_axis,
        i.y_axis,
        i.z_axis,
        i.w_axis,
    );
    result.clip_position = camera.proj * camera.view * instance_transform * transform * vec4f(v.position, 1.0);
    result.position = (transform * vec4f(v.position, 1.0)).xyz;
    result.normal = (transform * vec4f(v.normal, 1.0)).xyz;
    result.uv = v.uv;
    result.tangent = (transform * vec4f(v.tangent, 1.0)).xyz;
    result.bitangent = (transform * vec4f(v.bitangent, 1.0)).xyz;
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

struct Camera {
    proj: mat4x4<f32>,
    view: mat4x4<f32>,
    inv_view: mat4x4<f32>,
    position: vec3f,
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
    let N = normalize(world_normal.xyz);
    let P = in.position;
    let V = normalize(camera.position - in.position);

    let diffuse_texture = textureSample(texture, s_texture, in.uv).rgb;
    let diffuse = diffuse_texture * diffuse_color;
    let len = arrayLength(&lights);
    for (var i = 0u; i < len; i++) {
        let light = lights[i];
        let light_direction = (light.position).xyz - P * light.position.w;
        var intensity: vec3f;
        if light.position.w == 0.0 {
            intensity = directional_light(light_direction, light);
        } else if light.custom_data.w == -1.0 {
            intensity = point_light(light_direction, light);
        } else {
            intensity = spot_light(light_direction, light);
        }
        let L = normalize(light_direction);

        lighting += brdf(V, L, N, diffuse) * intensity;
    }

    return vec4<f32>(lighting + ambient * diffuse_color, 1.0);
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
const PI :f32= 3.14159265358979323846;

fn brdf(V: vec3f, L: vec3f, N: vec3f, diffuse: vec3f) -> vec3f {
    let H = normalize(L + V);
    let NdotH = saturate(dot(N, H));
    let LdotH = saturate(dot(L, H));
    let NdotV = saturate(dot(N, V));
    let NdotL = saturate(dot(N, L));

    let diffuse_bdrf = diffuse * diffuse_burley(NdotV, NdotL, LdotH, roughness);
    let f0 = calculate_f0(ior);

    let D = microfacet_distribution_ggx(NdotH);
    let F = fresnel(LdotH, vec3f(f0));
    let G = clamp(visibility_smith_ggx_correlated(NdotV, NdotL, roughness), 0.0, 1.0);

    let specular_bdrf = D * F * G;
    return (1.0 - specular) * diffuse_bdrf + specular * specular_bdrf;
}

fn calculate_f0(ior: f32) -> f32 {
    let num = (ior - 1.0);
    let den = (ior + 1.0);
    return num * num / (den * den);
}

fn diffuse_lambertian() -> f32 {
    return 1.0 / PI;
}

fn diffuse_burley(NdotV: f32, NdotL: f32, LdotH: f32, roughness: f32) -> f32 {
    let f90 = 0.5 + 2.0 * roughness * LdotH * LdotH;
    let light_scatter = fresnel_shlick(NdotL, 1.0, f90);
    let view_scatter = fresnel_shlick(NdotV, 1.0, f90);
    return light_scatter * view_scatter * (1.0 / PI);
}
fn fresnel(LdotH: f32, f0: vec3f) -> vec3f {
    let f90 = saturate(dot(f0, vec3<f32>(50.0 * 0.33)));
    return fresnel_shlick_vec(LdotH, f0, f90);
}

fn fresnel_shlick_vec(LdotH: f32, f0: vec3f, f90: f32) -> vec3f {
    return f0 + (f90 - f0) * pow(1.0 - LdotH, 5.0);
}
fn fresnel_shlick(LdotH: f32, f0: f32, f90: f32) -> f32 {
    return f0 + (f90 - f0) * pow(1.0 - LdotH, 5.0);
}


// T is direction aligned to where roughness is mx
fn anisotropic_microfacet_distribution(H: vec3f, N: vec3f, T: vec3f) -> f32 {
    let m = vec2f(10.0, 1.0); // Roughness vector   
    let NdotH = saturate(dot(N, H));
    let NdotH2 = NdotH * NdotH;
    let NdotH4 = NdotH2 * NdotH2;
    let first_factor = 1.0 / (4.0 * m.x * m.y * NdotH4);
    // P is H projected onto plane with normal N
    let P = normalize(H - NdotH * N);
    let TdotP = saturate(dot(T, P));
    let TdotP2 = TdotP * TdotP;
    let second_factor = TdotP2 / (m.x * m.x) + (1.0 - TdotP2) / (m.y * m.y);
    let third_factor = (NdotH2 - 1.0) / NdotH2;
    return first_factor * exp(second_factor * third_factor);
}


fn microfacet_distribution_ggx(NdotH: f32) -> f32 {
    let a = NdotH * roughness;
    let k = roughness / (1.0 - NdotH * NdotH + a * a);
    return k * k * (1.0 / PI);
}


fn visibility_smith_ggx_correlated(NdotV: f32, NdotL: f32, a: f32) -> f32 {
    let a2 = roughness * roughness;
    let GGXV = NdotL * sqrt((NdotV - a2 * NdotV) * NdotV + a2);
    let GGXL = NdotV * sqrt((NdotL - a2 * NdotL) * NdotL + a2);
    // It can divide by zero if NdotL and NdotV is 0
    return 0.5 / (GGXV + GGXL);
}


fn directional_light(light_direction: vec3f, light: Light) -> vec3f {
    let intensity = light.color_range.rgb;
    return intensity;
}
fn point_light(light_direction: vec3f, light: Light) -> vec3f {
    let L = normalize(light_direction);
    let light_color = light.color_range.rgb;

    let range = light.color_range.w;
    let distance = length(light_direction);
    let attenuation_consts = light.custom_data.xyz;

    let attenuation = attenuation_consts[0] + attenuation_consts[1] * distance + attenuation_consts[2] * distance * distance;
    var intensity = light_color / attenuation;

    if distance > range {
        intensity = vec3f(0.0);
    }
    return  intensity;
}
fn spot_light(light_direction: vec3f, light: Light) -> vec3f {
    let L = normalize(light_direction);
    let light_color = light.color_range.rgb;

    let range = light.color_range.w;
    let distance = length(light_direction);
    let attenuation_consts = light.custom_data.xyz;
    let direction = vec4f(light.custom_data.xyz, 0.0);
    let outer_cutoff = light.custom_data.w;
    let spot_direction = normalize((-direction).xyz);
    let dot = saturate(dot(L, spot_direction));
    let delta = 1.0 - outer_cutoff;
    var intensity = light_color * saturate((dot - outer_cutoff) / delta);

    if distance > range || dot < outer_cutoff {
        intensity = vec3f(0.0);
    }
    return intensity;
}



