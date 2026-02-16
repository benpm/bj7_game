use crate::GameState;
use crate::aberration::Aberration;
use crate::loading::TextureAssets;
use crate::pause::game_not_paused;
use crate::player::FpsCamera;
use crate::scaling::CANVAS_SCALE;
use bevy::prelude::*;
use bevy::window::{
    CursorGrabMode, CursorIcon, CursorOptions, CustomCursor, CustomCursorImage, PrimaryWindow,
    SystemCursorIcon,
};

pub struct DispelPlugin;

impl Plugin for DispelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), init_dispel)
            .add_systems(
                Update,
                (
                    toggle_dispel,
                    dispel_draw,
                    check_closure_and_dispel,
                    exit_dispel_on_right_click,
                    draw_dispel_gizmos,
                )
                    .chain()
                    .run_if(in_state(GameState::Playing).and(game_not_paused)),
            )
            .add_systems(OnExit(GameState::Playing), cleanup_dispel);
    }
}

/// How long the player must wait between adding points to the current polygon.
const SEGMENT_INTERVAL: f32 = 0.05;
/// Maximum distance from the starting point for the polygon to be considered closed (in viewport pixels).
const CLOSURE_DISTANCE: f32 = 30.0;
/// Minimum number of points required to attempt closure and dispel. This prevents accidental clicks from doing anything.
const MIN_POINTS: usize = 10;
/// How far in front of the camera to place gizmo points (world units).
const GIZMO_DEPTH: f32 = 0.5;
/// Minimum distance between points to add a new one, to prevent over-sampling when the player holds still.
const MIN_POINT_DISTANCE: f32 = 5.0;

#[derive(Resource)]
pub struct DispelState {
    pub active: bool,
    drawing: bool,
    points: Vec<Vec2>,
    segment_timer: Timer,
}

impl Default for DispelState {
    fn default() -> Self {
        Self {
            active: false,
            drawing: false,
            points: Vec::new(),
            segment_timer: Timer::from_seconds(SEGMENT_INTERVAL, TimerMode::Repeating),
        }
    }
}

fn init_dispel(mut commands: Commands) {
    commands.insert_resource(DispelState::default());
}

fn toggle_dispel(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    mut state: ResMut<DispelState>,
    mut cursor_q: Query<&mut CursorOptions, With<PrimaryWindow>>,
    window_q: Query<(Entity, &Window), With<PrimaryWindow>>,
    textures: Res<TextureAssets>,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    if !state.active {
        // Enter dispel mode
        state.active = true;
        state.drawing = false;
        state.points.clear();
        if let Ok(mut cursor) = cursor_q.single_mut() {
            cursor.grab_mode = CursorGrabMode::None;
            cursor.visible = true;
        }
        // Set custom cursor
        if let Ok((window_entity, _)) = window_q.single() {
            commands.entity(window_entity).insert(CursorIcon::Custom(
                CustomCursor::Image(CustomCursorImage {
                    handle: textures.feather_cursor.clone(),
                    texture_atlas: None,
                    flip_x: false,
                    flip_y: false,
                    rect: None,
                    hotspot: (0, 0),
                }),
            ));
        }
    } else if !state.drawing {
        // Start drawing
        state.drawing = true;
        state.points.clear();
        state.segment_timer.reset();
        if let Ok((_, window)) = window_q.single()
            && let Some(pos) = window.cursor_position()
        {
            state.points.push(pos);
        }
    }
}

fn dispel_draw(
    mut state: ResMut<DispelState>,
    mouse: Res<ButtonInput<MouseButton>>,
    time: Res<Time>,
    window_q: Query<&Window, With<PrimaryWindow>>,
) {
    if !state.active || !state.drawing {
        return;
    }

    if mouse.just_released(MouseButton::Left) {
        state.drawing = false;
        state.points.clear();
        return;
    }

    if !mouse.pressed(MouseButton::Left) {
        return;
    }

    state.segment_timer.tick(time.delta());
    if state.segment_timer.just_finished()
        && let Ok(window) = window_q.single()
        && let Some(pos) = window.cursor_position()
        && state.points.last().is_none_or(|last| pos.distance(*last) > MIN_POINT_DISTANCE)
    {
        state.points.push(pos);
    }
}

fn check_closure_and_dispel(
    mut commands: Commands,
    mut state: ResMut<DispelState>,
    window_q: Query<&Window, With<PrimaryWindow>>,
    window_entity_q: Query<Entity, With<PrimaryWindow>>,
    aberration_q: Query<(Entity, &GlobalTransform), With<Aberration>>,
    camera_q: Query<(&Camera, &GlobalTransform), With<FpsCamera>>,
    mut cursor_q: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    if !state.active || !state.drawing || state.points.len() < MIN_POINTS {
        return;
    }

    let Ok(window) = window_q.single() else {
        return;
    };
    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };

    let first = state.points[0];
    if cursor_pos.distance(first) > CLOSURE_DISTANCE {
        return;
    }

    // Loop closed — push final point to complete the polygon
    state.points.push(cursor_pos);

    // Check each aberration against the polygon
    if let Ok((camera, cam_transform)) = camera_q.single() {
        for (entity, ab_transform) in &aberration_q {
            if let Ok(viewport_pos) =
                camera.world_to_viewport(cam_transform, ab_transform.translation())
                && point_in_polygon(viewport_pos * CANVAS_SCALE, &state.points)
            {
                commands.entity(entity).despawn();
            }
        }
    }

    // Exit dispel mode
    deactivate_dispel(&mut commands, &mut state, &mut cursor_q, &window_entity_q);
}

fn exit_dispel_on_right_click(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    mut state: ResMut<DispelState>,
    mut cursor_q: Query<&mut CursorOptions, With<PrimaryWindow>>,
    window_entity_q: Query<Entity, With<PrimaryWindow>>,
) {
    if state.active && mouse.just_pressed(MouseButton::Right) {
        deactivate_dispel(&mut commands, &mut state, &mut cursor_q, &window_entity_q);
    }
}

fn deactivate_dispel(
    commands: &mut Commands,
    state: &mut DispelState,
    cursor_q: &mut Query<&mut CursorOptions, With<PrimaryWindow>>,
    window_q: &Query<Entity, With<PrimaryWindow>>,
) {
    state.active = false;
    state.drawing = false;
    state.points.clear();
    if let Ok(mut cursor) = cursor_q.single_mut() {
        cursor.grab_mode = CursorGrabMode::Locked;
        cursor.visible = false;
    }
    // Restore default cursor
    if let Ok(window_entity) = window_q.single() {
        commands
            .entity(window_entity)
            .insert(CursorIcon::System(SystemCursorIcon::Default));
    }
}

/// Project a window pixel coordinate to a 3D world point at GIZMO_DEPTH in front of the camera.
/// Scales from window coords to half-res viewport coords before projecting.
fn viewport_to_world_point(camera: &Camera, cam_transform: &GlobalTransform, px: Vec2) -> Option<Vec3> {
    let viewport_px = px / CANVAS_SCALE;
    let ray = camera.viewport_to_world(cam_transform, viewport_px).ok()?;
    Some(ray.get_point(GIZMO_DEPTH))
}

fn draw_dispel_gizmos(
    state: Res<DispelState>,
    mut gizmos: Gizmos,
    window_q: Query<&Window, With<PrimaryWindow>>,
    camera_q: Query<(&Camera, &GlobalTransform), With<FpsCamera>>,
) {
    if !state.active || state.points.len() < 2 {
        return;
    }

    let Ok((camera, cam_transform)) = camera_q.single() else {
        return;
    };

    // Convert viewport points to 3D world positions near the camera
    let world_points: Vec<Vec3> = state
        .points
        .iter()
        .filter_map(|px| viewport_to_world_point(camera, cam_transform, *px))
        .collect();

    if world_points.len() >= 2 {
        gizmos.linestrip(world_points.iter().copied(), Color::WHITE);
    }

    // Draw closure target indicator at the start point
    if let Some(start) = viewport_to_world_point(camera, cam_transform, state.points[0]) {
        gizmos.sphere(Isometry3d::from_translation(start), GIZMO_DEPTH * 0.02, Color::WHITE);
    }

    // Draw circle at cursor position
    let Ok(window) = window_q.single() else {
        return;
    };
    if let Some(cursor_pos) = window.cursor_position()
        && let Some(cursor_world) = viewport_to_world_point(camera, cam_transform, cursor_pos)
    {
        gizmos.sphere(
            Isometry3d::from_translation(cursor_world),
            GIZMO_DEPTH * 0.005,
            Color::WHITE,
        );
    }
}

/// Ray-casting point-in-polygon test.
fn point_in_polygon(point: Vec2, polygon: &[Vec2]) -> bool {
    let mut inside = false;
    let n = polygon.len();
    let mut j = n - 1;
    for i in 0..n {
        let pi = polygon[i];
        let pj = polygon[j];
        if ((pi.y > point.y) != (pj.y > point.y))
            && (point.x < (pj.x - pi.x) * (point.y - pi.y) / (pj.y - pi.y) + pi.x)
        {
            inside = !inside;
        }
        j = i;
    }
    inside
}

fn cleanup_dispel(mut commands: Commands) {
    commands.remove_resource::<DispelState>();
}
