struct SettingsUniform {
    df_size: u32,
    layer_offset: u32,
}

@group(0) @binding(0) var<uniform> settings: SettingsUniform;

@group(0) @binding(1) var df_texture: texture_3d<f32>;
@group(0) @binding(2) var df_sampler: sampler;

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
    let x = (in.uv.x + 1.0) * 0.5;
    let y = (in.uv.y + 1.0) * 0.5;
    let z = f32(settings.layer_offset) / f32(settings.df_size);

    let value = textureSample(df_texture, df_sampler, vec3<f32>(x, y, z));

    return value;
}
