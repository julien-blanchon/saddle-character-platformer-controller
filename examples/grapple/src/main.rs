//! Platformer controller — grapple hook example
//!
//! Demonstrates rope-swing physics: fire a grapple at marked anchor points,
//! swing on the rope, retract/extend it, and detach with a momentum boost.
//! Grapple points are shown as green circles. The player turns green while
//! grappling.
//!
//! Controls:
//!   A / D or Left/Right — move
//!   Space               — jump (hold for higher) / detach from grapple
//!   E                   — fire grapple toward nearest anchor
//!   R                   — release grapple
//!   W / Up              — retract rope (pull closer)
//!   S / Down            — extend rope (swing wider)

use avian2d::prelude::*;
use bevy::{app::AppExit, camera::ScalingMode, prelude::*, window::WindowResolution};
use saddle_character_platformer_controller::{
    PlatformerControllerBundle, PlatformerControllerConfig, PlatformerControllerPlugin,
    PlatformerControllerState, PlatformerControllerSystems, PlatformerGrappleBundle,
    PlatformerGrappleConfig, PlatformerGrappleIntent, PlatformerGrapplePhase,
    PlatformerGrapplePlugin, PlatformerGrapplePoint, PlatformerGrappleState,
    PlatformerMotionPhase, PlatformerMovementIntent,
};
use saddle_pane::prelude::*;

const PLAYER_SIZE: Vec2 = Vec2::new(18.0, 30.0);

#[derive(Component)]
struct Player;

#[derive(Component)]
struct FollowCamera {
    smoothing: f32,
}

#[derive(Resource, Debug, Clone, PartialEq, Pane)]
#[pane(title = "Grapple", position = "top-right")]
struct GrapplePane {
    #[pane(slider, min = 100.0, max = 600.0, step = 10.0)]
    max_range: f32,
    #[pane(slider, min = 0.0, max = 800.0, step = 10.0)]
    pull_speed: f32,
    #[pane(slider, min = 1.0, max = 2.0, step = 0.05)]
    detach_boost: f32,
    #[pane(slider, min = 50.0, max = 500.0, step = 10.0)]
    retract_speed: f32,
    #[pane(slider, min = 0.0, max = 600.0, step = 10.0)]
    swing_input_force: f32,
    #[pane(monitor)]
    phase: String,
    #[pane(monitor)]
    rope_length: f32,
}

impl Default for GrapplePane {
    fn default() -> Self {
        let g = PlatformerGrappleConfig::default();
        Self {
            max_range: g.max_range,
            pull_speed: g.pull_speed,
            detach_boost: g.detach_speed_boost,
            retract_speed: g.retract_speed,
            swing_input_force: g.swing_input_force,
            phase: "Idle".into(),
            rope_length: 0.0,
        }
    }
}

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum DemoSystems {
    DriveIntent,
}

fn main() -> AppExit {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.06, 0.07, 0.09)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "platformer_controller / grapple".into(),
                resolution: WindowResolution::new(1440, 900),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(Time::<Fixed>::from_hz(60.0))
        .add_plugins(PhysicsPlugins::default().with_length_unit(20.0))
        .add_plugins((
            PlatformerControllerPlugin::always_on(FixedUpdate),
            PlatformerGrapplePlugin::always_on(FixedUpdate),
        ))
        .add_plugins((
            bevy_flair::FlairPlugin,
            bevy_input_focus::InputDispatchPlugin,
            bevy_ui_widgets::UiWidgetsPlugins,
            bevy_input_focus::tab_navigation::TabNavigationPlugin,
            PanePlugin,
        ))
        .register_pane::<GrapplePane>()
        .configure_sets(
            FixedUpdate,
            DemoSystems::DriveIntent.before(PlatformerControllerSystems::ReadIntent),
        )
        .add_systems(Startup, setup_scene)
        .add_systems(
            FixedUpdate,
            drive_keyboard_intent.in_set(DemoSystems::DriveIntent),
        )
        .add_systems(Update, (sync_pane_to_config, update_pane_monitors).chain())
        .add_systems(PostUpdate, (follow_camera, tint_player, draw_grapple_rope))
        .run()
}

fn setup_scene(mut commands: Commands) {
    commands.spawn((
        Name::new("Camera"),
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: 600.0,
            },
            ..OrthographicProjection::default_2d()
        }),
        Transform::from_xyz(0.0, 0.0, 999.0),
        FollowCamera { smoothing: 6.0 },
    ));

    // Player
    let config = PlatformerControllerConfig {
        jump: saddle_character_platformer_controller::PlatformerJumpConfig {
            max_air_jumps: 1,
            ..default()
        },
        ..default()
    };

    commands.spawn((
        Name::new("Player"),
        Player,
        Sprite {
            color: Color::srgb(0.94, 0.58, 0.22),
            custom_size: Some(PLAYER_SIZE),
            ..default()
        },
        PlatformerControllerBundle::with_config(
            Collider::rectangle(PLAYER_SIZE.x, PLAYER_SIZE.y),
            config,
        )
        .with_transform(Transform::from_xyz(-250.0, -100.0, 10.0)),
        PlatformerGrappleBundle::default(),
    ));

    // --- Level geometry ---

    // Ground floor
    spawn_block(
        &mut commands,
        "Ground",
        Vec2::new(0.0, -200.0),
        Vec2::new(1000.0, 38.0),
        Color::srgb(0.20, 0.22, 0.26),
    );

    // Platforms forming a grapple course
    spawn_block(
        &mut commands,
        "Left Platform",
        Vec2::new(-300.0, -100.0),
        Vec2::new(140.0, 20.0),
        Color::srgb(0.32, 0.38, 0.48),
    );
    spawn_block(
        &mut commands,
        "Right Platform",
        Vec2::new(300.0, -100.0),
        Vec2::new(140.0, 20.0),
        Color::srgb(0.32, 0.38, 0.48),
    );
    spawn_block(
        &mut commands,
        "High Platform",
        Vec2::new(0.0, 100.0),
        Vec2::new(120.0, 20.0),
        Color::srgb(0.44, 0.50, 0.60),
    );

    // Walls
    spawn_block(
        &mut commands,
        "Left Wall",
        Vec2::new(-480.0, 0.0),
        Vec2::new(20.0, 500.0),
        Color::srgb(0.18, 0.20, 0.24),
    );
    spawn_block(
        &mut commands,
        "Right Wall",
        Vec2::new(480.0, 0.0),
        Vec2::new(20.0, 500.0),
        Color::srgb(0.18, 0.20, 0.24),
    );

    // --- Grapple points (green circles) ---
    let grapple_positions = [
        Vec2::new(-120.0, 80.0),
        Vec2::new(120.0, 80.0),
        Vec2::new(0.0, 180.0),
        Vec2::new(-200.0, 30.0),
        Vec2::new(200.0, 30.0),
    ];

    for (i, pos) in grapple_positions.iter().enumerate() {
        commands.spawn((
            Name::new(format!("Grapple Point {}", i + 1)),
            PlatformerGrapplePoint,
            Sprite {
                color: Color::srgb(0.2, 0.85, 0.3),
                custom_size: Some(Vec2::splat(12.0)),
                ..default()
            },
            Transform::from_xyz(pos.x, pos.y, 5.0),
        ));
    }

    // On-screen instructions
    commands.spawn((
        Name::new("Instructions"),
        Text::new(
            "A/D: Move  |  Space: Jump / Detach  |  E: Fire Grapple  |  R: Release\n\
             W/S: Retract/Extend Rope  |  Swing between anchor points!",
        ),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::srgba(1.0, 1.0, 1.0, 0.7)),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(16.0),
            bottom: Val::Px(16.0),
            ..default()
        },
    ));
}

fn spawn_block(commands: &mut Commands, name: &str, center: Vec2, size: Vec2, color: Color) {
    commands.spawn((
        Name::new(name.to_string()),
        Sprite {
            color,
            custom_size: Some(size),
            ..default()
        },
        Transform::from_xyz(center.x, center.y, 0.0),
        RigidBody::Static,
        Collider::rectangle(size.x, size.y),
    ));
}

fn drive_keyboard_intent(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut movement_intent: Single<&mut PlatformerMovementIntent, With<Player>>,
    mut grapple_intent: Single<&mut PlatformerGrappleIntent, With<Player>>,
) {
    let left = keyboard.any_pressed([KeyCode::KeyA, KeyCode::ArrowLeft]);
    let right = keyboard.any_pressed([KeyCode::KeyD, KeyCode::ArrowRight]);

    movement_intent.move_axis = right as i8 as f32 - left as i8 as f32;
    movement_intent.jump_pressed = keyboard.just_pressed(KeyCode::Space);
    movement_intent.jump_held = keyboard.pressed(KeyCode::Space);
    movement_intent.drop_pressed = keyboard.any_just_pressed([KeyCode::KeyS, KeyCode::ArrowDown]);

    grapple_intent.pressed = keyboard.just_pressed(KeyCode::KeyE);
    grapple_intent.released = keyboard.just_pressed(KeyCode::KeyR);
    grapple_intent.retract = keyboard.any_pressed([KeyCode::KeyW, KeyCode::ArrowUp]);
    grapple_intent.extend = keyboard.any_pressed([KeyCode::KeyS, KeyCode::ArrowDown]);
    grapple_intent.direction = Vec2::ZERO;
}

fn sync_pane_to_config(
    pane: Res<GrapplePane>,
    mut grapples: Query<&mut PlatformerGrappleConfig, With<Player>>,
) {
    if !pane.is_changed() {
        return;
    }
    for mut config in &mut grapples {
        config.max_range = pane.max_range;
        config.pull_speed = pane.pull_speed;
        config.detach_speed_boost = pane.detach_boost;
        config.retract_speed = pane.retract_speed;
        config.swing_input_force = pane.swing_input_force;
    }
}

fn update_pane_monitors(
    player: Single<(&PlatformerControllerState, &PlatformerGrappleState), With<Player>>,
    mut pane: ResMut<GrapplePane>,
) {
    pane.phase = match player.1.phase {
        PlatformerGrapplePhase::Idle => format!("{:?}", player.0.phase),
        active_phase => format!("{:?}", active_phase),
    };
    pane.rope_length = match player.1.phase {
        PlatformerGrapplePhase::Pulling { rope_length, .. } => rope_length,
        _ => 0.0,
    };
}

fn follow_camera(
    time: Res<Time>,
    player: Single<&Transform, With<Player>>,
    mut camera: Single<(&FollowCamera, &mut Transform), (With<FollowCamera>, Without<Player>)>,
) {
    let target = player.translation.xy();
    let camera_y = target.y.max(-30.0);
    let desired = Vec3::new(target.x, camera_y, camera.1.translation.z);
    let blend = 1.0 - (-camera.0.smoothing * time.delta_secs()).exp();
    camera.1.translation = camera.1.translation.lerp(desired, blend);
}

fn tint_player(
    mut player: Single<(&PlatformerControllerState, &PlatformerGrappleState, &mut Sprite), With<Player>>,
) {
    player.2.color = if !matches!(player.1.phase, PlatformerGrapplePhase::Idle) {
        Color::srgb(0.30, 0.92, 0.40)
    } else {
        match player.0.phase {
            PlatformerMotionPhase::Grounded => Color::srgb(0.94, 0.58, 0.22),
            PlatformerMotionPhase::Rising => Color::srgb(0.98, 0.82, 0.30),
            PlatformerMotionPhase::Apex => Color::srgb(0.86, 0.86, 0.40),
            PlatformerMotionPhase::Falling => Color::srgb(0.84, 0.42, 0.26),
            PlatformerMotionPhase::WallSliding => Color::srgb(0.42, 0.76, 0.96),
            PlatformerMotionPhase::WallClinging => Color::srgb(0.32, 0.56, 0.88),
            PlatformerMotionPhase::Airborne => Color::srgb(0.92, 0.60, 0.30),
        }
    };
}

fn draw_grapple_rope(
    player: Single<(&Transform, &PlatformerGrappleState), With<Player>>,
    mut gizmos: Gizmos,
) {
    if let PlatformerGrapplePhase::Pulling { target, .. } = player.1.phase {
        let from = player.0.translation.xy();
        gizmos.line_2d(from, target, Color::srgb(0.30, 0.92, 0.40));
        gizmos.circle_2d(target, 6.0, Color::srgb(0.30, 0.92, 0.40));
    }
}
