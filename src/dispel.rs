use crate::GameState;
use crate::aberration::Aberration;
use crate::pause::game_not_paused;
use crate::player::FpsCamera;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};

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

const SEGMENT_INTERVAL: f32 = 0.05;
const CLOSURE_DISTANCE: f32 = 30.0;
const MIN_POINTS: usize = 10;

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

#[derive(Component)]
struct DispelCamera;

fn init_dispel(mut commands: Commands) {
    commands.insert_resource(DispelState::default());
}

fn toggle_dispel(
    mouse: Res<ButtonInput<MouseButton>>,
    mut state: ResMut<DispelState>,
    mut commands: Commands,
    mut cursor_q: Query<&mut CursorOptions, With<PrimaryWindow>>,
    window_q: Query<&Window, With<PrimaryWindow>>,
    camera_q: Query<Entity, With<DispelCamera>>,
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
        // Spawn overlay Camera2d for gizmo rendering
        if camera_q.is_empty() {
            commands.spawn((
                Camera2d,
                Camera {
                    order: 2,
                    clear_color: ClearColorConfig::None,
                    ..default()
                },
                Msaa::Off,
                DispelCamera,
            ));
        }
    } else if !state.drawing {
        // Start drawing
        state.drawing = true;
        state.points.clear();
        state.segment_timer.reset();
        if let Ok(window) = window_q.single()
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
    {
        state.points.push(pos);
    }
}

fn check_closure_and_dispel(
    mut commands: Commands,
    mut state: ResMut<DispelState>,
    window_q: Query<&Window, With<PrimaryWindow>>,
    aberration_q: Query<(Entity, &GlobalTransform), With<Aberration>>,
    camera_q: Query<(&Camera, &GlobalTransform), With<FpsCamera>>,
    dispel_cam_q: Query<Entity, With<DispelCamera>>,
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
                && point_in_polygon(viewport_pos, &state.points)
            {
                commands.entity(entity).despawn();
            }
        }
    }

    // Exit dispel mode
    deactivate_dispel(&mut state, &mut commands, &mut cursor_q, &dispel_cam_q);
}

fn exit_dispel_on_right_click(
    mouse: Res<ButtonInput<MouseButton>>,
    mut state: ResMut<DispelState>,
    mut commands: Commands,
    mut cursor_q: Query<&mut CursorOptions, With<PrimaryWindow>>,
    dispel_cam_q: Query<Entity, With<DispelCamera>>,
) {
    if state.active && mouse.just_pressed(MouseButton::Right) {
        deactivate_dispel(&mut state, &mut commands, &mut cursor_q, &dispel_cam_q);
    }
}

fn deactivate_dispel(
    state: &mut DispelState,
    commands: &mut Commands,
    cursor_q: &mut Query<&mut CursorOptions, With<PrimaryWindow>>,
    dispel_cam_q: &Query<Entity, With<DispelCamera>>,
) {
    state.active = false;
    state.drawing = false;
    state.points.clear();
    if let Ok(mut cursor) = cursor_q.single_mut() {
        cursor.grab_mode = CursorGrabMode::Locked;
        cursor.visible = false;
    }
    for entity in dispel_cam_q.iter() {
        commands.entity(entity).despawn();
    }
}

fn draw_dispel_gizmos(
    state: Res<DispelState>,
    mut gizmos: Gizmos,
    window_q: Query<&Window, With<PrimaryWindow>>,
) {
    if !state.active || state.points.len() < 2 {
        return;
    }

    let Ok(window) = window_q.single() else {
        return;
    };
    let half_w = window.width() / 2.0;
    let half_h = window.height() / 2.0;

    let to_gizmo = |px: Vec2| -> Vec2 { Vec2::new(px.x - half_w, half_h - px.y) };

    let converted: Vec<Vec2> = state.points.iter().map(|p| to_gizmo(*p)).collect();
    gizmos.linestrip_2d(converted.iter().copied(), Color::WHITE);

    // Draw closure target indicator at the start point
    let start = to_gizmo(state.points[0]);
    gizmos.circle_2d(
        Isometry2d::from_translation(start),
        CLOSURE_DISTANCE,
        Color::srgba(1.0, 1.0, 1.0, 0.4),
    );

    // Draw current cursor position
    if let Some(cursor_pos) = window.cursor_position() {
        let cursor_gizmo = to_gizmo(cursor_pos);
        gizmos.circle_2d(
            Isometry2d::from_translation(cursor_gizmo),
            4.0,
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

fn cleanup_dispel(mut commands: Commands, dispel_cam_q: Query<Entity, With<DispelCamera>>) {
    commands.remove_resource::<DispelState>();
    for entity in &dispel_cam_q {
        commands.entity(entity).despawn();
    }
}
