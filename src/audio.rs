use crate::GameState;
use crate::actions::Actions;
use crate::actor::Actor;
use crate::loading::AudioAssets;
use crate::pause::game_not_paused;
use crate::player::Player;
use bevy::prelude::*;
use bevy_kira_audio::{Audio, AudioControl, AudioInstance, AudioPlugin, AudioTween};

pub struct GameAudioPlugin;

impl Plugin for GameAudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(AudioPlugin)
            .insert_resource(GameVolume(0.2))
            .add_systems(OnEnter(GameState::Playing), start_footsteps)
            .add_systems(
                Update,
                manage_footsteps.run_if(in_state(GameState::Playing).and(game_not_paused)),
            )
            .add_systems(Update, sync_volume)
            .add_systems(OnExit(GameState::Playing), cleanup_audio);
    }
}

/// Global volume level (0.0–1.0 linear). Persists across states.
#[derive(Resource)]
pub struct GameVolume(pub f32);

/// Convert linear volume (0.0–1.0) to decibels for bevy_kira_audio.
fn volume_db(vol: f32) -> f32 {
    if vol <= 0.0 {
        -80.0
    } else {
        20.0 * vol.log10()
    }
}

#[derive(Resource)]
struct FootstepLoop {
    handle: Handle<AudioInstance>,
    playing: bool,
}

fn start_footsteps(mut commands: Commands, audio: Res<Audio>, assets: Res<AudioAssets>, vol: Res<GameVolume>) {
    let handle = audio
        .play(assets.footsteps.clone())
        .looped()
        .paused()
        .with_volume(volume_db(vol.0))
        .handle();
    commands.insert_resource(FootstepLoop {
        handle,
        playing: false,
    });
}

fn manage_footsteps(
    actions: Res<Actions>,
    player_q: Query<&Actor, With<Player>>,
    mut footstep: ResMut<FootstepLoop>,
    mut audio_instances: ResMut<Assets<AudioInstance>>,
) {
    let should_play = actions.player_movement.is_some()
        && player_q.single().is_ok_and(|a| a.grounded);

    if should_play == footstep.playing {
        return;
    }

    if let Some(instance) = audio_instances.get_mut(&footstep.handle) {
        if should_play {
            instance.resume(AudioTween::default());
        } else {
            instance.pause(AudioTween::default());
        }
        footstep.playing = should_play;
    }
}

fn sync_volume(vol: Res<GameVolume>, audio: Res<Audio>) {
    if vol.is_changed() {
        audio.set_volume(volume_db(vol.0));
    }
}

fn cleanup_audio(
    mut commands: Commands,
    footstep: Option<Res<FootstepLoop>>,
    mut audio_instances: ResMut<Assets<AudioInstance>>,
) {
    if let Some(footstep) = footstep {
        if let Some(instance) = audio_instances.get_mut(&footstep.handle) {
            instance.stop(AudioTween::default());
        }
        commands.remove_resource::<FootstepLoop>();
    }
}
