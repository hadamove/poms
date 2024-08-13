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
};

// Distance Field Resource
@group(0) @binding(0) var<uniform> df_grid: GridUniform;
@group(0) @binding(1) var df_texture: texture_3d<f32>;
@group(0) @binding(2) var df_sampler: sampler;

// Camera & Light Resources
@group(1) @binding(0) var<uniform> camera: CameraUniform;
@group(2) @binding(0) var<uniform> light: LightUniform; 


fn distance_from_df_trilinear(position: vec3<f32>) -> f32 {
    let tex_coord: vec3<f32> = (position - df_grid.origin.xyz) / (f32(df_grid.resolution) * df_grid.offset);
    return textureSampleLevel(df_texture, df_sampler, tex_coord, 0.).r;
}

fn distance_from_df_tricubic(position: vec3<f32>) -> f32 {
    let resolution = f32(df_grid.resolution);

    let coord: vec3<f32> = (position - df_grid.origin.xyz) / (resolution * df_grid.offset);
    let coord_grid = resolution * coord - vec3<f32>(0.5);
    let index = floor(coord_grid);

    let fraction = coord_grid - index;
    let one_minus_fraction = vec3<f32>(1.0) - fraction;

    let w0 = 1.0/6.0 * one_minus_fraction * one_minus_fraction * one_minus_fraction;
    let w1 = 2.0/3.0 - 0.5 * fraction * fraction * (2.0 - fraction);
    let w2 = 2.0/3.0 - 0.5 * one_minus_fraction * one_minus_fraction * (2.0 - one_minus_fraction);
    let w3 = 1.0/6.0 * fraction * fraction * fraction;

    let g0 = w0 + w1;
    let g1 = w2 + w3;

    let h0 = (w1 / g0 - 0.5 + index) / resolution;
    let h1 = (w3 / g1 + 1.5 + index) / resolution;

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

const MAX_STEPS: u32 = 160u;
const MINIMUM_HIT_DISTANCE: f32 = 0.05;
const TRICUBIC_THRESHOLD: f32 = 0.1;

const NO_HIT: RayHit = RayHit(false, vec3<f32>(0.0), vec3<f32>(0.0));
const SURFACE_COLOR: vec3<f32> = vec3<f32>(1.0, 0.8, 0.8);

fn ray_march(origin: vec3<f32>, direction: vec3<f32>) -> RayHit {
    // Find closest intersection with the bounding box grid.
    let tmin = (df_grid.origin.xyz - origin) / direction;
    let tmax = (df_grid.origin.xyz + vec3<f32>(f32(df_grid.resolution) * df_grid.offset) - origin) / direction;

    let t0 = min(tmin, tmax);
    let t1 = max(tmin, tmax);

    let tnear = max(t0.x, max(t0.y, t0.z));
    let tfar = min(t1.x, min(t1.y, t1.z));

    if (tnear > tfar) {
        return NO_HIT;
    }

    var total_distance: f32 = tnear;

    for (var i: u32 = 0u; i < MAX_STEPS; i += 1u) {
        let current_position: vec3<f32> = origin + total_distance * direction;

        // First sample the distance field using trilinear interpolation for early termination.
        let distance_trilinear: f32 = distance_from_df_trilinear(current_position);
        if (distance_trilinear > TRICUBIC_THRESHOLD) {
            total_distance += distance_trilinear;
            continue;
        }

        // If the distance is too large, sample the using tricubic interpolation.
        let distance: f32 = distance_from_df_tricubic(current_position);

        if (distance < MINIMUM_HIT_DISTANCE) {
            // Calculate normal.
            let small_step = vec3<f32>(0.03, 0.0, 0.0) * df_grid.offset;

            let p: vec3<f32> = current_position + distance * direction;
            let gradient_x: f32 = distance_from_df_tricubic(p + small_step.xyy) - distance_from_df_tricubic(p - small_step.xyy);
            let gradient_y: f32 = distance_from_df_tricubic(p + small_step.yxy) - distance_from_df_tricubic(p - small_step.yxy);
            let gradient_z: f32 = distance_from_df_tricubic(p + small_step.yyx) - distance_from_df_tricubic(p - small_step.yyx);

            let normal: vec3<f32> = normalize(vec3<f32>(gradient_x, gradient_y, gradient_z));

            let color = vec3<f32>(1.0);
            let ambient: f32 = 0.15;

            let light_dir: vec3<f32> = normalize(light.direction);
            let diffuse: f32 =  max(0.0, dot(normal, light_dir));

            let reflect_dir: vec3<f32> = reflect(light_dir, normal);  
            let specular: f32 = pow(max(dot(direction, reflect_dir), 0.0), 16.0) * 0.3;

            let color_shaded = color * (ambient + specular + diffuse) * SURFACE_COLOR;

            return RayHit(true, p, color_shaded);
        }
        total_distance += distance;

        // Make sure we don't march too far.
        if (total_distance > tfar) {
            return NO_HIT;
        }
    }

    // Ray missed in the maximum number of steps.
    return NO_HIT;
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

    let x: f32 = quad_vertices[in_vertex_index].x;
    let y: f32 = quad_vertices[in_vertex_index].y;

    let clip_position = vec4<f32>(x, y, 0.0, 1.0);
    let uv = vec2<f32>(x, y);

    return VertexOutput(clip_position, uv);
}

struct FragmentOutput {
    @builtin(frag_depth) depth: f32,
    @location(0) color: vec4<f32>,
};

@fragment
fn fs_main(in: VertexOutput) -> FragmentOutput {
    // Ray starts at the camera position.
    let ray_origin: vec3<f32> = camera.pos.xyz;

    let ray_direction_pixel: vec4<f32> = vec4<f32>(in.uv, -1.0, 1.0);
    // Apply inverse projection matrix to get the ray in view space.
    let ray_direction_view = vec4<f32>(
        (camera.proj_inverse * ray_direction_pixel).xyz, 0.0
    );
    // Apply inverse view matrix to get the ray in world space.
    let ray_direction_world: vec4<f32> = camera.view_inverse * ray_direction_view;

    let rayhit = ray_march(ray_origin, normalize(ray_direction_world.xyz));
    if (!rayhit.hit) {
        // Ray missed.
        discard;
    }

    // Calculate the distance from the camera to the hit position.
    let rayhit_point_proj: vec4<f32> = camera.proj * camera.view * vec4<f32>(rayhit.position, 1.0);
    let rayhit_depth: f32 = rayhit_point_proj.z / rayhit_point_proj.w;

    let color = vec4<f32>(rayhit.color, 1.0);

    return FragmentOutput(rayhit_depth, color);
}
