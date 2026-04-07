//! Platformer controller — surface modifiers example
//!
//! Demonstrates per-surface physics: ice (low friction), conveyor belts
//! (surface velocity), and mud (reduced max speed). Walk across different
//! surfaces to feel the difference.
//!
//! Controls:
//!   A / D or ←/→  — move
//!   Space          — jump (hold for higher)
//!   S / ↓          — drop through one-way platforms

use avian2d::prelude::*;
use bevy::{app::AppExit, camera::ScalingMode, prelude::*, window::WindowResolution};
use saddle_character_platformer_controller::{
    PlatformerControllerBundle, PlatformerControllerConfig, PlatformerControllerPlugin,
    PlatformerControllerState, PlatformerMotionPhase, PlatformerMovementIntent,
    PlatformerSurfaceModifier,
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
#[pane(title = "Surface Modifiers", position = "top-right")]
struct SurfacePane {
    #[pane(monitor)]
    phase: String,
    #[pane(monitor)]
    surface: String,
    #[pane(monitor)]
    velocity_x: f32,
}

impl Default for SurfacePane {
    fn default() -> Self {
        Self {
            phase: "Grounded".into(),
            surface: "Normal".into(),
            velocity_x: 0.0,
        }
    }
}

fn main() -> AppExit {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.06, 0.07, 0.09)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "platformer_controller / surface_modifiers".into(),
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
        .register_pane::<SurfacePane>()
        .add_systems(Startup, setup_scene)
        .add_systems(Update, drive_keyboard_intent)
        .add_systems(Update, update_pane_monitors)
        .add_systems(PostUpdate, (follow_camera, tint_player))
        .run()
}

fn setup_scene(mut commands: Commands) {
    commands.spawn((
        Name::new("Camera"),
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: 360.0,
            },
            ..OrthographicProjection::default_2d()
        }),
        Transform::from_xyz(0.0, 0.0, 999.0),
        FollowCamera { smoothing: 8.0 },
    ));

    // Player
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
            PlatformerControllerConfig::default(),
        )
        .with_transform(Transform::from_xyz(-380.0, -40.0, 10.0)),
    ));

    // --- Normal ground ---
    spawn_block(
        &mut commands,
        "Normal Ground",
        Vec2::new(-340.0, -100.0),
        Vec2::new(160.0, 30.0),
        Color::srgb(0.30, 0.30, 0.35),
        None,
    );

    // --- Ice surface ---
    spawn_block(
        &mut commands,
        "Ice Surface",
        Vec2::new(-140.0, -100.0),
        Vec2::new(180.0, 30.0),
        Color::srgb(0.55, 0.82, 0.96),
        Some(PlatformerSurfaceModifier {
            friction_multiplier: 0.12,
            surface_velocity: Vec2::ZERO,
            speed_multiplier: 1.2,
        }),
    );

    // --- Conveyor belt (right) ---
    spawn_block(
        &mut commands,
        "Conveyor Right",
        Vec2::new(80.0, -100.0),
        Vec2::new(180.0, 30.0),
        Color::srgb(0.85, 0.55, 0.20),
        Some(PlatformerSurfaceModifier {
            friction_multiplier: 1.0,
            surface_velocity: Vec2::new(120.0, 0.0),
            speed_multiplier: 1.0,
        }),
    );

    // --- Conveyor belt (left) ---
    spawn_block(
        &mut commands,
        "Conveyor Left",
        Vec2::new(300.0, -100.0),
        Vec2::new(180.0, 30.0),
        Color::srgb(0.85, 0.55, 0.20),
        Some(PlatformerSurfaceModifier {
            friction_multiplier: 1.0,
            surface_velocity: Vec2::new(-120.0, 0.0),
            speed_multiplier: 1.0,
        }),
    );

    // --- Mud surface ---
    spawn_block(
        &mut commands,
        "Mud Surface",
        Vec2::new(520.0, -100.0),
        Vec2::new(180.0, 30.0),
        Color::srgb(0.40, 0.28, 0.18),
        Some(PlatformerSurfaceModifier {
            friction_multiplier: 2.5,
            surface_velocity: Vec2::ZERO,
            speed_multiplier: 0.5,
        }),
    );

    // Labels
    spawn_label(&mut commands, "NORMAL", Vec2::new(-340.0, -70.0));
    spawn_label(&mut commands, "ICE", Vec2::new(-140.0, -70.0));
    spawn_label(&mut commands, "CONVEYOR →", Vec2::new(80.0, -70.0));
    spawn_label(&mut commands, "← CONVEYOR", Vec2::new(300.0, -70.0));
    spawn_label(&mut commands, "MUD", Vec2::new(520.0, -70.0));

    // On-screen instructions
    commands.spawn((
        Name::new("Instructions"),
        Text::new(
            "A/D: Move  |  Space: Jump\n\
             Walk across different surfaces to feel the physics differences!",
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

fn spawn_block(
    commands: &mut Commands,
    name: &str,
    center: Vec2,
    size: Vec2,
    color: Color,
    surface_modifier: Option<PlatformerSurfaceModifier>,
) {
    let mut entity = commands.spawn((
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
    if let Some(modifier) = surface_modifier {
        entity.insert(modifier);
    }
}

fn spawn_label(commands: &mut Commands, text: &str, position: Vec2) {
    commands.spawn((
        Name::new(format!("Label: {}", text)),
        Text2d::new(text),
        TextFont {
            font_size: 14.0,
            ..default()
        },
        TextColor(Color::srgba(1.0, 1.0, 1.0, 0.6)),
        Transform::from_xyz(position.x, position.y, 5.0),
    ));
}

fn drive_keyboard_intent(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut intent: Single<&mut PlatformerMovementIntent, With<Player>>,
) {
    let left = keyboard.any_pressed([KeyCode::KeyA, KeyCode::ArrowLeft]);
    let right = keyboard.any_pressed([KeyCode::KeyD, KeyCode::ArrowRight]);

    // Continuous state — always overwrite with latest value.
    intent.move_axis = right as i8 as f32 - left as i8 as f32;
    intent.jump_held = keyboard.pressed(KeyCode::Space);

    // One-shot flags — latch on, never overwrite to false.
    if keyboard.just_pressed(KeyCode::Space) {
        intent.jump_pressed = true;
    }
    if keyboard.any_just_pressed([KeyCode::KeyS, KeyCode::ArrowDown]) {
        intent.drop_pressed = true;
    }
}

fn update_pane_monitors(
    player: Single<&PlatformerControllerState, With<Player>>,
    mut pane: ResMut<SurfacePane>,
) {
    pane.phase = format!("{:?}", player.phase);
    pane.velocity_x = player.velocity.x;
    pane.surface = if let Some(modifier) = &player.surface_modifier {
        if modifier.friction_multiplier < 0.3 {
            "Ice".into()
        } else if modifier.surface_velocity.length_squared() > 1.0 {
            format!("Conveyor ({:.0})", modifier.surface_velocity.x)
        } else if modifier.speed_multiplier < 0.8 {
            "Mud".into()
        } else {
            "Custom".into()
        }
    } else {
        "Normal".into()
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

fn tint_player(mut player: Single<(&PlatformerControllerState, &mut Sprite), With<Player>>) {
    player.1.color = match player.0.phase {
        PlatformerMotionPhase::Grounded => Color::srgb(0.94, 0.58, 0.22),
        PlatformerMotionPhase::Rising => Color::srgb(0.98, 0.82, 0.30),
        PlatformerMotionPhase::Apex => Color::srgb(0.86, 0.86, 0.40),
        PlatformerMotionPhase::Falling => Color::srgb(0.84, 0.42, 0.26),
        PlatformerMotionPhase::WallSliding => Color::srgb(0.42, 0.76, 0.96),
        PlatformerMotionPhase::WallClinging => Color::srgb(0.32, 0.56, 0.88),
        PlatformerMotionPhase::Airborne => Color::srgb(0.92, 0.60, 0.30),
    };
}
