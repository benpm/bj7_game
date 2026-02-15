#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

const DITHER: bool = true;
const DOWN_SCALE: f32 = 2.0;
const PALETTE_SIZE: i32 = 3;
const HIGHLIGHT_SIZE: i32 = 5;
const HIGHLIGHT_THRESHOLD: f32 = 0.15;

struct PaletteSqueeze {
    resolution: vec3f,
    time: f32,
    darken: f32,
}

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;
@group(0) @binding(2) var<uniform> u: PaletteSqueeze;
@group(0) @binding(3) var noise_tex: texture_2d<f32>;
@group(0) @binding(4) var noise_samp: sampler;

const palette = array<vec3f, 3>(
    vec3f(0.000, 0.000, 0.000),
    vec3f(0.278, 0.278, 0.278),
    vec3f(1.000, 1.000, 1.000),
);

const highlights = array<vec3f, 5>(
    vec3f(1.000, 0.000, 0.929),
    vec3f(1.000, 1.000, 0.000),
    vec3f(0.196, 1.000, 0.000),
    vec3f(0.000, 0.906, 1.000),
    vec3f(1.000, 0.263, 0.235),
);

// Get a color from the palette with optional dithering between entries
// - x: expected to be in [0, 1] range, representing position between palette entries
// - pixel: used for dithering noise lookup
fn get_dithered_palette(x: f32, pixel: vec2f) -> vec3f {
    let idx = clamp(x, 0.0, 1.0) * f32(PALETTE_SIZE - 1);
    let i = i32(idx);
    let i_next = min(i + 1, PALETTE_SIZE - 1);

    let c1 = palette[i];
    let c2 = palette[i_next];

    var mix_amt: f32;
    if DITHER {
        let noise_dims = vec2f(textureDimensions(noise_tex));
        let dith = textureSample(noise_tex, noise_samp, pixel / noise_dims).r;
        mix_amt = select(0.0, 1.0, fract(idx) > dith);
    } else {
        mix_amt = fract(idx);
    }

    return mix(c1, c2, mix_amt);
}

fn check_highlight(color: vec3f) -> vec4f {
    for (var i = 0; i < HIGHLIGHT_SIZE; i++) {
        let diff = length(color - highlights[i]);
        if diff < HIGHLIGHT_THRESHOLD {
            return vec4f(highlights[i], 1.0);
        }
    }
    return vec4f(0.0);
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4f {
    let fc = floor(in.position.xy / DOWN_SCALE) * DOWN_SCALE;
    let pixel = fc / DOWN_SCALE;

    let screen_color = textureSample(screen_texture, texture_sampler, in.uv).rgb;
    let incolor = mix(screen_color, vec3f(0.0), u.darken);

    // Check if input color is near any highlight — pass through directly
    let highlight = check_highlight(incolor);
    if highlight.a > 0.5 {
        return highlight;
    }

    // Otherwise apply grayscale palette quantization
    let luminance = dot(incolor, vec3f(0.2126, 0.7152, 0.0722));
    let darkened = luminance * (1.0 - u.darken);

    let color = get_dithered_palette(darkened, pixel);
    return vec4f(color, 1.0);
}
