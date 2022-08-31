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
    // Add 8 bytes padding to avoid alignment issues.
    _padding: vec2<f32>,
};

struct Atom {
    position: vec3<f32>,
    radius: f32,
    color: vec4<f32>,
};

struct SortedAtomBuffer {
    atoms: array<Atom>,
};

struct GridCellStartIndicesBuffer {
    indices: array<u32>,
};

@group(0) @binding(0) var<uniform> camera: CameraUniform;
@group(1) @binding(1) var<uniform> ses_grid: GridUniform;
@group(2) @binding(2) var<uniform> neighbor_grid: GridUniform;
@group(3) @binding(3) var<storage, read> sorted_atoms: SortedAtomBuffer;
@group(4) @binding(4) var<storage, read> grid_cell_start: GridCellStartIndicesBuffer;

fn distance_from_sphere(point: vec3<f32>, center: vec3<f32>, radius: f32) -> f32 {
    return length(point  - center) - radius;
}

struct DistanceAndColor {
    distance: f32,
    color: vec3<f32>,
};

fn map_the_grid(point: vec3<f32>) -> DistanceAndColor {
    
    var out: DistanceAndColor;

    var GRID_POINTS_RADIUS = 0.4;

    var grid_space_point = point - ses_grid.origin.xyz;
    var grid_space_coords = vec3<i32>(grid_space_point / ses_grid.offset);
    
    var res = i32(ses_grid.resolution);

    if (grid_space_coords.x < 0   || grid_space_coords.y < 0   || grid_space_coords.z < 0 
     || grid_space_coords.x > res || grid_space_coords.y > res || grid_space_coords.z > res) {
        // Point is outside the grid.
        out.distance = 1.0;
        return out;
    }

    // Compute distance to the nearest grid point.
    var nearest_grid_point = ses_grid.origin.xyz + vec3<f32>(grid_space_coords) * ses_grid.offset + vec3<f32>(GRID_POINTS_RADIUS);
    out.distance = distance_from_sphere(point, nearest_grid_point, GRID_POINTS_RADIUS);
    out.color = point - nearest_grid_point;

    return out;
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
        var distance_and_color = map_the_grid(current_position);

        if (distance_and_color.distance < MINIMUM_HIT_DISTANCE) {
            rayhit.hit = true;
            rayhit.point = current_position;
            rayhit.color = distance_and_color.color;
            return rayhit;
        }
        total_distance += distance_and_color.distance;
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
