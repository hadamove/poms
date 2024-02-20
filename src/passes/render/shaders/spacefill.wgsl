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

@group(0) @binding(0) var<storage, read> atoms: AtomBuffer;
@group(1) @binding(0) var<uniform> camera: CameraUniform;


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

    var atom_index: u32 = vertex_index / 6u;
    var atom: Atom = atoms.atoms[atom_index];

    var quad_vertices = array<vec2<f32>, 6>(
        vec2<f32>(-1.0,  1.0),
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 1.0, -1.0),
        vec2<f32>(-1.0,  1.0),
        vec2<f32>( 1.0, -1.0),
        vec2<f32>( 1.0,  1.0)
    );

    var atom_pos = vec4<f32>(atom.position, 1.0);
    var quad_pos: vec2<f32> = quad_vertices[vertex_index % 6u];
    var vertex_pos: vec2<f32> = atom.radius * quad_pos;

    var camera_right_worldspace = vec3<f32>(camera.view[0][0], camera.view[1][0], camera.view[2][0]);
    var camera_up_worldspace = vec3<f32>(camera.view[0][1], camera.view[1][1], camera.view[2][1]);

    var worldspace_pos = vec4<f32>(
        atom_pos.xyz +
        vertex_pos.x * camera_right_worldspace +
        vertex_pos.y * camera_up_worldspace,
        1.0
    );

    var out: VertexOutput;
    out.color = atom.color;
    out.uv = quad_pos;
    out.proj_position = camera.proj * camera.view * worldspace_pos;
    out.atom_radius = atom.radius;
    out.clip_position = out.proj_position;

    return out;
}

struct FragmentOutput {
    @builtin(frag_depth) depth: f32,
    @location(0) color: vec4<f32>,
};

@fragment
fn fs_main(in: VertexOutput) -> FragmentOutput {

    var dist_xy: f32 = dot(in.uv, in.uv);

    // Discard fragments outside of the unit circle.
    if (dist_xy > 1.0) {
        discard;
    }

    // Compute the distance to the sphere surface.
    var z: f32 = sqrt(1.0 - dist_xy);
    var offset_z: vec4<f32> = camera.proj * vec4<f32>(0.0, 0.0, z * in.atom_radius, 0.0);
    var proj_surface_position: vec4<f32> = in.proj_position + offset_z;

    var out: FragmentOutput;

    var normal = vec3<f32>(in.uv, z);
    var ambient: f32 = 0.15;

    var view_dir = vec3<f32>(0.0, 0.0, -1.0);
    var light_dir: vec3<f32> = normalize(-view_dir);
    var diffuse: f32 =  max(0.0, dot(normal, light_dir));

    var reflect_dir: vec3<f32> = reflect(light_dir, normal);  
    var specular: f32 = pow(max(dot(view_dir, reflect_dir), 0.0), 16.0) * 0.3;

    var color: vec3<f32> = in.color.xyz * (ambient + specular + diffuse);

    out.color = vec4<f32>(color, 1.0);
    out.depth = proj_surface_position.z / proj_surface_position.w;

    return out;
}
