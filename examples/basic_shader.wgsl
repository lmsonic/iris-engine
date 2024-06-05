
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
};

struct Camera {
    projection: mat4x4<f32>,
    view: mat4x4<f32>,
    position: vec3f,
}

@group(0)
@binding(0)
var<uniform> camera: Camera;

@vertex
fn vs_main(
    @location(0) position: vec4<f32>,
) -> VertexOutput {
    var result: VertexOutput;
    result.position = camera.projection * camera.view * position;
    return result;
}


@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 1.0, 1.0, 1.0);
}

@fragment
fn fs_wire(vertex: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(0.0, 0.5, 0.0, 0.5);
}