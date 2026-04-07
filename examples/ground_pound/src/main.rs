//! Platformer controller — ground pound example
//!
//! Demonstrates the ground pound mechanic: hover briefly mid-air, then slam
//! downward at high speed. Land on elevated platforms and use ground pound
//! to smash back down. The player color changes to red during the slam.
//!
//! Controls:
//!   A / D or ←/→  — move
//!   Space          — jump (hold for higher)
//!   Q              — ground pound (mid-air)
//!   S / ↓          — drop through one-way platforms

use avian2d::prelude::*;
use bevy::{app::AppExit, camera::ScalingMode, prelude::*, window::WindowResolution};
use saddle_character_platformer_controller::{
    PlatformerControllerBundle, PlatformerControllerConfig, PlatformerControllerPlugin,
    PlatformerControllerState, PlatformerGroundPoundBundle, PlatformerGroundPoundConfig,
    PlatformerGroundPoundIntent, PlatformerGroundPoundPhase, PlatformerGroundPoundPlugin,
    PlatformerGroundPoundState, PlatformerMotionPhase, PlatformerMovementIntent,
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
#[pane(title = "Ground Pound", position = "top-right")]
struct GroundPoundPane {
    #[pane(slider, min = 0.0, max = 0.3, step = 0.01)]
    hover_duration: f32,
    #[pane(slider, min = 200.0, max = 1200.0, step = 10.0)]
    fall_speed: f32,
    #[pane(slider, min = 0.0, max = 0.4, step = 0.01)]
    impact_stun: f32,
    #[pane(checkbox)]
    cancel_horizontal: bool,
    #[pane(monitor)]
    phase: String,
    #[pane(monitor)]
    grounded: bool,
    #[pane(monitor)]
    velocity_y: f32,
}

impl Default for GroundPoundPane {
    fn default() -> Self {
        let gp = PlatformerGroundPoundConfig::default();
        Self {
            hover_duration: gp.hover_duration,
            fall_speed: gp.fall_speed,
            impact_stun: gp.impact_stun_duration,
            cancel_horizontal: gp.cancel_horizontal_speed,
            phase: "Grounded".into(),
            grounded: false,
            velocity_y: 0.0,
        }
    }
}

fn main() -> AppExit {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.06, 0.07, 0.09)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "platformer_controller / ground_pound".into(),
                resolution: WindowResolution::new(1440, 900),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(Time::<Fixed>::from_hz(60.0))
        .add_plugins(PhysicsPlugins::default().with_length_unit(20.0))
        .add_plugins((
            PlatformerControllerPlugin::always_on(FixedUpdate),
            PlatformerGroundPoundPlugin::always_on(FixedUpdate),
        ))
        .add_plugins((
            bevy_flair::FlairPlugin,
            bevy_input_focus::InputDispatchPlugin,
            bevy_ui_widgets::UiWidgetsPlugins,
            bevy_input_focus::tab_navigation::TabNavigationPlugin,
            PanePlugin,
        ))
        .register_pane::<GroundPoundPane>()
        .add_systems(Startup, setup_scene)
        .add_systems(Update, drive_keyboard_intent)
        .add_systems(Update, (sync_pane_to_config, update_pane_monitors).chain())
        .add_systems(PostUpdate, (follow_camera, tint_player))
        .run()
}

fn setup_scene(mut commands: Commands) {
    commands.spawn((
        Name::new("Camera"),
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: 400.0,
            },
            ..OrthographicProjection::default_2d()
        }),
        Transform::from_xyz(0.0, 0.0, 999.0),
        FollowCamera { smoothing: 8.0 },
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
        .with_transform(Transform::from_xyz(-200.0, -20.0, 10.0)),
        PlatformerGroundPoundBundle::default(),
    ));

    // --- Level geometry ---

    // Ground floor
    spawn_block(
        &mut commands,
        "Ground",
        Vec2::new(0.0, -150.0),
        Vec2::new(800.0, 38.0),
        Color::srgb(0.20, 0.22, 0.26),
    );

    // Elevated platforms at various heights — jump up and ground-pound down
    spawn_block(
        &mut commands,
        "Low Platform",
        Vec2::new(-160.0, -80.0),
        Vec2::new(120.0, 20.0),
        Color::srgb(0.32, 0.38, 0.48),
    );
    spawn_block(
        &mut commands,
        "Mid Platform",
        Vec2::new(0.0, -20.0),
        Vec2::new(120.0, 20.0),
        Color::srgb(0.38, 0.44, 0.54),
    );
    spawn_block(
        &mut commands,
        "High Platform",
        Vec2::new(160.0, 40.0),
        Vec2::new(120.0, 20.0),
        Color::srgb(0.44, 0.50, 0.60),
    );
    spawn_block(
        &mut commands,
        "Top Platform",
        Vec2::new(0.0, 100.0),
        Vec2::new(160.0, 20.0),
        Color::srgb(0.50, 0.56, 0.66),
    );

    // Walls to contain the area
    spawn_block(
        &mut commands,
        "Left Wall",
        Vec2::new(-380.0, 0.0),
        Vec2::new(20.0, 400.0),
        Color::srgb(0.18, 0.20, 0.24),
    );
    spawn_block(
        &mut commands,
        "Right Wall",
        Vec2::new(380.0, 0.0),
        Vec2::new(20.0, 400.0),
        Color::srgb(0.18, 0.20, 0.24),
    );

    // On-screen instructions
    commands.spawn((
        Name::new("Instructions"),
        Text::new(
            "A/D: Move  |  Space: Jump  |  Q: Ground Pound (mid-air)\n\
             Jump to a platform, then press Q to slam downward!",
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
    mut ground_pound_intent: Single<&mut PlatformerGroundPoundIntent, With<Player>>,
) {
    let left = keyboard.any_pressed([KeyCode::KeyA, KeyCode::ArrowLeft]);
    let right = keyboard.any_pressed([KeyCode::KeyD, KeyCode::ArrowRight]);

    // Continuous state — always overwrite with latest value.
    movement_intent.move_axis = right as i8 as f32 - left as i8 as f32;
    movement_intent.jump_held = keyboard.pressed(KeyCode::Space);

    // One-shot flags — latch on, never overwrite to false.
    if keyboard.just_pressed(KeyCode::Space) {
        movement_intent.jump_pressed = true;
    }
    if keyboard.any_just_pressed([KeyCode::KeyS, KeyCode::ArrowDown]) {
        movement_intent.drop_pressed = true;
    }
    if keyboard.just_pressed(KeyCode::KeyQ) {
        ground_pound_intent.pressed = true;
    }
}

fn sync_pane_to_config(
    pane: Res<GroundPoundPane>,
    mut ground_pounds: Query<&mut PlatformerGroundPoundConfig, With<Player>>,
) {
    if !pane.is_changed() {
        return;
    }
    for mut config in &mut ground_pounds {
        config.hover_duration = pane.hover_duration;
        config.fall_speed = pane.fall_speed;
        config.impact_stun_duration = pane.impact_stun;
        config.cancel_horizontal_speed = pane.cancel_horizontal;
    }
}

fn update_pane_monitors(
    player: Single<(&PlatformerControllerState, &PlatformerGroundPoundState), With<Player>>,
    mut pane: ResMut<GroundPoundPane>,
) {
    pane.phase = if player.1.phase != PlatformerGroundPoundPhase::Idle {
        format!("{:?}", player.1.phase)
    } else {
        format!("{:?}", player.0.phase)
    };
    pane.grounded = player.0.is_grounded;
    pane.velocity_y = player.0.velocity.y;
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
    mut player: Single<
        (&PlatformerControllerState, &PlatformerGroundPoundState, &mut Sprite),
        With<Player>,
    >,
) {
    player.2.color = if player.1.phase != PlatformerGroundPoundPhase::Idle {
        Color::srgb(0.88, 0.18, 0.18)
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
