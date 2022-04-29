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

struct Camera {
	view_pos: vec4<f32>,
	view_proj: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> un: Uniform;
@group(1) @binding(0)
var<uniform> cam: Camera;
@group(2) @binding(0)
var volume: texture_storage_3d<rgba8unorm, read>;
@group(3) @binding(0)
var out_tex: texture_storage_2d<rgba16float, write>;

var<private> tmin: f32 = 0.;
var<private> tmax: f32 = 0.;

let NUM_STEPS: i32 = 100;
let MIN_DIST: f32 = 0.0;
let MAX_DIST: f32 = 5.0;

fn intersect_box(orig: float3, dir: float3) -> float2 {
    let box_min = vec3<f32>(-1.0);
    let box_max = vec3<f32>(1.0);
    let inv_dir = 1.0 / dir;
    let tmin_tmp = (box_min - orig) * inv_dir;
    let tmax_tmp = (box_max - orig) * inv_dir;
    let tmin = min(tmin_tmp, tmax_tmp);
    let tmax = max(tmin_tmp, tmax_tmp);
    let t0 = max(tmin.x, max(tmin.y, tmin.z));
    let t1 = min(tmax.x, min(tmax.y, tmax.z));
    return float2(t0, t1);
}

fn get_cam(eye: float3, tar: float3) -> mat3x3<f32> {
    let zaxis = normalize(tar - eye);
    let xaxis = normalize(cross(zaxis, vec3<f32>(0., 1., 0.)));
    let yaxis = cross(xaxis, zaxis);
    return mat3x3<f32>(xaxis, yaxis, zaxis);
}

fn get_color(org: float3, dir: float3, tmin: f32, tmax: f32, clear_color: float3) -> float4 {
    var t_color = vec4<f32>(clear_color, 0.);
    var t_curr = tmax;

    var fixed_step = max((tmax - tmin) / f32(NUM_STEPS), 0.001);
    let block_size = vec3<f32>(textureDimensions(volume));

    var samp = vec3<i32>((block_size / 2.) * (org + t_curr * dir + 1.));
    var new_read = textureLoad(volume, samp);

    for (var i = 0; i < NUM_STEPS; i++) {
        // if (t_curr < tmin) { break; }

        let alpha_squared = pow(new_read.a, 1.);

        t_color = vec4<f32>(new_read.rgb + alpha_squared + t_color.rgb * t_color.a * (1. - alpha_squared), t_color.a);
        t_color.a = alpha_squared + t_color.a * (1. - alpha_squared);

        t_curr -= fixed_step;

        samp = vec3<i32>((block_size / 2.) * (org + t_curr * dir + 1.));

        new_read = textureLoad(volume, samp);
    }

    return t_color;
}

@compute @workgroup_size(16, 16, 1)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let dims = vec2<f32>(textureDimensions(out_tex));
    let aspect_ratio = dims.y / dims.x;

    let start_x = f32(global_id.x) / dims.x - 0.5;
    let start_y = -(f32(global_id.y) / dims.y - 0.5) * aspect_ratio;

    let zoom = 1.;
    var org = vec3<f32>(0., 0., 1.);
    org = 6. * vec3<f32>(cos(un.time), 0., sin(un.time)) + vec3<f32>(0., 2., 0.);
    let camera = get_cam(org, vec3<f32>(0.0, 0.0, 0.0));
    let dir = camera * vec3<f32>(start_x, start_y, zoom);

    let clear_color = vec4<f32>(0.1, 0.3, 0.3, 0.01);

    if (f32(global_id.xy.x) < dims.x && f32(global_id.y) < dims.y) {
        var t_hit = intersect_box(org, dir);
        t_hit = t_hit.yx;
        if (t_hit.x > t_hit.y) {
            t_hit.x = max(t_hit.x, 0.0);
            let col = get_color(org, dir, t_hit.x, t_hit.y, clear_color.rgb);
            textureStore(out_tex, global_id, col);
        } else {
            textureStore(out_tex, global_id, clear_color);
        }
    }
}
