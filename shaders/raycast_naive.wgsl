type float2 = vec2<f32>;
type float3 = vec3<f32>;
type float4 = vec4<f32>;

struct VertexInput {
    @location(0) position: float3,
};

struct VertexOutput {
    @builtin(position) position: float4,
    @location(0) transformed_eye: float3,
    @location(1) ray_dir: float3,
};

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
	proj_view: mat4x4<f32>,
	inv_proj: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> un: Uniform;
@group(1) @binding(0)
var<uniform> cam: Camera;
@group(2) @binding(0)
var volume: texture_3d<f32>;
@group(2) @binding(1)
var tex_sampler: sampler;

@vertex
fn vs_main(vert: VertexInput) -> VertexOutput {
    var vout: VertexOutput;
    var pos = vert.position;
    vout.position = cam.proj_view * vec4<f32>(pos, 1.0);
    vout.transformed_eye = cam.view_pos.xyz;
    vout.ray_dir = pos - vout.transformed_eye;
    return vout;
}

fn intersect_box(orig: float3, dir: float3) -> float2 {
    let box_min = vec3<f32>(0.0);
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

fn linear_to_srgb(x: f32) -> f32 {
    if (x <= 0.0031308) {
        return 12.92 * x;
    }
    return 1.055 * pow(x, 1.0 / 2.4) - 0.055;
}

let TAU: f32 = 6.28318;
fn palette(t: f32, a: float3, b: float3, c: float3, d: float3) -> float3 {
    return a + b * cos(TAU * (c * t + d));
}

fn vertigo(t: f32) -> float3 {
    let a = vec3<f32>(0.5);
    let b = vec3<f32>(0.5);
    let c = vec3<f32>(1.0, 1.7, 0.4);
    let d = vec3<f32>(0.0, 0.15, 0.20);
    return palette(t, a, b, c, d);
}

@fragment
fn fs_main(vin: VertexOutput) -> @location(0) float4 {
    var ray_dir = normalize(vin.ray_dir);
    let eye = vin.transformed_eye;

    let background = vec4<f32>(0.1, 0.2, 0.3, 0.01);

    var t_hit = intersect_box(eye, ray_dir);
    if (t_hit.x > t_hit.y) {
        return vec4<f32>(0., 0., 0., 1.);
    }
    t_hit.x = max(t_hit.x, 0.0);

    var color = vec4<f32>(0.0);
    let dt_vec = 1.0 / (vec3<f32>(256.0) * abs(ray_dir));
    let dt_scale = 1.0;
    let dt = dt_scale * min(dt_vec.x, min(dt_vec.y, dt_vec.z));
    var p = eye + t_hit.x * ray_dir;
    for (var t = t_hit.x; t < t_hit.y; t = t + dt) {
        let tex_content = textureSampleLevel(volume, tex_sampler, p, 0.0);
        var val = tex_content.rgb;
        let val_alpha = pow(tex_content.a, 2.0);

        val = clamp(vec3<f32>(0.4), vec3<f32>(.9), val);
        val = smoothstep(vec3<f32>(0.10), vec3<f32>(1.2), val);
        var val_color = vec4<f32>(vertigo(val.r), val.r);

		// Opacity correction
        // val_color.a = 1.0 - pow(1.0 - val_color.a, dt_scale);
        var tmp = color.rgb + (1.0 - color.a) * val_color.a * val_color.xyz + background.rgb * background.a * (1. - val_alpha);
        color = vec4<f32>(tmp, color.a);
        color.a = color.a + (1.0 - color.a) * val_color.a;
        if (color.a >= 0.95) {
			break;
        }
        p = p + ray_dir * dt;
    }

    color.r = linear_to_srgb(color.r);
    color.g = linear_to_srgb(color.g);
    color.b = linear_to_srgb(color.b);
    return vec4<f32>(color.rgb, 1.);
}
