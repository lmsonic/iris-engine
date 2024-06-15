

struct VertexInput {
    @location(0) position: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
};


struct Camera {
    proj: mat4x4<f32>,
    view: mat4x4<f32>,
    position: vec3f,
}
@group(1) @binding(0) var<uniform> transform: mat4x4<f32>;

@group(2) @binding(0) var<uniform> camera: Camera;

@vertex
fn vs_main(
    in: VertexInput,
) -> VertexOutput {
    var result: VertexOutput;
    result.clip_position = camera.proj * camera.view * transform * vec4f(in.position, 1.0);
    return result;
}



@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4f {
    return vec4f(0.0, 0.5, 0.0, 0.5);
}