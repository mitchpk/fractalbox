struct CameraUniform {
    pos: vec2<f64>,
    zoom: f32,
    aspect: f32,
};
@group(1)
@binding(0)
var<uniform> camera: CameraUniform;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

@vertex
fn vs_main(
    @location(0) pos: vec3<f32>
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(pos, 1.0);
    out.tex_coords = vec2<f32>(pos.x * camera.aspect, -pos.y);
    return out;
}

// Fragment shader

struct FragmentInput {
    @location(0) tex_coords: vec2<f32>,
};

@group(2)
@binding(0)
var<uniform> frame_count: f32;

fn compute_next(current: vec2<f64>, constant: vec2<f64>) -> vec2<f64> {
    let zr = current.x * current.x - current.y * current.y;
    let zi = current.x * current.y * f64(2.0);
    return vec2<f64>(zr, zi) + constant;
}

fn compute_iterations(z0: vec2<f64>, constant: vec2<f64>, max_iteration: i32) -> f32 {
    var zn = z0;
    var iteration = 0;
    var length = zn.x * zn.x + zn.y * zn.y;
    var is_inside = true;
    while iteration < max_iteration {
        zn = compute_next(zn, constant);
        length = zn.x * zn.x + zn.y * zn.y;
        iteration += 1;
        if length > f64(4.0) {
            is_inside = false;
            break;
        }
    }

    if is_inside {
        return -1.0;
    }

    zn = compute_next(zn, constant);
    length = zn.x * zn.x + zn.y * zn.y;
    iteration += 1;
    zn = compute_next(zn, constant);
    length = zn.x * zn.x + zn.y * zn.y;
    iteration += 1;

    let smooth_iteration = f32(iteration) - log2(max(1.0, log2(f32(length))));
    return smooth_iteration / f32(max_iteration);
}

fn get_colour(iterations: f32) -> vec3<f32> {
    return vec3<f32>(
        pow(cos(sqrt(iterations)*1.0 + 0.0), 2.0),
        pow(cos(sqrt(iterations)*1.0 + 120.0), 2.0),
        pow(cos(sqrt(iterations)*1.0 + 240.0), 2.0),
    );
}

@fragment
fn fs_main(in: FragmentInput) -> @location(0) vec4<f32> {
    let result = compute_iterations(
        vec2<f64>(in.tex_coords) * f64(exp(-camera.zoom)
    ) + camera.pos, vec2<f64>(f64(-0.79), f64(0.15)), 200);

    return vec4<f32>(get_colour(result), 1.0);
}
