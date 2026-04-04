//! Platformer controller — basic example
//!
//! Shows the minimum setup for a 2D platformer character: physics plugin,
//! controller plugin, a player entity with the controller bundle, flat ground
//! with a few platforms, and keyboard-driven intent.

use avian2d::prelude::*;
use bevy::{app::AppExit, camera::ScalingMode, prelude::*, window::WindowResolution};
use saddle_character_platformer_controller::{
    PlatformerControllerBundle, PlatformerControllerConfig, PlatformerControllerPlugin,
    PlatformerControllerState, PlatformerControllerSystems, PlatformerMotionPhase,
    PlatformerMovementIntent,
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
// Pane — live-tweak parameters
// ---------------------------------------------------------------------------

#[derive(Resource, Debug, Clone, PartialEq, Pane)]
#[pane(title = "Platformer — Basic", position = "top-right")]
struct BasicPane {
    #[pane(slider, min = 120.0, max = 360.0, step = 1.0)]
    max_speed: f32,
    #[pane(slider, min = 40.0, max = 140.0, step = 1.0)]
    jump_height: f32,
    #[pane(slider, min = 0.2, max = 0.8, step = 0.01)]
    time_to_apex: f32,
    #[pane(slider, min = 0.0, max = 0.25, step = 0.01)]
    coyote_time: f32,
    #[pane(slider, min = 0.0, max = 0.25, step = 0.01)]
    jump_buffer_time: f32,
    #[pane(slider, min = 0, max = 3, step = 1)]
    max_air_jumps: u32,
    #[pane(monitor)]
    phase: String,
    #[pane(monitor)]
    grounded: bool,
}

impl Default for BasicPane {
    fn default() -> Self {
        Self {
            max_speed: 240.0,
            jump_height: 88.0,
            time_to_apex: 0.4,
            coyote_time: 0.11,
            jump_buffer_time: 0.12,
            max_air_jumps: 1,
            phase: "Grounded".into(),
            grounded: false,
        }
    }
}

// ---------------------------------------------------------------------------
// System sets for ordering
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
        // --- Window & rendering ---
        .insert_resource(ClearColor(Color::srgb(0.06, 0.07, 0.09)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "platformer_controller / basic".into(),
                resolution: WindowResolution::new(1440, 900),
                ..default()
            }),
            ..default()
        }))
        // --- Physics ---
        .insert_resource(Time::<Fixed>::from_hz(60.0))
        .add_plugins(PhysicsPlugins::default().with_length_unit(20.0))
        // --- Platformer controller ---
        .add_plugins(PlatformerControllerPlugin::always_on(FixedUpdate))
        // --- Pane (live-tweak UI) ---
        .add_plugins((
            bevy_flair::FlairPlugin,
            bevy_input_focus::InputDispatchPlugin,
            bevy_ui_widgets::UiWidgetsPlugins,
            bevy_input_focus::tab_navigation::TabNavigationPlugin,
            PanePlugin,
        ))
        .register_pane::<BasicPane>()
        // --- Ordering: keyboard intent runs before the controller reads it ---
        .configure_sets(
            FixedUpdate,
            DemoSystems::DriveIntent.before(PlatformerControllerSystems::ReadIntent),
        )
        // --- Systems ---
        .add_systems(Startup, setup_scene)
        .add_systems(FixedUpdate, drive_keyboard_intent.in_set(DemoSystems::DriveIntent))
        .add_systems(Update, (sync_pane_to_config, update_pane_monitors).chain())
        .add_systems(PostUpdate, (follow_camera, tint_player))
        .run()
}

// ---------------------------------------------------------------------------
// Scene setup — camera, player, level geometry
// ---------------------------------------------------------------------------

fn setup_scene(mut commands: Commands) {
    // Camera
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

    // Player — the controller bundle provides all physics + state components
    let config = PlatformerControllerConfig {
        movement: saddle_character_platformer_controller::MovementConfig {
            max_speed: 240.0,
            ..default()
        },
        jump: saddle_character_platformer_controller::PlatformerJumpConfig {
            height: 88.0,
            time_to_apex: 0.4,
            coyote_time: 0.11,
            jump_buffer_time: 0.12,
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
        .with_transform(Transform::from_xyz(-270.0, -40.0, 10.0)),
    ));

    // --- Level geometry ---

    // Ground
    spawn_block(&mut commands, "Ground", Vec2::new(0.0, -150.0), Vec2::new(920.0, 38.0), Color::srgb(0.20, 0.22, 0.26));
    // Step
    spawn_block(&mut commands, "Step", Vec2::new(-85.0, -86.0), Vec2::new(110.0, 28.0), Color::srgb(0.30, 0.36, 0.43));
    // Tower
    spawn_block(&mut commands, "Tower", Vec2::new(175.0, -28.0), Vec2::new(80.0, 160.0), Color::srgb(0.33, 0.40, 0.49));
    // Ramp (rotated)
    commands.spawn((
        Name::new("Ramp"),
        Sprite {
            color: Color::srgb(0.28, 0.48, 0.42),
            custom_size: Some(Vec2::new(200.0, 20.0)),
            ..default()
        },
        Transform::from_xyz(5.0, -118.0, 0.0)
            .with_rotation(Quat::from_rotation_z(0.32)),
        RigidBody::Static,
        Collider::rectangle(200.0, 20.0),
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
// Pane ←→ config sync
// ---------------------------------------------------------------------------

fn sync_pane_to_config(
    pane: Res<BasicPane>,
    mut controllers: Query<&mut PlatformerControllerConfig, With<Player>>,
) {
    if !pane.is_changed() {
        return;
    }
    for mut config in &mut controllers {
        config.movement.max_speed = pane.max_speed;
        config.jump.height = pane.jump_height;
        config.jump.time_to_apex = pane.time_to_apex.max(0.01);
        config.jump.coyote_time = pane.coyote_time;
        config.jump.jump_buffer_time = pane.jump_buffer_time;
        config.jump.max_air_jumps = pane.max_air_jumps;
    }
}

fn update_pane_monitors(
    player: Single<&PlatformerControllerState, With<Player>>,
    mut pane: ResMut<BasicPane>,
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
// Visual feedback — tint player sprite by motion phase
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
