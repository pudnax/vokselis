struct VertexOutput {
    @builtin(position) pos: vec4<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_idx: u32) -> VertexOutput {
    let vertex_idx = i32(vertex_idx);
    var res: vec4<f32>;
    if (vertex_idx == 0) {
        res = vec4<f32>(-0.25, -0.25, 0., 1.);
    } else if (vertex_idx == 1) {
        res = vec4<f32>(0.25, -0.25, 0., 1.);
    } else {
        res = vec4<f32>(0., 0.25, 0., 1.);
    }
    return VertexOutput(res);
}

@fragment
fn fs_main(vin: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(1., 0., 1., 1.);
}

