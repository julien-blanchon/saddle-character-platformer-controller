use avian2d::prelude::{Collider, LinearVelocity};
use bevy::prelude::*;

use saddle_character_platformer_controller::{
    PlatformerControllerBundle, PlatformerControllerConfig, PlatformerControllerState,
    PlatformerMotionPhase, PlatformerMovementIntent,
};

use crate::support::DemoDiagnostics;
use crate::{DemoPlayer, LabMessageLog, ScriptedControl};

pub type DiagnosticsResource = DemoDiagnostics;

pub fn player_entity(world: &World) -> Option<Entity> {
    world
        .get_resource::<DemoDiagnostics>()
        .and_then(|diagnostics| diagnostics.player_entity)
}

pub fn player_state(world: &World) -> Option<PlatformerControllerState> {
    world
        .get_resource::<DemoDiagnostics>()
        .and_then(|diagnostics| diagnostics.controller_state.clone())
}

pub fn overlay_text(world: &World) -> Option<String> {
    world
        .get_resource::<DemoDiagnostics>()
        .and_then(|diagnostics| {
            (!diagnostics.overlay_text.is_empty()).then_some(diagnostics.overlay_text.clone())
        })
}

pub fn teleport_player(world: &mut World, translation: Vec2, velocity: Vec2) {
    let scene = world.resource::<crate::support::DemoState>().scene;
    teleport_player_with_config(world, translation, velocity, scene.controller_config());
}

pub fn teleport_player_with_config(
    world: &mut World,
    translation: Vec2,
    velocity: Vec2,
    config: PlatformerControllerConfig,
) {
    if let Some(entity) = player_entity(world) {
        let _ = world.despawn(entity);
    }

    let entity = world
        .spawn((
            Name::new("Demo Player"),
            DemoPlayer,
            Sprite {
                color: Color::srgb(0.94, 0.58, 0.22),
                custom_size: Some(crate::support::PLAYER_SIZE),
                ..default()
            },
            PlatformerControllerBundle::with_config(
                Collider::rectangle(crate::support::PLAYER_SIZE.x, crate::support::PLAYER_SIZE.y),
                config.clone(),
            )
            .with_transform(Transform::from_xyz(translation.x, translation.y, 10.0)),
        ))
        .id();
    world
        .get_mut::<LinearVelocity>(entity)
        .expect("fresh demo player should have LinearVelocity")
        .0 = velocity;
    world
        .get_mut::<PlatformerMovementIntent>(entity)
        .expect("fresh demo player should have PlatformerMovementIntent")
        .clone_from(&PlatformerMovementIntent::default());

    if let Some(mut diagnostics) = world.get_resource_mut::<DemoDiagnostics>() {
        diagnostics.player_entity = Some(entity);
        diagnostics.player_position = translation;
        diagnostics.player_velocity = velocity;
        diagnostics.grounded = false;
        diagnostics.phase = PlatformerMotionPhase::Airborne;
        diagnostics.remaining_air_jumps = config.jump.max_air_jumps;
        diagnostics.buffered_jump = false;
        diagnostics.wall_side = None;
        diagnostics.controller_state = None;
        diagnostics.overlay_text.clear();
    }
    if let Some(mut scripted) = world.get_resource_mut::<ScriptedControl>() {
        *scripted = ScriptedControl::default();
    }
    if let Some(mut log) = world.get_resource_mut::<LabMessageLog>() {
        *log = LabMessageLog::default();
    }
}

pub fn set_scripted_control(
    world: &mut World,
    move_axis: f32,
    jump_held: bool,
    pulse_jump: bool,
    pulse_dash: bool,
    pulse_drop: bool,
) {
    let mut control = world.resource_mut::<ScriptedControl>();
    control.active = true;
    control.move_axis = move_axis;
    control.jump_held = jump_held;
    control.jump_pressed |= pulse_jump;
    control.dash_pressed |= pulse_dash;
    control.drop_pressed |= pulse_drop;
}
