type float2 = vec2<f32>;
type float3 = vec3<f32>;
type float4 = vec4<f32>;

struct Uniform {
    pos: vec3<f32>,
    frame: u32,
    resolution: vec2<f32>,
    mouse: vec2<f32>,
    mouse_pressed: u32,
    time: f32,
    time_delta: f32,
};

@group(0) @binding(0)
var<uniform> un: Uniform;

@group(1) @binding(0)
var src_texture: texture_2d<f32>;
@group(2) @binding(0)
var src_sampler: sampler;

fn linear_to_srgb(col: vec4<f32>) -> vec4<f32> {
    let color_linear = col.rgb;
    let selector = ceil(color_linear - 0.0031308);
    let under = 12.92 * color_linear;
    let over = 1.055 * pow(color_linear, vec3<f32>(0.41666)) - 0.055;
    let result = mix(under, over, selector);
    return vec4<f32>(result, col.a);
}

//https://knarkowicz.wordpress.com/2016/01/06/aces-filmic-tone-mapping-curve/
fn ACESFilm(x: vec3<f32>) -> vec3<f32> {
    return clamp((x * (2.51 * x + 0.03)) / (x * (2.43 * x + 0.59) + 0.14), vec3(0.0), vec3(1.0));
}

fn tex_sample(tex: texture_2d<f32>, uv: vec2<f32>) -> float4 {
    return textureSample(tex, src_sampler, uv);
}

fn texture_quadratic(tex: texture_2d<f32>, uv: vec2<f32>) -> vec4<f32> {
    let tex_size = f32(textureDimensions(tex).x);
    var p = uv * tex_size;
    let i = floor(p);
    var f = fract(p);
    p = i + f * 0.5;
    p = p / tex_size;
    f = f * f * (3.0 - 2.0 * f); // optional for extra sweet
    let w = 0.5 / tex_size;
    var res = mix(
        mix(tex_sample(tex, p + float2(0., 0.)), tex_sample(tex, p + float2(w, 0.0)), f.x),
        mix(tex_sample(tex, p + float2(0., w)), tex_sample(tex, p + float2(w, w)), f.x),
        f.y
    );
    return res;
}

// w0, w1, w2, and w3 are the four cubic B-spline basis functions
fn w0(a: f32) -> f32 { return (1.0 / 6.0) * (a * (a * (-a + 3.0) - 3.0) + 1.0); }
fn w1(a: f32) -> f32 { return (1.0 / 6.0) * (a * a * (3.0 * a - 6.0) + 4.0); }
fn w2(a: f32) -> f32 { return (1.0 / 6.0) * (a * (a * (-3.0 * a + 3.0) + 3.0) + 1.0); }
fn w3(a: f32) -> f32 { return (1.0 / 6.0) * (a * a * a); }
// g0 and g1 are the two amplitude functions
fn g0(a: f32) -> f32 { return w0(a) + w1(a); }
fn g1(a: f32) -> f32 { return w2(a) + w3(a); }
// h0 and h1 are the two offset functions
fn h0(a: f32) -> f32 { return -1.0 + w1(a) / (w0(a) + w1(a)); }
fn h1(a: f32) -> f32 { return 1.0 + w3(a) / (w2(a) + w3(a)); }

fn texture_bicubic(tex: texture_2d<f32>, p: vec2<f32>) -> vec4<f32> {
    let tex_size = vec2<f32>(textureDimensions(tex));
    let texel_size = vec4<f32>(1. / tex_size, tex_size);
    let uv = p * texel_size.zw + 0.5;
    let iuv = floor(uv);
    let fuv = fract(uv);

    let g0x = g0(fuv.x);
    let g1x = g1(fuv.x);
    let h0x = h0(fuv.x);
    let h1x = h1(fuv.x);
    let h0y = h0(fuv.y);
    let h1y = h1(fuv.y);

    let p0 = (vec2<f32>(iuv.x + h0x, iuv.y + h0y) - 0.5) * texel_size.xy;
    let p1 = (vec2<f32>(iuv.x + h1x, iuv.y + h0y) - 0.5) * texel_size.xy;
    let p2 = (vec2<f32>(iuv.x + h0x, iuv.y + h1y) - 0.5) * texel_size.xy;
    let p3 = (vec2<f32>(iuv.x + h1x, iuv.y + h1y) - 0.5) * texel_size.xy;

    return g0(fuv.y) * (g0x * tex_sample(tex, p0) + g1x * tex_sample(tex, p1)) + g1(fuv.y) * (g0x * tex_sample(tex, p2) + g1x * tex_sample(tex, p3));
}


struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    let vertex_idx = i32(in_vertex_index);
    let uv = vec2<f32>(f32((vertex_idx << 1u) & 2), f32(vertex_idx & 2));
    let position = vec4<f32>(uv.x * 2.0 + -1.0, 1.0 - uv.y * 2.0, 0.0, 1.0);
    return VertexOutput(position, uv);
}

struct FragmentOutput {
    @location(0) main: vec4<f32>,
    @location(1) secnd: vec4<f32>,
};

@fragment
fn fs_main(vin: VertexOutput) -> FragmentOutput {
    let col = tex_sample(src_texture, vin.uv);
    // let col = texture_quadratic(src_texture, vin.uv);
    // let col = texture_bicubic(src_texture, vin.uv);
    let col = vec4(ACESFilm(col.rgb), col.a);
    let col = linear_to_srgb(col);
    return FragmentOutput(col, col);
}

@fragment
fn fs_main_raw(vin: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(src_texture, src_sampler, vin.uv);
}
