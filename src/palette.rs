use bevy::core_pipeline::core_3d::graph::Node3d;
use bevy::core_pipeline::fullscreen_material::{FullscreenMaterial, FullscreenMaterialPlugin};
use bevy::prelude::*;
use bevy::render::extract_component::ExtractComponent;
use bevy::render::render_graph::{InternedRenderLabel, RenderLabel};
use bevy::render::render_resource::ShaderType;
use bevy::shader::ShaderRef;

pub struct PalettePlugin;

impl Plugin for PalettePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FullscreenMaterialPlugin::<PaletteQuantize>::default());
    }
}

/// Fullscreen post-process that quantizes colors to a 3-color palette (black, dark grey, white).
/// Add this component to a Camera3d entity to enable the effect.
#[derive(Component, ExtractComponent, Clone, Copy, ShaderType, Default)]
pub struct PaletteQuantize {
    // Placeholder field required by ShaderType (must have at least one field)
    pub _padding: f32,
}

impl FullscreenMaterial for PaletteQuantize {
    fn fragment_shader() -> ShaderRef {
        "shaders/palette_quantize.wgsl".into()
    }

    fn node_edges() -> Vec<InternedRenderLabel> {
        vec![
            Node3d::Tonemapping.intern(),
            Self::node_label().intern(),
            Node3d::EndMainPassPostProcessing.intern(),
        ]
    }
}
