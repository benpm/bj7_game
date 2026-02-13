#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;

struct PaletteQuantize {
    num_colors: f32,
}
@group(0) @binding(2) var<uniform> settings: PaletteQuantize;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(screen_texture, texture_sampler, in.uv);

    // Posterization: reduce each color channel to discrete levels
    let levels = settings.num_colors - 1.0;
    let posterized = round(color.rgb * levels) / levels;

    return vec4<f32>(posterized, color.a);
}
