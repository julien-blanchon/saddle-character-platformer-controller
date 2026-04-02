#[cfg(feature = "e2e")]
mod e2e;
#[cfg(feature = "e2e")]
mod scenarios;
use saddle_character_platformer_controller_example_support as support;

use bevy::prelude::*;
use saddle_character_platformer_controller::{
    AirJumpConsumed, JumpStarted, Landed, PlatformerControllerDebugPlugin,
    PlatformerControllerDebugSettings, PlatformerControllerSystems, PlatformerJumpKind,
    PlatformerMovementIntent, PlatformerWallSide, WallJumpStarted,
};
use support::{DemoFixedSystems, DemoPlayer, DemoScene};

const LAB_BRP_PORT: u16 = 15_732;

#[derive(Resource, Reflect, Default, Debug, Clone, Copy)]
#[reflect(Resource, Debug, Default)]
pub struct ScriptedControl {
    pub active: bool,
    pub move_axis: f32,
    pub jump_held: bool,
    pub jump_pressed: bool,
    pub drop_pressed: bool,
}

#[derive(Resource, Reflect, Default, Debug, Clone)]
#[reflect(Resource, Debug, Default)]
pub struct LabMessageLog {
    pub jump_count: u32,
    pub last_jump_kind: Option<PlatformerJumpKind>,
    pub last_jump_used_buffer: bool,
    pub last_jump_velocity: Vec2,
    pub wall_jump_count: u32,
    pub last_wall_jump_side: Option<PlatformerWallSide>,
    pub last_wall_jump_velocity: Vec2,
    pub landed_count: u32,
    pub last_landed_support: Option<Entity>,
    pub last_impact_speed: f32,
    pub air_jump_consumed_count: u32,
    pub last_remaining_air_jumps: Option<u32>,
}

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum LabFixedSystems {
    ScriptedIntent,
    ObserveMessages,
}

fn main() {
    let mut app = App::new();
    let scene = requested_scene();
    let debug_enabled = std::env::var("PLATFORMER_CONTROLLER_DEBUG")
        .is_ok_and(|value| value == "1" || value.eq_ignore_ascii_case("true"));

    support::configure_demo_app(&mut app, scene, false);
    app.init_resource::<ScriptedControl>();
    app.init_resource::<LabMessageLog>();
    app.register_type::<ScriptedControl>()
        .register_type::<LabMessageLog>();
    app.insert_resource(PlatformerControllerDebugSettings {
        enabled: debug_enabled,
        ..default()
    });
    app.add_plugins(PlatformerControllerDebugPlugin);
    #[cfg(all(feature = "dev", not(target_arch = "wasm32")))]
    app.add_plugins(bevy_brp_extras::BrpExtrasPlugin::with_port(LAB_BRP_PORT));
    #[cfg(feature = "e2e")]
    app.add_plugins(e2e::PlatformerControllerLabE2EPlugin);

    app.configure_sets(
        FixedUpdate,
        LabFixedSystems::ScriptedIntent
            .after(DemoFixedSystems::DriveIntent)
            .before(PlatformerControllerSystems::ReadIntent),
    );
    app.configure_sets(
        FixedUpdate,
        LabFixedSystems::ObserveMessages.after(PlatformerControllerSystems::SyncState),
    );
    app.add_systems(
        FixedUpdate,
        (
            support::drive_keyboard_intent.in_set(DemoFixedSystems::DriveIntent),
            apply_scripted_control.in_set(LabFixedSystems::ScriptedIntent),
            record_jump_messages.in_set(LabFixedSystems::ObserveMessages),
            record_wall_jump_messages.in_set(LabFixedSystems::ObserveMessages),
            record_landed_messages.in_set(LabFixedSystems::ObserveMessages),
            record_air_jump_messages.in_set(LabFixedSystems::ObserveMessages),
        ),
    );

    app.run();
}

fn requested_scene() -> DemoScene {
    if let Ok(scene) = std::env::var("PLATFORMER_CONTROLLER_LAB_SCENE") {
        return parse_scene_name(&scene).unwrap_or(DemoScene::Basic);
    }

    for arg in std::env::args().skip(1) {
        if arg.starts_with('-') {
            continue;
        }
        if let Some(scene) = parse_scene_name(&arg) {
            return scene;
        }
    }

    DemoScene::Basic
}

fn parse_scene_name(value: &str) -> Option<DemoScene> {
    let value = value.to_ascii_lowercase();
    if value.contains("wall") {
        Some(DemoScene::WallJumps)
    } else if value.contains("moving_platform") || value.contains("platforms") {
        Some(DemoScene::MovingPlatforms)
    } else if value.contains("one_way") || value.contains("drop_through") {
        Some(DemoScene::OneWayPlatforms)
    } else if value.contains("basic")
        || value.contains("smoke")
        || value.contains("coyote")
        || value.contains("buffer")
    {
        Some(DemoScene::Basic)
    } else {
        None
    }
}

fn apply_scripted_control(
    mut scripted: ResMut<ScriptedControl>,
    mut intent: Single<&mut PlatformerMovementIntent, With<DemoPlayer>>,
) {
    if !scripted.active {
        return;
    }

    intent.move_axis = scripted.move_axis;
    intent.jump_held = scripted.jump_held;

    if scripted.jump_pressed {
        intent.jump_pressed = true;
        scripted.jump_pressed = false;
    }

    if scripted.drop_pressed {
        intent.drop_pressed = true;
        scripted.drop_pressed = false;
    }
}

fn record_jump_messages(mut log: ResMut<LabMessageLog>, mut messages: MessageReader<JumpStarted>) {
    for message in messages.read() {
        log.jump_count += 1;
        log.last_jump_kind = Some(message.kind);
        log.last_jump_used_buffer = message.used_buffer;
        log.last_jump_velocity = message.velocity;
    }
}

fn record_wall_jump_messages(
    mut log: ResMut<LabMessageLog>,
    mut messages: MessageReader<WallJumpStarted>,
) {
    for message in messages.read() {
        log.wall_jump_count += 1;
        log.last_wall_jump_side = Some(message.side);
        log.last_wall_jump_velocity = message.velocity;
    }
}

fn record_landed_messages(mut log: ResMut<LabMessageLog>, mut messages: MessageReader<Landed>) {
    for message in messages.read() {
        log.landed_count += 1;
        log.last_landed_support = message.support_entity;
        log.last_impact_speed = message.impact_speed;
    }
}

fn record_air_jump_messages(
    mut log: ResMut<LabMessageLog>,
    mut messages: MessageReader<AirJumpConsumed>,
) {
    for message in messages.read() {
        log.air_jump_consumed_count += 1;
        log.last_remaining_air_jumps = Some(message.remaining_air_jumps);
    }
}
