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

fn get_color(pos: vec2<f32>) -> f32 {
    return f32(dot(discr.vec_a, pos) - discr.scl_b < 0);
}

const MSAA: u32 = 2;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let base: vec4<f32> = textureSample(t, s, in.uv) * in.color;

    let pos = in.position.xy - vec2(0.5, 0.5);
    let dx = 1.0 / f32(MSAA);

    var sum = 0.0;

    let start = vec2(dx, dx);

    for (var i = u32(0); i < MSAA; i++) {
        for (var j = u32(0); j < MSAA; j++) {
            let p = pos + start + vec2(f32(i) * dx, f32(j) * dx);
            sum += get_color(p);
        }
    }

    return base * sum / f32(MSAA * MSAA);
}
