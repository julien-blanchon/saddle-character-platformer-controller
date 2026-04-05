//! Platformer controller — moving platforms example
//!
//! Demonstrates kinematic moving platforms with velocity inheritance. A
//! horizontal and a vertical platform oscillate, and the player inherits their
//! velocity when standing on them or jumping off.

use avian2d::prelude::*;
use bevy::{app::AppExit, camera::ScalingMode, prelude::*, window::WindowResolution};
use saddle_character_platformer_controller::{
    PlatformVelocityInheritance, PlatformerControllerBundle, PlatformerControllerConfig,
    PlatformerControllerPlugin, PlatformerControllerState, PlatformerControllerSystems,
    PlatformerMotionPhase, PlatformerMovementIntent,
};
use saddle_pane::prelude::*;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const PLAYER_SIZE: Vec2 = Vec2::new(18.0, 30.0);

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

#[derive(Component)]
struct Player;

#[derive(Component)]
struct FollowCamera {
    smoothing: f32,
}

/// Marks a kinematic body that oscillates along an axis.
#[derive(Component)]
struct MovingPlatform {
    origin: Vec2,
    axis: Vec2,
    amplitude: f32,
    speed: f32,
}

// ---------------------------------------------------------------------------
// Pane
// ---------------------------------------------------------------------------

#[derive(Resource, Debug, Clone, PartialEq, Pane)]
#[pane(title = "Platformer — Moving Platforms", position = "top-right")]
struct MovingPlatformPane {
    #[pane(slider, min = 40.0, max = 140.0, step = 1.0)]
    jump_height: f32,
    #[pane(slider, min = 0.2, max = 0.8, step = 0.01)]
    time_to_apex: f32,
    #[pane(monitor)]
    phase: String,
    #[pane(monitor)]
    grounded: bool,
    #[pane(monitor)]
    support_velocity_x: f32,
    #[pane(monitor)]
    support_velocity_y: f32,
}

impl Default for MovingPlatformPane {
    fn default() -> Self {
        Self {
            jump_height: 86.0,
            time_to_apex: 0.43,
            phase: "Grounded".into(),
            grounded: false,
            support_velocity_x: 0.0,
            support_velocity_y: 0.0,
        }
    }
}

// ---------------------------------------------------------------------------
// System sets
// ---------------------------------------------------------------------------

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum DemoSystems {
    DriveIntent,
    AnimatePlatforms,
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

fn main() -> AppExit {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.06, 0.07, 0.09)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "platformer_controller / moving_platforms".into(),
                resolution: WindowResolution::new(1440, 900),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(Time::<Fixed>::from_hz(60.0))
        .add_plugins(PhysicsPlugins::default().with_length_unit(20.0))
        .add_plugins(PlatformerControllerPlugin::always_on(FixedUpdate))
        .add_plugins((
            bevy_flair::FlairPlugin,
            bevy_input_focus::InputDispatchPlugin,
            bevy_ui_widgets::UiWidgetsPlugins,
            bevy_input_focus::tab_navigation::TabNavigationPlugin,
            PanePlugin,
        ))
        .register_pane::<MovingPlatformPane>()
        .configure_sets(
            FixedUpdate,
            (
                DemoSystems::DriveIntent.before(PlatformerControllerSystems::ReadIntent),
                DemoSystems::AnimatePlatforms.before(PlatformerControllerSystems::SenseContacts),
            ),
        )
        .add_systems(Startup, setup_scene)
        .add_systems(
            FixedUpdate,
            (
                drive_keyboard_intent.in_set(DemoSystems::DriveIntent),
                animate_platforms.in_set(DemoSystems::AnimatePlatforms),
            ),
        )
        .add_systems(Update, (sync_pane_to_config, update_pane_monitors).chain())
        .add_systems(PostUpdate, (follow_camera, tint_player))
        .run()
}

// ---------------------------------------------------------------------------
// Scene setup
// ---------------------------------------------------------------------------

fn setup_scene(mut commands: Commands) {
    commands.spawn((
        Name::new("Camera"),
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: 320.0,
            },
            ..OrthographicProjection::default_2d()
        }),
        Transform::from_xyz(0.0, 0.0, 999.0),
        FollowCamera { smoothing: 8.0 },
    ));

    // Player — full velocity inheritance from platforms
    let config = PlatformerControllerConfig {
        jump: saddle_character_platformer_controller::PlatformerJumpConfig {
            height: 86.0,
            time_to_apex: 0.43,
            ..default()
        },
        platforms: saddle_character_platformer_controller::PlatformInteractionConfig {
            velocity_inheritance: PlatformVelocityInheritance::Full,
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
        .with_transform(Transform::from_xyz(-260.0, -50.0, 10.0)),
    ));

    // --- Level geometry ---
    spawn_static_block(
        &mut commands,
        "Ground",
        Vec2::new(0.0, -150.0),
        Vec2::new(920.0, 38.0),
        Color::srgb(0.20, 0.22, 0.26),
    );

    // Horizontal moving platform
    spawn_moving_platform(
        &mut commands,
        "Horizontal Platform",
        Vec2::new(-40.0, -20.0),
        Vec2::X,
        130.0,
        0.7,
        Vec2::new(120.0, 18.0),
        Color::srgb(0.76, 0.56, 0.20),
    );

    // Vertical moving platform
    spawn_moving_platform(
        &mut commands,
        "Vertical Platform",
        Vec2::new(170.0, -50.0),
        Vec2::Y,
        95.0,
        0.9,
        Vec2::new(100.0, 18.0),
        Color::srgb(0.32, 0.58, 0.74),
    );

    // Landing deck (static)
    spawn_static_block(
        &mut commands,
        "Landing Deck",
        Vec2::new(330.0, 80.0),
        Vec2::new(180.0, 20.0),
        Color::srgb(0.28, 0.40, 0.48),
    );
}

fn spawn_static_block(commands: &mut Commands, name: &str, center: Vec2, size: Vec2, color: Color) {
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

fn spawn_moving_platform(
    commands: &mut Commands,
    name: &str,
    origin: Vec2,
    axis: Vec2,
    amplitude: f32,
    speed: f32,
    size: Vec2,
    color: Color,
) {
    commands.spawn((
        Name::new(name.to_string()),
        MovingPlatform {
            origin,
            axis,
            amplitude,
            speed,
        },
        Sprite {
            color,
            custom_size: Some(size),
            ..default()
        },
        Transform::from_xyz(origin.x, origin.y, 0.0),
        Position(origin),
        Rotation::default(),
        LinearVelocity::ZERO,
        RigidBody::Kinematic,
        CustomPositionIntegration,
        Collider::rectangle(size.x, size.y),
    ));
}

// ---------------------------------------------------------------------------
// Animate moving platforms (sinusoidal oscillation)
// ---------------------------------------------------------------------------

fn animate_platforms(
    time: Res<Time>,
    mut platforms: Query<(
        &MovingPlatform,
        &mut Position,
        &mut LinearVelocity,
        &mut Transform,
    )>,
) {
    let dt = time.delta_secs().max(f32::EPSILON);

    for (platform, mut position, mut velocity, mut transform) in &mut platforms {
        let progress = time.elapsed_secs() * platform.speed;
        let next = platform.origin
            + platform.axis.normalize_or_zero() * platform.amplitude * progress.sin();
        velocity.0 = (next - position.0) / dt;
        position.0 = next;
        transform.translation.x = next.x;
        transform.translation.y = next.y;
    }
}

// ---------------------------------------------------------------------------
// Keyboard → intent
// ---------------------------------------------------------------------------

fn drive_keyboard_intent(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut intent: Single<&mut PlatformerMovementIntent, With<Player>>,
) {
    let left = keyboard.any_pressed([KeyCode::KeyA, KeyCode::ArrowLeft]);
    let right = keyboard.any_pressed([KeyCode::KeyD, KeyCode::ArrowRight]);

    intent.move_axis = right as i8 as f32 - left as i8 as f32;
    intent.jump_pressed = keyboard.just_pressed(KeyCode::Space);
    intent.jump_held = keyboard.pressed(KeyCode::Space);
    intent.dash_pressed = keyboard.any_just_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
    intent.drop_pressed = keyboard.any_just_pressed([KeyCode::KeyS, KeyCode::ArrowDown]);
    intent.ground_pound_pressed = keyboard.just_pressed(KeyCode::KeyQ);
}

// ---------------------------------------------------------------------------
// Pane sync
// ---------------------------------------------------------------------------

fn sync_pane_to_config(
    pane: Res<MovingPlatformPane>,
    mut controllers: Query<&mut PlatformerControllerConfig, With<Player>>,
) {
    if !pane.is_changed() {
        return;
    }
    for mut config in &mut controllers {
        config.jump.height = pane.jump_height;
        config.jump.time_to_apex = pane.time_to_apex.max(0.01);
    }
}

fn update_pane_monitors(
    player: Single<&PlatformerControllerState, With<Player>>,
    mut pane: ResMut<MovingPlatformPane>,
) {
    pane.grounded = player.is_grounded;
    pane.phase = format!("{:?}", player.phase);
    pane.support_velocity_x = player.support_velocity.x;
    pane.support_velocity_y = player.support_velocity.y;
}

// ---------------------------------------------------------------------------
// Camera follow
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Tint
// ---------------------------------------------------------------------------

fn tint_player(mut player: Single<(&PlatformerControllerState, &mut Sprite), With<Player>>) {
    player.1.color = match player.0.phase {
        PlatformerMotionPhase::Grounded => Color::srgb(0.94, 0.58, 0.22),
        PlatformerMotionPhase::Dashing => Color::srgb(0.98, 0.28, 0.48),
        PlatformerMotionPhase::Rising => Color::srgb(0.98, 0.82, 0.30),
        PlatformerMotionPhase::Apex => Color::srgb(0.86, 0.86, 0.40),
        PlatformerMotionPhase::Falling => Color::srgb(0.84, 0.42, 0.26),
        PlatformerMotionPhase::WallSliding => Color::srgb(0.42, 0.76, 0.96),
        PlatformerMotionPhase::WallClinging => Color::srgb(0.32, 0.56, 0.88),
        PlatformerMotionPhase::GroundPounding => Color::srgb(0.88, 0.18, 0.18),
        PlatformerMotionPhase::Grappling => Color::srgb(0.60, 0.92, 0.30),
        PlatformerMotionPhase::Airborne => Color::srgb(0.92, 0.60, 0.30),
    };
}
