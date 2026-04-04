//! Platformer controller — one-way platforms example
//!
//! Demonstrates one-way (pass-through) platforms: the player can jump up
//! through them from below and land on top, then press Down/S to drop through.

use avian2d::prelude::*;
use bevy::{app::AppExit, camera::ScalingMode, prelude::*, window::WindowResolution};
use saddle_character_platformer_controller::{
    PlatformerControllerBundle, PlatformerControllerConfig, PlatformerControllerPlugin,
    PlatformerControllerState, PlatformerControllerSystems, PlatformerMotionPhase,
    PlatformerMovementIntent, PlatformerOneWayPlatform,
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

// ---------------------------------------------------------------------------
// Pane
// ---------------------------------------------------------------------------

#[derive(Resource, Debug, Clone, PartialEq, Pane)]
#[pane(title = "Platformer — One-Way Platforms", position = "top-right")]
struct OneWayPane {
    #[pane(slider, min = 40.0, max = 140.0, step = 1.0)]
    jump_height: f32,
    #[pane(slider, min = 0.05, max = 0.5, step = 0.01)]
    drop_through_duration: f32,
    #[pane(monitor)]
    phase: String,
    #[pane(monitor)]
    grounded: bool,
}

impl Default for OneWayPane {
    fn default() -> Self {
        Self {
            jump_height: 84.0,
            drop_through_duration: 0.22,
            phase: "Grounded".into(),
            grounded: false,
        }
    }
}

// ---------------------------------------------------------------------------
// System sets
// ---------------------------------------------------------------------------

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum DemoSystems {
    DriveIntent,
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

fn main() -> AppExit {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.06, 0.07, 0.09)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "platformer_controller / one_way_platforms".into(),
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
        .register_pane::<OneWayPane>()
        .configure_sets(
            FixedUpdate,
            DemoSystems::DriveIntent.before(PlatformerControllerSystems::ReadIntent),
        )
        .add_systems(Startup, setup_scene)
        .add_systems(FixedUpdate, drive_keyboard_intent.in_set(DemoSystems::DriveIntent))
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

    // Player — one-way platform config
    let config = PlatformerControllerConfig {
        jump: saddle_character_platformer_controller::PlatformerJumpConfig {
            height: 84.0,
            time_to_apex: 0.42,
            ..default()
        },
        platforms: saddle_character_platformer_controller::PlatformInteractionConfig {
            drop_through_duration: 0.22,
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
        .with_transform(Transform::from_xyz(-260.0, -20.0, 10.0)),
    ));

    // --- Level geometry ---
    spawn_solid_block(&mut commands, "Ground", Vec2::new(0.0, -150.0), Vec2::new(920.0, 38.0), Color::srgb(0.20, 0.22, 0.26));

    // One-way platforms — tagged with PlatformerOneWayPlatform so the
    // controller allows upward passage and enables drop-through.
    let one_way_color = Color::srgba(0.72, 0.82, 0.95, 0.35);
    let one_way_size = Vec2::new(170.0, 12.0);
    spawn_one_way_platform(&mut commands, "One Way A", Vec2::new(-110.0, -60.0), one_way_size, one_way_color);
    spawn_one_way_platform(&mut commands, "One Way B", Vec2::new(40.0, 15.0),    one_way_size, one_way_color);
    spawn_one_way_platform(&mut commands, "One Way C", Vec2::new(200.0, 90.0),   one_way_size, one_way_color);

    // Solid backdrop tower
    spawn_solid_block(&mut commands, "Backdrop Tower", Vec2::new(290.0, -8.0), Vec2::new(90.0, 160.0), Color::srgb(0.26, 0.28, 0.34));
}

fn spawn_solid_block(commands: &mut Commands, name: &str, center: Vec2, size: Vec2, color: Color) {
    commands.spawn((
        Name::new(name.to_string()),
        Sprite { color, custom_size: Some(size), ..default() },
        Transform::from_xyz(center.x, center.y, 0.0),
        RigidBody::Static,
        Collider::rectangle(size.x, size.y),
    ));
}

fn spawn_one_way_platform(commands: &mut Commands, name: &str, center: Vec2, size: Vec2, color: Color) {
    commands.spawn((
        Name::new(name.to_string()),
        PlatformerOneWayPlatform,
        Sprite { color, custom_size: Some(size), ..default() },
        Transform::from_xyz(center.x, center.y, 0.0),
        RigidBody::Static,
        Collider::rectangle(size.x, size.y),
    ));
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
}

// ---------------------------------------------------------------------------
// Pane sync
// ---------------------------------------------------------------------------

fn sync_pane_to_config(
    pane: Res<OneWayPane>,
    mut controllers: Query<&mut PlatformerControllerConfig, With<Player>>,
) {
    if !pane.is_changed() {
        return;
    }
    for mut config in &mut controllers {
        config.jump.height = pane.jump_height;
        config.platforms.drop_through_duration = pane.drop_through_duration;
    }
}

fn update_pane_monitors(
    player: Single<&PlatformerControllerState, With<Player>>,
    mut pane: ResMut<OneWayPane>,
) {
    pane.grounded = player.is_grounded;
    pane.phase = format!("{:?}", player.phase);
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
        PlatformerMotionPhase::Airborne => Color::srgb(0.92, 0.60, 0.30),
    };
}
