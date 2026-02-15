use bevy::core_pipeline::FullscreenShader;
use bevy::core_pipeline::core_3d::graph::{Core3d, Node3d};
use bevy::ui_render::graph::NodeUi;
use bevy::ecs::query::QueryItem;
use bevy::image::BevyDefault;
use bevy::prelude::*;
use bevy::render::extract_component::{
    ComponentUniforms, DynamicUniformIndex, ExtractComponent, ExtractComponentPlugin,
    UniformComponentPlugin,
};
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_graph::{
    NodeRunError, RenderGraphContext, RenderGraphExt, RenderLabel, ViewNode, ViewNodeRunner,
};
use bevy::render::render_resource::{
    AddressMode, BindGroupEntries, BindGroupLayoutDescriptor, BindGroupLayoutEntries,
    CachedRenderPipelineId, ColorTargetState, ColorWrites, FilterMode, FragmentState, Operations,
    PipelineCache, RenderPassColorAttachment, RenderPassDescriptor, RenderPipelineDescriptor,
    Sampler, SamplerBindingType, SamplerDescriptor, ShaderStages, ShaderType, TextureFormat,
    TextureSampleType,
    binding_types::{sampler, texture_2d, uniform_buffer},
};
use bevy::render::renderer::{RenderContext, RenderDevice};
use bevy::render::texture::GpuImage;
use bevy::render::view::ViewTarget;
use bevy::render::{RenderApp, RenderStartup};
use bevy::window::PrimaryWindow;

pub struct PalettePlugin;

impl Plugin for PalettePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ExtractComponentPlugin::<PaletteSqueeze>::default(),
            UniformComponentPlugin::<PaletteSqueeze>::default(),
        ))
        .add_systems(Update, update_palette_squeeze);

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .add_systems(RenderStartup, init_palette_pipeline)
            .add_render_graph_node::<ViewNodeRunner<PaletteSqueezeNode>>(
                Core3d,
                PaletteSqueezeLabel,
            )
            .add_render_graph_edges(
                Core3d,
                (
                    NodeUi::UiPass,
                    PaletteSqueezeLabel,
                    Node3d::Upscaling,
                ),
            );
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct PaletteSqueezeLabel;

/// Fullscreen post-process that applies palette quantization with blue-noise dithering.
/// Add this component to a Camera3d entity to enable the effect.
#[derive(Component, ExtractComponent, Clone, Copy, ShaderType)]
pub struct PaletteSqueeze {
    pub resolution: Vec3,
    pub time: f32,
    /// 0.0 = normal, 1.0 = fully dark. Used for environment transitions.
    pub darken: f32,
}

impl Default for PaletteSqueeze {
    fn default() -> Self {
        Self {
            resolution: Vec3::new(1280.0, 720.0, 0.0),
            time: 0.0,
            darken: 0.0,
        }
    }
}

/// Resource that other systems can write to control the palette darken effect.
#[derive(Resource, Default)]
pub struct PaletteDarken {
    pub value: f32,
}

fn update_palette_squeeze(
    time: Res<Time>,
    window_q: Query<&Window, With<PrimaryWindow>>,
    darken: Option<Res<PaletteDarken>>,
    mut squeeze_q: Query<&mut PaletteSqueeze>,
) {
    let Ok(window) = window_q.single() else {
        return;
    };
    let resolution = Vec3::new(window.width(), window.height(), 0.0);
    let elapsed = time.elapsed_secs();
    let _darken_val = darken.map_or(0.0, |d| d.value);

    for mut squeeze in &mut squeeze_q {
        squeeze.resolution = resolution;
        squeeze.time = elapsed;
        squeeze.darken = 0.0;
    }
}

#[derive(Resource)]
struct PaletteSqueezePipeline {
    layout: BindGroupLayoutDescriptor,
    sampler: Sampler,
    noise_sampler: Sampler,
    noise_image: Handle<Image>,
    pipeline_id: CachedRenderPipelineId,
    pipeline_id_hdr: CachedRenderPipelineId,
}

fn init_palette_pipeline(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    asset_server: Res<AssetServer>,
    fullscreen_shader: Res<FullscreenShader>,
    pipeline_cache: Res<PipelineCache>,
) {
    let layout = BindGroupLayoutDescriptor::new(
        "palette_squeeze_bind_group_layout",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::FRAGMENT,
            (
                // binding 0: screen texture
                texture_2d(TextureSampleType::Float { filterable: true }),
                // binding 1: screen sampler
                sampler(SamplerBindingType::Filtering),
                // binding 2: uniform buffer
                uniform_buffer::<PaletteSqueeze>(true),
                // binding 3: noise texture
                texture_2d(TextureSampleType::Float { filterable: true }),
                // binding 4: noise sampler
                sampler(SamplerBindingType::Filtering),
            ),
        ),
    );

    let sampler = render_device.create_sampler(&SamplerDescriptor::default());
    let noise_sampler = render_device.create_sampler(&SamplerDescriptor {
        address_mode_u: AddressMode::Repeat,
        address_mode_v: AddressMode::Repeat,
        mag_filter: FilterMode::Nearest,
        min_filter: FilterMode::Nearest,
        ..default()
    });

    let noise_image: Handle<Image> = asset_server.load("textures/blue_noise_rgba.png");
    let shader = asset_server.load("shaders/palette_squeeze.wgsl");

    let vertex_state = fullscreen_shader.to_vertex_state();
    let mut desc = RenderPipelineDescriptor {
        label: Some("palette_squeeze_pipeline".into()),
        layout: vec![layout.clone()],
        vertex: vertex_state,
        fragment: Some(FragmentState {
            shader,
            targets: vec![Some(ColorTargetState {
                format: TextureFormat::bevy_default(),
                blend: None,
                write_mask: ColorWrites::ALL,
            })],
            ..default()
        }),
        ..default()
    };

    let pipeline_id = pipeline_cache.queue_render_pipeline(desc.clone());
    desc.fragment.as_mut().unwrap().targets[0]
        .as_mut()
        .unwrap()
        .format = ViewTarget::TEXTURE_FORMAT_HDR;
    let pipeline_id_hdr = pipeline_cache.queue_render_pipeline(desc);

    commands.insert_resource(PaletteSqueezePipeline {
        layout,
        sampler,
        noise_sampler,
        noise_image,
        pipeline_id,
        pipeline_id_hdr,
    });
}

#[derive(Default)]
struct PaletteSqueezeNode;

impl ViewNode for PaletteSqueezeNode {
    type ViewQuery = (
        &'static ViewTarget,
        &'static DynamicUniformIndex<PaletteSqueeze>,
    );

    fn run<'w>(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext<'w>,
        (view_target, settings_index): QueryItem<'w, '_, Self::ViewQuery>,
        world: &'w World,
    ) -> Result<(), NodeRunError> {
        let pipeline_res = world.resource::<PaletteSqueezePipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();
        let gpu_images = world.resource::<RenderAssets<GpuImage>>();

        let pipeline_id = if view_target.is_hdr() {
            pipeline_res.pipeline_id_hdr
        } else {
            pipeline_res.pipeline_id
        };

        let Some(pipeline) = pipeline_cache.get_render_pipeline(pipeline_id) else {
            return Ok(());
        };

        let settings_uniforms = world.resource::<ComponentUniforms<PaletteSqueeze>>();
        let Some(settings_binding) = settings_uniforms.uniforms().binding() else {
            return Ok(());
        };

        let Some(noise_gpu) = gpu_images.get(&pipeline_res.noise_image) else {
            return Ok(());
        };

        let post_process = view_target.post_process_write();

        let bind_group = render_context.render_device().create_bind_group(
            "palette_squeeze_bind_group",
            &pipeline_cache.get_bind_group_layout(&pipeline_res.layout),
            &BindGroupEntries::sequential((
                post_process.source,
                &pipeline_res.sampler,
                settings_binding.clone(),
                &noise_gpu.texture_view,
                &pipeline_res.noise_sampler,
            )),
        );

        let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("palette_squeeze_pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: post_process.destination,
                depth_slice: None,
                resolve_target: None,
                ops: Operations::default(),
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_render_pipeline(pipeline);
        render_pass.set_bind_group(0, &bind_group, &[settings_index.index()]);
        render_pass.draw(0..3, 0..1);

        Ok(())
    }
}
