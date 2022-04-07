struct CameraUniform {
    view: mat4x4<f32>;
    proj: mat4x4<f32>;
};

struct Atom {
    position: vec3<f32>;
    radius: f32;
    color: vec4<f32>;
};

struct AtomBuffer {
    atoms: array<Atom>;
};

[[group(0), binding(0)]] var<uniform> camera: CameraUniform;
[[group(1), binding(1)]] var<storage, read> atoms: AtomBuffer;


struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] color: vec4<f32>;
    [[location(1)]] uv: vec2<f32>;
    [[location(2)]] proj_position: vec4<f32>;
    [[location(3)]] atom_radius: f32;
};


// Renders atoms using sphere impostor technique on quad billboards
// Atom data (position, color, radius) is fetched from a storage buffer
[[stage(vertex)]]
fn vs_main(
    [[builtin(vertex_index)]] vertex_index: u32,
) -> VertexOutput {

    var atom_index = vertex_index / 6u;
    var atom = atoms.atoms[atom_index];

    var quad_vertices = array<vec2<f32>, 6>(
        vec2<f32>(-1.0,  1.0),
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 1.0, -1.0),
        vec2<f32>(-1.0,  1.0),
        vec2<f32>( 1.0, -1.0),
        vec2<f32>( 1.0,  1.0)
    );

    var atom_pos = vec4<f32>(atom.position, 1.0);
    var quad_pos = quad_vertices[vertex_index % 6u];
    var vertex_pos = atom.radius * quad_pos;

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
    [[builtin(frag_depth)]] depth: f32;
    [[location(0)]] color: vec4<f32>;
};

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> FragmentOutput {

    var dist_xy = pow(in.uv.x, 2.0) + pow(in.uv.y, 2.0);

    if (dist_xy > 1.0) {
        discard;
    }

    var z = sqrt(1.0 - dist_xy);
    var offset_z = camera.proj * in.atom_radius * vec4<f32>(0.0, 0.0, z, 1.0);
    var proj_surface_position = in.proj_position + offset_z;

    var shading = vec4<f32>(in.uv, 1.0, 1.0) * 0.2;

    var out: FragmentOutput;
    out.color = in.color + shading;
    out.depth = proj_surface_position.z / proj_surface_position.w;

    return out;
}
