struct Params { _pad: vec4<f32>, }
@group(2) @binding(0) var<uniform> params: Params;

@group(2) @binding(1) var world_normal_map: texture_2d<f32>;
@group(2) @binding(2) var world_normal_sampler: sampler;

struct VertexOut {
    @location(0) world_pos    : vec3<f32>,
    @location(1) world_normal : vec3<f32>,
    @location(2) uv           : vec2<f32>,
    @builtin(front_facing) is_front : bool,
}

@fragment
fn fragment(in: VertexOut) -> @location(0) vec4<f32> {
    var n = textureSample(world_normal_map, world_normal_sampler, in.uv).rgb;
    n = normalize(n * 2.0 - vec3<f32>(1.0));

    // if (!in.is_front) { n = -n; }

    let light_dir = normalize(vec3<f32>(0.4, 0.7, 0.3));
    let base_col  = vec3<f32>(0.54, 0.44, 0.33);
    let diff = max(dot(n, light_dir), 0.0);
    let lit = base_col * (0.15 + 0.85 * diff);
    return vec4<f32>(lit, 1.0);
}