struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
}

struct LinearDiscrimination {
    vec_a: vec2<f32>,
    scl_b: f32,
}

@group(1) @binding(0)
var t: texture_2d<f32>;

@group(1) @binding(1)
var s: sampler;

@group(3) @binding(0)
var<uniform> discr: LinearDiscrimination;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let base: vec4<f32> = textureSample(t, s, in.uv) * in.color;

    let color: f32 = f32(dot(discr.vec_a, in.position.xy) - discr.scl_b < 0);

    return base * color;
}
