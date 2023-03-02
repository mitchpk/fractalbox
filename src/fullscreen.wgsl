struct CameraUniform {
    pos: vec2<f64>,
    zoom: f64
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
    out.tex_coords = vec2<f32>(pos.x, -pos.y);
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

fn compute_iterations(z0: vec2<f64>, constant: vec2<f64>, max_iteration: i32) -> i32 {
    var zn = z0;
    var iteration = 0;
    while zn.x * zn.x + zn.y * zn.y < f64(4.0) && iteration < max_iteration {
        zn = compute_next(zn, constant);
        iteration += 1;
    }
    return iteration;
}

fn get_colour(iterations: i32) -> vec3<f32> {
    return vec3<f32>(f32(iterations) / 500.0);
}

@fragment
fn fs_main(in: FragmentInput) -> @location(0) vec4<f32> {
    let result = compute_iterations(vec2<f64>(in.tex_coords) * f64(exp(f32(camera.zoom))) + camera.pos, vec2<f64>(f64(-0.56), f64(0.55)), 500);
    return vec4<f32>(get_colour(result), 1.0);
}
