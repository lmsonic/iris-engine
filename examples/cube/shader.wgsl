
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(1) normal: vec2<f32>,
};



@group(0)
@binding(0)
var<uniform> view_proj: mat4x4<f32>;

@vertex
fn vs_main(
    @location(0) position: vec4<f32>,
    @location(1) normal: vec2<f32>,
) -> VertexOutput {
    var result: VertexOutput;
    result.normal = normal;
    result.position = view_proj * position;
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