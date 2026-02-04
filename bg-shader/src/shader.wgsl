struct Uniforms {
    resolution: vec2<f32>,
    time: f32,
    _padding: f32,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    // Full-screen triangle
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0),
    );

    var out: VertexOutput;
    let pos = positions[vertex_index];
    out.position = vec4<f32>(pos, 0.0, 1.0);
    out.uv = pos * 0.5 + 0.5;
    return out;
}

// Simple hash-based noise
fn hash(p: vec2<f32>) -> f32 {
    var p3 = fract(vec3<f32>(p.x, p.y, p.x) * 0.13);
    p3 = p3 + dot(p3, vec3<f32>(p3.y + 3.333, p3.z + 3.333, p3.x + 3.333));
    return fract((p3.x + p3.y) * p3.z);
}

// Value noise
fn noise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);

    let a = hash(i);
    let b = hash(i + vec2<f32>(1.0, 0.0));
    let c = hash(i + vec2<f32>(0.0, 1.0));
    let d = hash(i + vec2<f32>(1.0, 1.0));

    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}

// Fractal Brownian Motion
fn fbm(p: vec2<f32>) -> f32 {
    var value = 0.0;
    var amplitude = 0.5;
    var frequency = 1.0;
    var pos = p;

    for (var i = 0; i < 5; i++) {
        value += amplitude * noise(pos * frequency);
        frequency *= 2.0;
        amplitude *= 0.5;
        pos += vec2<f32>(1.7, 9.2);
    }
    return value;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let aspect = uniforms.resolution.x / uniforms.resolution.y;
    var uv = in.uv;
    uv.x *= aspect;

    let t = uniforms.time * 0.08;

    // Layered noise for organic flow
    let n1 = fbm(uv * 3.0 + vec2<f32>(t * 0.3, t * 0.2));
    let n2 = fbm(uv * 5.0 - vec2<f32>(t * 0.2, t * 0.15) + vec2<f32>(n1 * 0.5));
    let n3 = fbm(uv * 2.0 + vec2<f32>(n2 * 0.3, n1 * 0.3) + vec2<f32>(t * 0.1));

    // Combine into subtle pattern
    let pattern = n1 * 0.4 + n2 * 0.35 + n3 * 0.25;

    // Map to very dark neutral tones (matching bg-neutral-950)
    // neutral-950 is approximately rgb(10, 10, 10) / #0a0a0a
    let base = 0.039; // 10/255
    let variation = pattern * 0.04; // Very subtle

    let color = vec3<f32>(base + variation);

    return vec4<f32>(color, 1.0);
}
