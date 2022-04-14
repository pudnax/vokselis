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

fn linear_to_srgb(linear: vec4<f32>) -> vec4<f32> {
    let color_linear = linear.rgb;
    let selector = ceil(color_linear - 0.0031308);
    let under = 12.92 * color_linear;
    let over = 1.055 * pow(color_linear, vec3<f32>(0.41666)) - 0.055;
    let result = mix(under, over, selector);
    return vec4<f32>(result, linear.a);
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@stage(vertex)
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    let vertex_idx = i32(in_vertex_index);
    let uv = vec2<f32>(f32((vertex_idx << 1u) & 2), f32(vertex_idx & 2));
    let position = vec4<f32>(uv.x * 2.0 + -1.0, 1.0 - uv.y * 2.0, 0.0, 1.0);
    return VertexOutput(position, uv);
}

@group(1) @binding(0)
var src_texture: texture_2d<f32>;
@group(2) @binding(0)
var src_sampler: sampler;

struct FragmentOutput {
    @location(0) main: vec4<f32>,
    @location(1) secnd: vec4<f32>,
};

@stage(fragment)
fn fs_main(in: VertexOutput) -> FragmentOutput {
    let col = textureSample(src_texture, src_sampler, in.uv);
    let col = linear_to_srgb(col);
    return FragmentOutput(col, col);
}

@stage(fragment)
fn fs_main_raw(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(src_texture, src_sampler, in.uv);
}
