#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;

struct PaletteQuantize {
    _padding: f32,
}
@group(0) @binding(2) var<uniform> settings: PaletteQuantize;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(screen_texture, texture_sampler, in.uv);

    // Convert to perceived luminance
    let luminance = dot(color.rgb, vec3<f32>(0.299, 0.587, 0.114));

    // Quantize to 3 levels: black, dark grey, white
    var out_color: f32;
    if luminance < 0.33 {
        out_color = 0.0;        // black
    } else if luminance < 0.66 {
        out_color = 0.25;       // dark grey
    } else {
        out_color = 1.0;        // white
    }

    return vec4<f32>(out_color, out_color, out_color, 1.0);
}
