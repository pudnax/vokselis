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
var xor_tex: texture_storage_3d<rgba8unorm, write>;

@compute @workgroup_size(8, 8, 8)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    var pos = (vec3<f32>(global_id)) * .1 + un.time;
    let res = 6.;
    let val = i32(pos.x % res) ^ i32(pos.y % res) ^ i32(pos.z % res);
    let val = f32(val);
    textureStore(xor_tex, global_id, vec4<f32>(val, val, val, val));
}
