//! Platformer controller — wall jumps example
//!
//! Demonstrates wall-slide and wall-jump configuration: a vertical shaft with
//! two walls, small ledges, and a finish platform at the top. The player runs
//! into a wall, holds toward it to slide, and presses jump to kick off across.

use avian2d::prelude::*;
use bevy::{app::AppExit, camera::ScalingMode, prelude::*, window::WindowResolution};
use saddle_character_platformer_controller::{
    PlatformerControllerBundle, PlatformerControllerConfig, PlatformerControllerPlugin,
    PlatformerControllerState, PlatformerControllerSystems, PlatformerMotionPhase,
    PlatformerMovementIntent, PlatformerWallConfig,
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
#[pane(title = "Platformer — Wall Jumps", position = "top-right")]
struct WallJumpPane {
    #[pane(slider, min = 40.0, max = 200.0, step = 1.0)]
    wall_slide_terminal_speed: f32,
    #[pane(slider, min = 0.1, max = 1.0, step = 0.01)]
    wall_slide_gravity_multiplier: f32,
    #[pane(slider, min = 100.0, max = 400.0, step = 1.0)]
    wall_jump_horizontal_speed: f32,
    #[pane(slider, min = 100.0, max = 500.0, step = 1.0)]
    wall_jump_vertical_speed: f32,
    #[pane(slider, min = 0.0, max = 0.4, step = 0.01)]
    wall_jump_steering_lock_time: f32,
    #[pane(monitor)]
    phase: String,
    #[pane(monitor)]
    grounded: bool,
}

impl Default for WallJumpPane {
    fn default() -> Self {
        Self {
            wall_slide_terminal_speed: 96.0,
            wall_slide_gravity_multiplier: 0.42,
            wall_jump_horizontal_speed: 250.0,
            wall_jump_vertical_speed: 305.0,
            wall_jump_steering_lock_time: 0.16,
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
                title: "platformer_controller / wall_jumps".into(),
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
        .register_pane::<WallJumpPane>()
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
                viewport_height: 280.0,
            },
            ..OrthographicProjection::default_2d()
        }),
        Transform::from_xyz(0.0, 0.0, 999.0),
        FollowCamera { smoothing: 8.0 },
    ));

    // Player — wall-jump-tuned config: no air jumps, enhanced wall parameters
    let config = PlatformerControllerConfig {
        movement: saddle_character_platformer_controller::MovementConfig {
            max_speed: 230.0,
            ..default()
        },
        jump: saddle_character_platformer_controller::PlatformerJumpConfig {
            height: 92.0,
            time_to_apex: 0.38,
            max_air_jumps: 0,
            ..default()
        },
        walls: PlatformerWallConfig {
            wall_slide_terminal_speed: 96.0,
            wall_slide_gravity_multiplier: 0.42,
            wall_jump_horizontal_speed: 250.0,
            wall_jump_vertical_speed: 305.0,
            wall_jump_steering_lock_time: 0.16,
            wall_jump_steering_factor: 0.08,
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
        .with_transform(Transform::from_xyz(0.0, -70.0, 10.0)),
    ));

    // --- Shaft geometry ---
    spawn_block(
        &mut commands,
        "Ground",
        Vec2::new(0.0, -150.0),
        Vec2::new(520.0, 38.0),
        Color::srgb(0.20, 0.22, 0.26),
    );
    spawn_block(
        &mut commands,
        "Left Wall",
        Vec2::new(-120.0, 30.0),
        Vec2::new(40.0, 360.0),
        Color::srgb(0.25, 0.30, 0.36),
    );
    spawn_block(
        &mut commands,
        "Right Wall",
        Vec2::new(120.0, 30.0),
        Vec2::new(40.0, 360.0),
        Color::srgb(0.25, 0.30, 0.36),
    );
    spawn_block(
        &mut commands,
        "Left Ledge",
        Vec2::new(-70.0, -5.0),
        Vec2::new(62.0, 18.0),
        Color::srgb(0.43, 0.34, 0.30),
    );
    spawn_block(
        &mut commands,
        "Right Ledge",
        Vec2::new(70.0, 85.0),
        Vec2::new(62.0, 18.0),
        Color::srgb(0.43, 0.34, 0.30),
    );
    spawn_block(
        &mut commands,
        "Finish Ledge",
        Vec2::new(0.0, 170.0),
        Vec2::new(150.0, 20.0),
        Color::srgb(0.38, 0.52, 0.40),
    );
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
    intent.ground_pound_pressed = keyboard.just_pressed(KeyCode::KeyQ);
}

// ---------------------------------------------------------------------------
// Pane sync
// ---------------------------------------------------------------------------

fn sync_pane_to_config(
    pane: Res<WallJumpPane>,
    mut controllers: Query<&mut PlatformerControllerConfig, With<Player>>,
) {
    if !pane.is_changed() {
        return;
    }
    for mut config in &mut controllers {
        config.walls.wall_slide_terminal_speed = pane.wall_slide_terminal_speed;
        config.walls.wall_slide_gravity_multiplier = pane.wall_slide_gravity_multiplier;
        config.walls.wall_jump_horizontal_speed = pane.wall_jump_horizontal_speed;
        config.walls.wall_jump_vertical_speed = pane.wall_jump_vertical_speed;
        config.walls.wall_jump_steering_lock_time = pane.wall_jump_steering_lock_time;
    }
}

fn update_pane_monitors(
    player: Single<&PlatformerControllerState, With<Player>>,
    mut pane: ResMut<WallJumpPane>,
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
        PlatformerMotionPhase::WallClinging => Color::srgb(0.32, 0.56, 0.88),
        PlatformerMotionPhase::GroundPounding => Color::srgb(0.88, 0.18, 0.18),
        PlatformerMotionPhase::Grappling => Color::srgb(0.60, 0.92, 0.30),
        PlatformerMotionPhase::Airborne => Color::srgb(0.92, 0.60, 0.30),
    };
}
