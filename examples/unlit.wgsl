

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(1) uv: vec2<f32>,
};


struct Camera {
    proj: mat4x4<f32>,
    view: mat4x4<f32>,
    inv_view: mat4x4<f32>,
    position: vec3f,
}
@group(0) @binding(0) var<uniform> camera: Camera;
@group(0) @binding(1) var texture: texture_2d<f32>;
@group(0) @binding(2) var s_texture: sampler;
@vertex
fn vs_main(
    in: VertexInput,
) -> VertexOutput {
    var result: VertexOutput;
    result.clip_position = camera.proj * camera.view * vec4(in.position, 1.0);
    result.uv = in.uv;
    return result;
}


@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
    let diffuse_color = textureSample(texture, s_texture, in.uv).rgb;
    return vec4f(diffuse_color, 1.0);
}

@fragment
fn fs_wire(vertex: VertexOutput) -> @location(0) vec4f {

    return vec4f(0.0, 0.5, 0.0, 0.5);
}