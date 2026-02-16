#import bevy_pbr::{
    mesh_view_bindings::globals,
    forward_io::VertexOutput,
}

@group(3) @binding(0) var base_texture: texture_2d<f32>;
@group(3) @binding(1) var base_sampler: sampler;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let time = globals.time;
    let strength = 0.03;
    let speed = 3.0;
    let freq = 8.0;

    var uv = mesh.uv;
    uv.x += sin(uv.y * freq + time * speed) * strength;
    uv.y += sin(uv.x * freq * 1.3 + time * speed * 0.7) * strength;

    let color = textureSample(base_texture, base_sampler, uv);

    if color.a < 0.5 {
        discard;
    }

    return color;
}
