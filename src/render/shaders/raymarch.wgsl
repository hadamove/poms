struct CameraUniform {
    pos: vec4<f32>,
    view: mat4x4<f32>,
    proj: mat4x4<f32>,
    view_inverse: mat4x4<f32>,
    proj_inverse: mat4x4<f32>,
};

@group(0) @binding(0) var<uniform> camera: CameraUniform;

fn distance_from_sphere(point: vec3<f32>, center: vec3<f32>, radius: f32) -> f32 {
    return length(point  - center) - radius;
}

struct RayHit {
    hit: bool,
    point: vec3<f32>,
    color: vec3<f32>,
};

fn ray_march(origin: vec3<f32>, direction: vec3<f32>) -> RayHit {
    var MAX_STEPS = 128u;
    var MINIMUM_HIT_DISTANCE: f32 = 0.001;
    var SPHERE_CENTER = vec3<f32>(79.17576, -51.610718, -5.8748875);

    var rayhit: RayHit;
    var total_distance: f32 = 0.0;

    for (var i: u32 = 0u; i < MAX_STEPS; i += 1u) {
        var current_position: vec3<f32> = origin + total_distance * direction;
        var distance = distance_from_sphere(current_position, SPHERE_CENTER, 10.0);

        if (distance < MINIMUM_HIT_DISTANCE) {
            rayhit.hit = true;
            rayhit.point = current_position;
            rayhit.color = vec3<f32>(1.0, 0.0, 1.0);
            return rayhit;
        }
        total_distance += distance;
    }
    rayhit.hit = false;
    return rayhit;
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@stage(vertex)
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    var quad_vertices = array<vec2<f32>, 6>(
        vec2<f32>(-1.0,  1.0),
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 1.0, -1.0),
        vec2<f32>(-1.0,  1.0),
        vec2<f32>( 1.0, -1.0),
        vec2<f32>( 1.0,  1.0)
    );

    var out: VertexOutput;
    let x = quad_vertices[in_vertex_index].x;
    let y = quad_vertices[in_vertex_index].y;

    out.clip_position = vec4<f32>(x, y, 0.0, 1.0);
    out.uv = vec2<f32>(x, y);

    return out;
}

struct FragmentOutput {
    @builtin(frag_depth) depth: f32,
    @location(0) color: vec4<f32>,
};

@stage(fragment)
fn fs_main(in: VertexOutput) -> FragmentOutput {
    // Ray starts at the camera position.
    var ray_origin: vec3<f32> = camera.pos.xyz;

    var ray_direction_pixel: vec4<f32> = vec4<f32>(in.uv, -1.0, 1.0);
    // Apply inverse projection matrix to get the ray in view space.
    var ray_direction_view = vec4<f32>(
        (camera.proj_inverse * ray_direction_pixel).xyz, 0.0
    );
    // Apply inverse view matrix to get the ray in world space.
    var ray_direction_world = camera.view_inverse * ray_direction_view;

    var rayhit = ray_march(ray_origin, normalize(ray_direction_world.xyz));
    if (!rayhit.hit) {
        // Ray missed.
        discard;
    }

    // Calculate the distance from the camera to the hit point.
    var rayhit_point_proj = camera.proj * camera.view * vec4<f32>(rayhit.point, 1.0);
    var rayhit_depth = rayhit_point_proj.z / rayhit_point_proj.w;

    var out: FragmentOutput;
    out.color = vec4<f32>(rayhit.color, 1.0);
    out.depth = rayhit_depth;

    return out;
}
