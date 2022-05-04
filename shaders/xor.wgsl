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
var xor_tex: texture_storage_3d<rgba16float, write>;
@group(1) @binding(1)
var normal_tex: texture_storage_3d<rgba16float, write>;

fn hash(h: f32) -> f32 {
    return fract(sin(h) * 43758.5453123);
}

fn noise(x: vec3<f32>) -> f32 {
    var p = floor(x);
    var f = fract(x);
    f = f * f * (3.0 - 2.0 * f);

    let n = p.x + p.y * 157.0 + 113.0 * p.z;
    return mix(
        mix(mix(hash(n + 0.0), hash(n + 1.0), f.x), mix(hash(n + 157.0), hash(n + 158.0), f.x), f.y),
        mix(mix(hash(n + 113.0), hash(n + 114.0), f.x), mix(hash(n + 270.0), hash(n + 271.0), f.x), f.y),
        f.z
    );
}

fn fbm(p: vec3<f32>) -> f32 {
    var p = p;
    var f = 0.0;
    f = 0.5000 * noise(p);
    p = p * 2.01;
    f += 0.2500 * noise(p);
    p = p * 2.02;
    f += 0.1250 * noise(p);
    return f;
}

fn volume(coord: vec3<f32>) -> vec4<f32> {
    let t = un.time;
    let pos = (coord + vec3(1., sin(t * 1.) * 0.1, 21.)) * 32.;
    let res = 25.;
    let val = f32(i32(pos.x * res) & i32(pos.y * res) & i32(pos.z * res)) / res;
    let alpha = val * smoothstep(0.7, 0.0, length(coord));
    return vec4(val, val, val, alpha);
}

fn noise_volume(coord: vec3<f32>) -> vec4<f32> {
    let t = un.time;
    let pos = (coord + vec3(1., sin(t * 1.) * 0.1, 21.)) * 32.;
    let val = fbm(pos);
    let alpha = val * smoothstep(0.5, 0.25, length(coord));
    return vec4(val, val, val, alpha);
}

fn gradient(pos: vec3<f32>, eps: f32) -> vec3<f32> {
    let eps = vec2(eps, 0.);
    let k = mat3x3<f32>(pos, pos, pos) - mat3x3<f32>(eps.xyy, eps.yxy, eps.yyx);
    return normalize(vec3(noise_volume(pos).a) - vec3(noise_volume(k[0]).a, noise_volume(k[1]).a, noise_volume(k[2]).a));
}

@compute @workgroup_size(8, 8, 8)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let dims = vec3<f32>(textureDimensions(xor_tex));
    var coord = (vec3<f32>(global_id) - dims / 2.) / dims;
    let vol = noise_volume(coord);
    let normal = gradient(coord, 0.0001);

    textureStore(xor_tex, global_id, vec4<f32>(vol.rgb / 2., vol.a));
    textureStore(normal_tex, global_id, vec4<f32>(normal, length(normal)));
}
