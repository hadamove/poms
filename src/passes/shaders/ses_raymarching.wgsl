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
    // Add 4 bytes padding to avoid alignment issues.
    _padding: f32,
};

struct LightUniform {
    direction: vec3<f32>,
    color: vec3<f32>,
};


@group(0) @binding(3) var<uniform> ses_grid: GridUniform;

@group(1) @binding(0) var df_texture: texture_3d<f32>;
@group(1) @binding(1) var df_sampler: sampler;

@group(2) @binding(0) var<uniform> camera: CameraUniform;
@group(3) @binding(0) var<uniform> light: LightUniform; 


fn distance_from_df_trilinear(position: vec3<f32>) -> f32 {
    var coord = (position - ses_grid.origin.xyz) / (f32(ses_grid.resolution) * ses_grid.offset);
    return textureSampleLevel(df_texture, df_sampler, coord, 0.).r;
}

fn distance_from_df_tricubic(position: vec3<f32>) -> f32 {
    var resolution = f32(ses_grid.resolution);

    var coord = (position - ses_grid.origin.xyz) / (resolution * ses_grid.offset);
    var coord_grid = resolution * coord - vec3<f32>(0.5);
    var index = floor(coord_grid);

    var fraction = coord_grid - index;
    var one_minus_fraction = vec3<f32>(1.0) - fraction;

    var w0 = 1.0/6.0 * one_minus_fraction * one_minus_fraction * one_minus_fraction;
    var w1 = 2.0/3.0 - 0.5 * fraction * fraction * (2.0 - fraction);
    var w2 = 2.0/3.0 - 0.5 * one_minus_fraction * one_minus_fraction * (2.0 - one_minus_fraction);
    var w3 = 1.0/6.0 * fraction * fraction * fraction;

    var g0 = w0 + w1;
    var g1 = w2 + w3;

    var h0 = (w1 / g0 - 0.5 + index) / resolution;
    var h1 = (w3 / g1 + 1.5 + index) / resolution;

	// Fetch the eight linear interpolations.
	var tex000 = textureSampleLevel(df_texture, df_sampler, h0, 0.).r;
	var tex100 = textureSampleLevel(df_texture, df_sampler, vec3(h1.x, h0.y, h0.z), 0.).r;
	tex000 = mix(tex100, tex000, g0.x);

	var tex010 = textureSampleLevel(df_texture, df_sampler, vec3(h0.x, h1.y, h0.z), 0.).r;
	var tex110 = textureSampleLevel(df_texture, df_sampler, vec3(h1.x, h1.y, h0.z), 0.).r;
	tex010 = mix(tex110, tex010, g0.x);
	tex000 = mix(tex010, tex000, g0.y);

	var tex001 = textureSampleLevel(df_texture, df_sampler, vec3(h0.x, h0.y, h1.z), 0.).r;
	var tex101 = textureSampleLevel(df_texture, df_sampler, vec3(h1.x, h0.y, h1.z), 0.).r;
	tex001 = mix(tex101, tex001, g0.x);

	var tex011 = textureSampleLevel(df_texture, df_sampler, vec3(h0.x, h1.y, h1.z), 0.).r;
	var tex111 = textureSampleLevel(df_texture, df_sampler, h1, 0.).r;
	tex011 = mix(tex111, tex011, g0.x);
	tex001 = mix(tex011, tex001, g0.y);

    return mix(tex001, tex000, g0.z);
}

struct RayHit {
    hit: bool,
    position: vec3<f32>,
    color: vec3<f32>,
};

fn ray_march(origin: vec3<f32>, direction: vec3<f32>) -> RayHit {
    var MAX_STEPS = 128u;
    var MINIMUM_HIT_DISTANCE: f32 = 0.05;
    var TRICUBIC_THRESHOLD: f32 = 0.1;

    var rayhit: RayHit;
    rayhit.hit = false;

    // Find closest intersection with the bounding box grid.
    var tmin = (ses_grid.origin.xyz - origin) / direction;
    var tmax = (ses_grid.origin.xyz + vec3<f32>(f32(ses_grid.resolution) * ses_grid.offset) - origin) / direction;

    var t0 = min(tmin, tmax);
    var t1 = max(tmin, tmax);

    var tnear = max(t0.x, max(t0.y, t0.z));
    var tfar = min(t1.x, min(t1.y, t1.z));

    if (tnear > tfar) {
        return rayhit;
    }

    var total_distance: f32 = tnear;

    for (var i: u32 = 0u; i < MAX_STEPS; i += 1u) {
        var current_position: vec3<f32> = origin + total_distance * direction;

        // First sample the distance field using trilinear interpolation.
        var distance_trilinear = distance_from_df_trilinear(current_position);
        if (distance_trilinear > TRICUBIC_THRESHOLD) {
            total_distance += distance_trilinear;
            continue;
        }

        // If the distance is too large, sample the using tricubic interpolation.
        var distance = distance_from_df_tricubic(current_position);

        if (distance < MINIMUM_HIT_DISTANCE) {
            // Calculate normal.
            var small_step = vec3<f32>(0.03, 0.0, 0.0) * ses_grid.offset;

            var p = current_position + distance * direction;
            var gradient_x = distance_from_df_tricubic(p + small_step.xyy) - distance_from_df_tricubic(p - small_step.xyy);
            var gradient_y = distance_from_df_tricubic(p + small_step.yxy) - distance_from_df_tricubic(p - small_step.yxy);
            var gradient_z = distance_from_df_tricubic(p + small_step.yyx) - distance_from_df_tricubic(p - small_step.yyx);

            var normal = normalize(vec3<f32>(gradient_x, gradient_y, gradient_z));

            var color = vec3<f32>(1.0);
            var ambient = 0.15;

            var light_dir = normalize(light.direction);
            var diff =  max(0.0, dot(normal, light_dir));

            var reflect_dir = reflect(light_dir, normal);  
            var spec = pow(max(dot(direction, reflect_dir), 0.0), 16.0) * 0.3;

            rayhit.color = color * (ambient + spec + diff) * light.color;

            rayhit.hit = true;
            rayhit.position = p;
            return rayhit;
        }
        total_distance += distance;

        // Make sure we don't march too far.
        if (total_distance > tfar) {
            return rayhit;
        }
    }
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
