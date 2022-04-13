struct VertexOutput {
    @builtin(position) pos: vec4<f32>,
};

@stage(vertex)
fn main(@builtin(vertex_index) vertex_idx: u32) -> VertexOutput {
    let pos = vec2<f32>(f32((vertex_idx << 1u) & 2u), f32(vertex_idx & 2u));
    return vec4<f32>(pos * 2.0 - 1.0, 0.0, 1.0);
}

@stage(fragment)
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(1., 1., 1., 1.);
}
