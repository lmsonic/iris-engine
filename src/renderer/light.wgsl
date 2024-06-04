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
struct SpotLight {
    position: vec3f,
    direction: vec3f,
    color: vec3f,
    range: f32,
    inner_cutoff: f32,
    outer_cutoff: f32,
    angle_attenuation: f32,
    attenuation: vec3f,
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


@group(0) @binding(0) var<storage,read> directional_lights: array<DirectionalLight>;
@group(0) @binding(1) var<storage,read> point_lights: array<PointLight>;
@group(0) @binding(2) var<storage,read> spot_lights: array<SpotLight>;


fn lighting(input: ptr<function, LightingInput>, material: ptr<function, Material>) -> vec3f {
    var diffuse = vec3f(0.0);
    var specular = vec3f(0.0);
    let d_length = arrayLength(&directional_lights);
    let position = (*input).position;
    let normal = (*input).normal;
    let view = (*input).view;
    let ambient = (*input).ambient_light;

    let diffuse_color = (*material).diffuse_color;
    let specular_color = (*material).specular_color;
    let specular_exponent = (*material).specular_exponent;

    for (var i: u32 = 0u; i < d_length; i++) {
        let light = directional_lights[i];
        let direction_to_light = -light.direction;
        let intensity = light.color;

        let ndotl = dot(normal, direction_to_light);
        diffuse += intensity * max(ndotl, 0.0);
        let half = normalize(direction_to_light + view);
        specular += intensity * pow(max(dot(normal, half), 0.0), specular_exponent) * select(1.0, 0.0, ndotl > 0.0);
    }
    let p_length = arrayLength(&spot_lights);

    for (var i: u32 = 0u; i < p_length; i++) {
        let light = point_lights[i];
        let delta = light.position - position;
        let distance = length(delta);
        if distance > light.range {
            continue;
        }
        let direction_to_light = normalize(delta);
        let attenuation = light.attenuation[0] + light.attenuation[1] * distance + light.attenuation[2] * distance * distance;
        let intensity = light.color / attenuation;

        let ndotl = dot(normal, direction_to_light);
        diffuse += intensity * max(ndotl, 0.0);
        let half = normalize(direction_to_light + view);
        specular += intensity * pow(max(dot(normal, half), 0.0), specular_exponent) * select(1.0, 0.0, ndotl > 0.0);
    }
    let s_length = arrayLength(&directional_lights);
    for (var i: u32 = 0u; i < s_length; i++) {
        let light = spot_lights[i];
        let delta = light.position - position;
        let distance = length(delta);
        if distance > light.range {
            continue;
        }
        let direction_to_light = normalize(delta);
        let dot = dot(direction_to_light, -light.direction);
        if dot < light.outer_cutoff {
            continue;
        }
        let intensity = light.color * smoothstep(light.outer_cutoff, light.inner_cutoff, dot);

        let ndotl = dot(normal, direction_to_light);
        diffuse += intensity * max(ndotl, 0.0);
        let half = normalize(direction_to_light + view);
        specular += intensity * pow(max(dot(normal, half), 0.0), specular_exponent) * select(1.0, 0.0, ndotl > 0.0);
    }

    return diffuse_color * ambient + diffuse_color * diffuse + specular_color * specular;
}