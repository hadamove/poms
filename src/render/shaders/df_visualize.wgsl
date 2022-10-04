struct SettingsUniform {
    df_size: u32,
    layer_offset: u32,
}

@group(0) @binding(0) var<storage, read> distance_field: array<f32>;
@group(0) @binding(1) var<uniform> settings: SettingsUniform;


struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    var quad_vertices = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, 1.0),
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(1.0, -1.0),
        vec2<f32>(-1.0, 1.0),
        vec2<f32>(1.0, -1.0),
        vec2<f32>(1.0, 1.0),
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
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let x = (in.uv.x + 1.0) * 0.5 * f32(settings.df_size);
    let y = (in.uv.y + 1.0) * 0.5 * f32(settings.df_size);

    let layer_offset = settings.layer_offset * settings.df_size * settings.df_size;
    let offset = u32(x) + u32(y) * settings.df_size + layer_offset;


    let value = (distance_field[offset] + 1.2) / 2.4;


    return vec4<f32>(
        value - 0.5,
        value - 0.5,
        value,
        1.0,
    );
}
