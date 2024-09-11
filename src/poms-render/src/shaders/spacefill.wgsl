struct CameraUniform {
    pos: vec4<f32>,
    view: mat4x4<f32>,
    proj: mat4x4<f32>,
    view_inverse: mat4x4<f32>,
    proj_inverse: mat4x4<f32>,
};

struct Atom {
    position: vec3<f32>,
    radius: f32,
    color: vec4<f32>,
};

struct AtomBuffer {
    atoms: array<Atom>,
};

// Camera Resource
@group(0) @binding(0) var<uniform> camera: CameraUniform;

// Atoms Resource
@group(1) @binding(0) var<storage, read> atoms: AtomBuffer;


struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) proj_position: vec4<f32>,
    @location(3) atom_radius: f32,
};


// Renders atoms using sphere impostor technique on quad billboards.
// Atom data (position, color, radius) is fetched from a storage buffer.
@vertex
fn vs_main(
    @builtin(vertex_index) vertex_index: u32,
) -> VertexOutput {

    let atom_index: u32 = vertex_index / 6u;
    let atom: Atom = atoms.atoms[atom_index];

    var quad_vertices = array<vec2<f32>, 6>(
        vec2<f32>(-1.0,  1.0),
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 1.0, -1.0),
        vec2<f32>(-1.0,  1.0),
        vec2<f32>( 1.0, -1.0),
        vec2<f32>( 1.0,  1.0)
    );

    let atom_pos = vec4<f32>(atom.position, 1.0);
    let quad_pos: vec2<f32> = quad_vertices[vertex_index % 6u];
    let vertex_pos: vec2<f32> = atom.radius * quad_pos;

    let camera_right_worldspace = vec3<f32>(camera.view[0][0], camera.view[1][0], camera.view[2][0]);
    let camera_up_worldspace = vec3<f32>(camera.view[0][1], camera.view[1][1], camera.view[2][1]);

    let worldspace_pos = vec4<f32>(
        atom_pos.xyz +
        vertex_pos.x * camera_right_worldspace +
        vertex_pos.y * camera_up_worldspace,
        1.0
    );

    let proj_position: vec4<f32> = camera.proj * camera.view * worldspace_pos;

    return VertexOutput(
        proj_position,
        atom.color,
        quad_pos, 
        proj_position,
        atom.radius
    );        
}

struct FragmentOutput {
    @builtin(frag_depth) depth: f32,
    @location(0) color: vec4<f32>,
    @location(1) normal: vec4<f32>,
};

@fragment
fn fs_main(in: VertexOutput) -> FragmentOutput {

    let dist_xy: f32 = dot(in.uv, in.uv);

    // Discard fragments outside of the unit circle.
    if (dist_xy > 1.0) {
        discard;
    }

    // Compute the distance to the sphere surface.
    let z: f32 = sqrt(1.0 - dist_xy);
    let offset_z: vec4<f32> = camera.proj * vec4<f32>(0.0, 0.0, z * in.atom_radius, 0.0);
    let proj_surface_position: vec4<f32> = in.proj_position + offset_z;

    let normal = vec3<f32>(in.uv, z);
    let ambient: f32 = 0.15;

    // Compute diffuse reflection.
    let view_dir = vec3<f32>(0.0, 0.0, -1.0);
    let light_dir: vec3<f32> = normalize(-view_dir);
    let diffuse: f32 =  max(0.0, dot(normal, light_dir));

    // Compute specular reflection.
    let reflect_dir: vec3<f32> = reflect(light_dir, normal);  
    let specular: f32 = pow(max(dot(view_dir, reflect_dir), 0.0), 16.0) * 0.3;

    let depth = proj_surface_position.z / proj_surface_position.w;
    let color = vec4<f32>(in.color.xyz * (ambient + specular + diffuse), 1.0);

    return FragmentOutput(depth, color, vec4<f32>(normalize(normal), 0.0));
}
