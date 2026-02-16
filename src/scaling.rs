use bevy::prelude::*;
use bevy::render::render_resource::TextureFormat;
use bevy::window::PrimaryWindow;

pub struct ScalingPlugin;

impl Plugin for ScalingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_canvas)
            .add_systems(Update, resize_canvas);
    }
}

/// The scale factor: game renders at 1/CANVAS_SCALE of window resolution.
pub const CANVAS_SCALE: f32 = 2.0;

/// Handle to the half-resolution render target image.
#[derive(Resource)]
pub struct CanvasImage(pub Handle<Image>);

#[derive(Component)]
struct UpscaleCamera;

#[derive(Component)]
struct UpscaleSprite;

fn setup_canvas(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    window_q: Query<&Window, With<PrimaryWindow>>,
) {
    let Ok(window) = window_q.single() else {
        return;
    };

    let w = (window.physical_width() as f32 / CANVAS_SCALE) as u32;
    let h = (window.physical_height() as f32 / CANVAS_SCALE) as u32;

    let canvas = Image::new_target_texture(w.max(1), h.max(1), TextureFormat::Rgba8UnormSrgb, None);
    let canvas_handle = images.add(canvas);

    commands.insert_resource(CanvasImage(canvas_handle.clone()));

    // Upscale camera: renders to the window, displays the canvas image fullscreen
    commands.spawn((
        Camera2d,
        Camera {
            order: 10,
            ..default()
        },
        Msaa::Off,
        UpscaleCamera,
    ));

    // Fullscreen sprite showing the half-res canvas
    commands.spawn((
        Sprite {
            image: canvas_handle,
            custom_size: Some(Vec2::new(window.width(), window.height())),
            ..default()
        },
        UpscaleSprite,
    ));
}

fn resize_canvas(
    window_q: Query<&Window, (With<PrimaryWindow>, Changed<Window>)>,
    canvas: Option<Res<CanvasImage>>,
    mut images: ResMut<Assets<Image>>,
    mut sprite_q: Query<&mut Sprite, With<UpscaleSprite>>,
) {
    let Ok(window) = window_q.single() else {
        return;
    };
    let Some(canvas) = canvas else {
        return;
    };

    let w = (window.physical_width() as f32 / CANVAS_SCALE) as u32;
    let h = (window.physical_height() as f32 / CANVAS_SCALE) as u32;

    if let Some(image) = images.get_mut(&canvas.0) {
        image.resize(bevy::render::render_resource::Extent3d {
            width: w.max(1),
            height: h.max(1),
            depth_or_array_layers: 1,
        });
    }

    for mut sprite in &mut sprite_q {
        sprite.custom_size = Some(Vec2::new(window.width(), window.height()));
    }
}
