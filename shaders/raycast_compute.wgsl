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

fn intersect_box(orig: vec3<f32>, dir: vec3<f32>) -> vec2<f32> {
    let box_min = vec3(-1.0);
    let box_max = vec3(1.0);
    let inv_dir = 1.0 / dir;
    let tmin_tmp = (box_min - orig) * inv_dir;
    let tmax_tmp = (box_max - orig) * inv_dir;
    let tmin = min(tmin_tmp, tmax_tmp);
    let tmax = max(tmin_tmp, tmax_tmp);
    let t0 = max(tmin.x, max(tmin.y, tmin.z));
    let t1 = min(tmax.x, min(tmax.y, tmax.z));
    return vec2<f32>(t0, t1);
}

fn get_cam(eye: vec3<f32>, tar: vec3<f32>) -> mat3x3<f32> {
    let zaxis = normalize(tar - eye);
    let xaxis = normalize(cross(zaxis, vec3(0., 1., 0.)));
    let yaxis = cross(xaxis, zaxis);
    return mat3x3(xaxis, yaxis, zaxis);
}

fn get_col2(eye: vec3<f32>, dir: vec3<f32>, tmin: f32, tmax: f32, clear_color: vec4<f32>) -> vec4<f32> {
    var color = vec4(0.0);
    let block_size = vec3<f32>(textureDimensions(volume));
    let dt_vec = 1.0 / (block_size * abs(dir));
    let dt_scale = 1.0;
    let dt = max(dt_scale * min(dt_vec.x, min(dt_vec.y, dt_vec.z)), 0.01);
    for (var t = tmin; t < tmax; t = t + dt) {
        var p = eye + t * dir;
        let vol_content = textureLoad(volume, vec3<i32>((p + 1.) * (block_size / 2.)));

        var vol_color = vol_content.rgb;
        let vol_alpha = pow(vol_content.a, 3.0);

        let tmp = color.rgb + (1.0 - color.a) * vol_alpha * vol_color + clear_color.rgb * clear_color.a * (1.0 - vol_alpha);
        color = vec4(tmp, color.a);
        color.a = color.a + (1.0 - color.a) * vol_alpha;
        if (color.a >= 0.95) {
			break;
        }
    }
    return color;
}

@compute @workgroup_size(16, 16, 1)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    textureStore(out_tex, global_id, vec4(0.)); // ???
    let time = un.time * 0.5;

    let dims = vec2<f32>(textureDimensions(out_tex));
    let aspect_ratio = dims.y / dims.x;

    let start_x = f32(global_id.x) / dims.x - 0.5;
    let start_y = -(f32(global_id.y) / dims.y - 0.5) * aspect_ratio;

    let zoom = 1.;
    var eye = vec3(2., 1., 2.);
    eye = 6. * vec3(cos(time), 0., sin(time)) + vec3(0., 1., 0.);
    let camera = get_cam(eye, vec3(0.0));
    let dir = camera * vec3(start_x, start_y, zoom);

    let clear_color = vec4<f32>(0.1, 0.3, 0.3, 0.01);

    if (any(vec2<f32>(global_id.xy) < dims)) {
        var t_hit = intersect_box(eye, dir);
        if (t_hit.x < t_hit.y) {
            t_hit.x = max(t_hit.x, 0.0);
            let col = get_col2(eye, dir, t_hit.x, t_hit.y, clear_color);
            textureStore(out_tex, global_id, col);
        } else {
            textureStore(out_tex, global_id, clear_color);
        }
    }
}
