struct CameraUniform {
    pos: vec4<f32>,
    view: mat4x4<f32>,
    proj: mat4x4<f32>,
    view_inverse: mat4x4<f32>,
    proj_inverse: mat4x4<f32>,
};


struct GridUniform {
    origin: vec4<f32>,
    resolution: u32,
    offset: f32,
    size: f32,
    // Add 8 bytes padding to avoid alignment issues.
    _padding: f32,
};

@group(0) @binding(0) var<uniform> camera: CameraUniform;

@group(1) @binding(0) var<uniform> ses_grid: GridUniform;
@group(1) @binding(1) var<storage, read> distance_field: array<f32>;


fn grid_point_index_to_position(grid_point_index: u32) -> vec3<f32> {
    return ses_grid.origin.xyz + vec3<f32>(
        f32(grid_point_index % ses_grid.resolution),
        f32((grid_point_index / ses_grid.resolution) % ses_grid.resolution),
        f32(grid_point_index / (ses_grid.resolution * ses_grid.resolution))
    ) * ses_grid.offset;
}

fn distance_from_df(position: vec3<f32>) -> f32 {
    if (position.x < ses_grid.origin.x || position.y < ses_grid.origin.y || position.z < ses_grid.origin.z ||
        position.x > ses_grid.origin.x + f32(ses_grid.resolution) * ses_grid.offset ||
        position.y > ses_grid.origin.y + f32(ses_grid.resolution) * ses_grid.offset ||
        position.z > ses_grid.origin.z + f32(ses_grid.resolution) * ses_grid.offset) {
        // Point is outside the grid.
        return 1.2;
    }

    var grid_space_point = position - ses_grid.origin.xyz;
    var grid_space_coords = vec3<i32>(grid_space_point / ses_grid.offset);
    
    var res = i32(ses_grid.resolution);

    var grid_point_index = grid_space_coords.x +
        grid_space_coords.y * res +
        grid_space_coords.z * res * res;
    

    var grid_position = grid_point_index_to_position(u32(grid_point_index));
    var weight = (position - grid_position) / ses_grid.offset;

    var d000 = distance_field[grid_point_index];
    var d100 = distance_field[grid_point_index + 1];
    var d010 = distance_field[grid_point_index + res];
    var d001 = distance_field[grid_point_index + res * res];
    var d110 = distance_field[grid_point_index + res + 1];
    var d011 = distance_field[grid_point_index + res * res + res];
    var d101 = distance_field[grid_point_index + res * res + 1];
    var d111 = distance_field[grid_point_index + res * res + res + 1];

    var d00 = mix(d000, d100, weight.x);
    var d01 = mix(d001, d101, weight.x);
    var d10 = mix(d010, d110, weight.x);
    var d11 = mix(d011, d111, weight.x);

    var d0 = mix(d00, d10, weight.y);
    var d1 = mix(d01, d11, weight.y);

    var d = mix(d0, d1, weight.z);

    return d;
}

struct RayHit {
    hit: bool,
    position: vec3<f32>,
    color: vec3<f32>,
};

fn ray_march(origin: vec3<f32>, direction: vec3<f32>) -> RayHit {
    var MAX_STEPS = 128u;
    var MINIMUM_HIT_DISTANCE: f32 = 1.1;

    var rayhit: RayHit;
    var total_distance: f32 = 0.0;

    for (var i: u32 = 0u; i < MAX_STEPS; i += 1u) {
        var current_position: vec3<f32> = origin + total_distance * direction;
        var distance = distance_from_df(current_position);

        if (distance < 0.5) {
            // calculate normal 
            var small_step = vec3<f32>(0.01, 0.0, 0.0);

            var p = current_position + distance * direction;
            var gradient_x = distance_from_df(p + small_step.xyy) - distance_from_df(p - small_step.xyy);
            var gradient_y = distance_from_df(p + small_step.yxy) - distance_from_df(p - small_step.yxy);
            var gradient_z = distance_from_df(p + small_step.yyx) - distance_from_df(p - small_step.yyx);

            var normal = normalize(vec3<f32>(gradient_x, gradient_y, gradient_z));

            rayhit.hit = true;
            rayhit.position = current_position + distance * direction;
            rayhit.color = normal * 0.5 + 0.5;
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

@vertex
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

@fragment
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

    // Calculate the distance from the camera to the hit position.
    var rayhit_point_proj = camera.proj * camera.view * vec4<f32>(rayhit.position, 1.0);
    var rayhit_depth = rayhit_point_proj.z / rayhit_point_proj.w;

    var out: FragmentOutput;
    out.color = vec4<f32>(rayhit.color, 1.0);
    out.depth = rayhit_depth;

    return out;
}
