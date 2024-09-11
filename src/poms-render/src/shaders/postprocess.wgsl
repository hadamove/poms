struct PostprocessUniforms {
    is_ssao_enabled: u32,
};

@group(0) @binding(0) var<uniform> settings: PostprocessUniforms;

@group(0) @binding(1) var color_texture: texture_2d<f32>;
@group(0) @binding(2) var ssao_texture: texture_2d<f32>;

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> @builtin(position) vec4<f32> {
    // Fullscreen triangle
    switch (vertex_index) {
        case 0u: { return vec4<f32>(-1.0, -1.0, 0.0, 1.0); }
        case 1u: { return vec4<f32>(3.0, -1.0, 0.0, 1.0); }
        default: { return vec4<f32>(-1.0, 3.0, 0.0, 1.0); }
    }
}

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let point = vec2<i32>(position.xy);
    let color = textureLoad(color_texture, point, 0);

    let ssao_value = textureLoad(ssao_texture, point, 0).r;
    let ssao_scale = color.a * f32(settings.is_ssao_enabled);

    return vec4<f32>(color.rgb * mix(1.0, ssao_value, ssao_scale), 1.0);
}
