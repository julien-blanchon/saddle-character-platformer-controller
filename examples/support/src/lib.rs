use std::env;

use avian2d::prelude::*;
use bevy::{app::AppExit, camera::ScalingMode, prelude::*, window::WindowResolution};
use saddle_character_platformer_controller::{
    PlatformVelocityInheritance, PlatformerControllerBundle, PlatformerControllerConfig,
    PlatformerControllerPlugin, PlatformerControllerState, PlatformerControllerSystems,
    PlatformerDashBundle, PlatformerDashConfig, PlatformerDashIntent, PlatformerDashPlugin,
    PlatformerDashState, PlatformerMotionPhase, PlatformerOneWayPlatform, PlatformerWallSide,
};
use saddle_pane::prelude::*;

pub const PLAYER_SIZE: Vec2 = Vec2::new(18.0, 30.0);

#[derive(Reflect, Clone, Copy, Debug, PartialEq, Eq)]
#[reflect(Debug, PartialEq)]
pub enum DemoScene {
    Basic,
    WallJumps,
    MovingPlatforms,
    OneWayPlatforms,
}

impl DemoScene {
    pub fn title(self) -> &'static str {
        match self {
            Self::Basic => "platformer_controller/basic",
            Self::WallJumps => "platformer_controller/wall_jumps",
            Self::MovingPlatforms => "platformer_controller/moving_platforms",
            Self::OneWayPlatforms => "platformer_controller/one_way_platforms",
        }
    }

    pub fn instructions(self, enhanced_input: bool) -> &'static str {
        let controls = if enhanced_input {
            "WASD/left stick to move, Space/South to jump, S or Down to drop through."
        } else {
            "A/D to move, Space to jump, S or Down to drop through."
        };

        match self {
            Self::Basic => controls,
            Self::WallJumps => {
                "Run into the shaft walls, hold toward them to slide, and jump to kick across."
            }
            Self::MovingPlatforms => {
                "Ride the moving platforms and jump while they carry you to verify velocity inheritance."
            }
            Self::OneWayPlatforms => {
                "Jump through the thin platforms and press S or Down to drop back through them."
            }
        }
    }

    pub fn start_position(self) -> Vec2 {
        match self {
            Self::Basic => Vec2::new(-270.0, -40.0),
            Self::WallJumps => Vec2::new(0.0, -70.0),
            Self::MovingPlatforms => Vec2::new(-260.0, -50.0),
            Self::OneWayPlatforms => Vec2::new(-260.0, -20.0),
        }
    }

    pub fn viewport_height(self) -> f32 {
        match self {
            Self::Basic => 320.0,
            Self::WallJumps => 280.0,
            Self::MovingPlatforms => 320.0,
            Self::OneWayPlatforms => 320.0,
        }
    }

    pub fn controller_config(self) -> PlatformerControllerConfig {
        let mut config = PlatformerControllerConfig::default();

        match self {
            Self::Basic => {
                config.movement.max_speed = 240.0;
                config.jump.height = 88.0;
                config.jump.time_to_apex = 0.4;
                config.jump.coyote_time = 0.11;
                config.jump.jump_buffer_time = 0.12;
                config.jump.max_air_jumps = 1;
            }
            Self::WallJumps => {
                config.movement.max_speed = 230.0;
                config.jump.height = 92.0;
                config.jump.time_to_apex = 0.38;
                config.jump.max_air_jumps = 0;
                config.walls.wall_slide_terminal_speed = 96.0;
                config.walls.wall_slide_gravity_multiplier = 0.42;
                config.walls.wall_jump_horizontal_speed = 250.0;
                config.walls.wall_jump_vertical_speed = 305.0;
                config.walls.wall_jump_steering_lock_time = 0.16;
                config.walls.wall_jump_steering_factor = 0.08;
            }
            Self::MovingPlatforms => {
                config.jump.height = 86.0;
                config.jump.time_to_apex = 0.43;
                config.platforms.velocity_inheritance = PlatformVelocityInheritance::Full;
            }
            Self::OneWayPlatforms => {
                config.jump.height = 84.0;
                config.jump.time_to_apex = 0.42;
                config.platforms.drop_through_duration = 0.22;
            }
        }

        config
    }
}

#[derive(Component, Reflect, Debug)]
#[reflect(Component, Debug)]
pub struct DemoPlayer;

#[derive(Component, Reflect, Debug)]
#[reflect(Component, Debug)]
pub struct DemoCamera {
    pub smoothing: f32,
}

#[derive(Component, Reflect, Debug)]
#[reflect(Component, Debug)]
pub struct DemoOverlay;

#[derive(Component, Reflect, Debug)]
#[reflect(Component, Debug)]
pub struct MovingPlatform {
    pub origin: Vec2,
    pub axis: Vec2,
    pub amplitude: f32,
    pub speed: f32,
    pub phase: f32,
}

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DemoFixedSystems {
    DriveIntent,
    AnimatePlatforms,
}

#[derive(Resource, Reflect, Clone, Copy, Debug)]
#[reflect(Resource, Debug)]
pub struct DemoState {
    pub scene: DemoScene,
    pub enhanced_input: bool,
}

#[derive(Resource, Debug, Clone, PartialEq, Pane)]
#[pane(title = "Platformer", position = "top-right")]
pub struct ExamplePlatformerPane {
    #[pane(slider, min = 120.0, max = 360.0, step = 1.0)]
    pub max_speed: f32,
    #[pane(slider, min = 40.0, max = 140.0, step = 1.0)]
    pub jump_height: f32,
    #[pane(slider, min = 0.2, max = 0.8, step = 0.01)]
    pub time_to_apex: f32,
    #[pane(slider, min = 0.0, max = 0.25, step = 0.01)]
    pub coyote_time: f32,
    #[pane(slider, min = 0.0, max = 0.25, step = 0.01)]
    pub jump_buffer_time: f32,
    #[pane(slider, min = 0.0, max = 140.0, step = 1.0)]
    pub dash_distance: f32,
    #[pane(slider, min = 0.05, max = 0.4, step = 0.01)]
    pub dash_duration: f32,
    #[pane(slider, min = 0.0, max = 0.5, step = 0.01)]
    pub dash_cooldown: f32,
    pub allow_ground_dash: bool,
    #[pane(slider, min = 2.0, max = 16.0, step = 0.25)]
    pub camera_smoothing: f32,
    #[pane(monitor)]
    pub player_x: f32,
    #[pane(monitor)]
    pub player_y: f32,
    #[pane(monitor)]
    pub velocity_x: f32,
    #[pane(monitor)]
    pub velocity_y: f32,
    #[pane(monitor)]
    pub grounded: bool,
    #[pane(monitor)]
    pub remaining_air_jumps: f32,
    #[pane(monitor)]
    pub phase: String,
}

impl Default for ExamplePlatformerPane {
    fn default() -> Self {
        Self {
            max_speed: 220.0,
            jump_height: 78.0,
            time_to_apex: 0.42,
            coyote_time: 0.1,
            jump_buffer_time: 0.12,
            dash_distance: 84.0,
            dash_duration: 0.16,
            dash_cooldown: 0.12,
            allow_ground_dash: true,
            camera_smoothing: 8.0,
            player_x: 0.0,
            player_y: 0.0,
            velocity_x: 0.0,
            velocity_y: 0.0,
            grounded: false,
            remaining_air_jumps: 0.0,
            phase: "Grounded".to_string(),
        }
    }
}

impl ExamplePlatformerPane {
    fn from_scene(scene: DemoScene) -> Self {
        let config = scene.controller_config();
        Self {
            max_speed: config.movement.max_speed,
            jump_height: config.jump.height,
            time_to_apex: config.jump.time_to_apex,
            coyote_time: config.jump.coyote_time,
            jump_buffer_time: config.jump.jump_buffer_time,
            dash_distance: PlatformerDashConfig::default().distance,
            dash_duration: PlatformerDashConfig::default().duration,
            dash_cooldown: PlatformerDashConfig::default().cooldown,
            allow_ground_dash: PlatformerDashConfig::default().allow_ground_dash,
            camera_smoothing: 8.0,
            ..Self::default()
        }
    }
}

#[derive(Resource, Reflect, Default, Debug, Clone)]
#[reflect(Resource, Debug, Default)]
pub struct DemoDiagnostics {
    pub player_entity: Option<Entity>,
    pub player_position: Vec2,
    pub player_velocity: Vec2,
    pub support_velocity: Vec2,
    pub support_entity: Option<Entity>,
    pub grounded: bool,
    pub phase: PlatformerMotionPhase,
    pub remaining_air_jumps: u32,
    pub buffered_jump: bool,
    pub coyote_time_remaining: f32,
    pub jump_buffer_remaining: f32,
    pub wall_jump_lock_remaining: f32,
    pub wall_side: Option<PlatformerWallSide>,
    pub controller_state: Option<PlatformerControllerState>,
    pub dash_state: Option<PlatformerDashState>,
    pub overlay_text: String,
}

#[derive(Resource)]
struct AutoExitTimer(Timer);

#[derive(Resource, Clone, Copy)]
struct ExamplePlatformerPaneBootstrap(DemoScene);

pub fn configure_demo_app(app: &mut App, scene: DemoScene, enhanced_input: bool) {
    app.insert_resource(ClearColor(Color::srgb(0.06, 0.07, 0.09)));
    app.insert_resource(Time::<Fixed>::from_hz(60.0));
    app.insert_resource(DemoState {
        scene,
        enhanced_input,
    });
    app.insert_resource(DemoDiagnostics::default());
    app.insert_resource(ExamplePlatformerPaneBootstrap(scene));
    app.register_type::<DemoScene>()
        .register_type::<DemoCamera>()
        .register_type::<DemoDiagnostics>()
        .register_type::<DemoOverlay>()
        .register_type::<DemoPlayer>()
        .register_type::<DemoState>()
        .register_type::<MovingPlatform>();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: scene.title().into(),
            resolution: WindowResolution::new(1440, 900),
            ..default()
        }),
        ..default()
    }));
    app.add_plugins((
        PhysicsPlugins::default().with_length_unit(20.0),
        PlatformerControllerPlugin::always_on(FixedUpdate),
        PlatformerDashPlugin::always_on(FixedUpdate),
    ));
    app.configure_sets(
        FixedUpdate,
        DemoFixedSystems::DriveIntent.before(PlatformerControllerSystems::ReadIntent),
    );
    app.configure_sets(
        FixedUpdate,
        DemoFixedSystems::AnimatePlatforms.before(PlatformerControllerSystems::SenseContacts),
    );
    app.add_systems(Startup, setup_scene);
    app.add_systems(
        FixedUpdate,
        animate_platforms.in_set(DemoFixedSystems::AnimatePlatforms),
    );
    app.add_systems(Update, (update_diagnostics, update_overlay).chain());
    app.add_systems(Update, auto_exit_if_requested);
    app.add_systems(PostUpdate, (follow_camera, tint_player));

    if let Ok(seconds) = env::var("PLATFORMER_CONTROLLER_AUTO_EXIT_SECS") {
        if let Ok(seconds) = seconds.parse::<f32>() {
            app.insert_resource(AutoExitTimer(Timer::from_seconds(seconds, TimerMode::Once)));
        }
    }
}

pub fn install_pane(app: &mut App) {
    if !app.is_plugin_added::<PanePlugin>() {
        app.add_plugins((
            bevy_flair::FlairPlugin,
            bevy_input_focus::InputDispatchPlugin,
            bevy_ui_widgets::UiWidgetsPlugins,
            bevy_input_focus::tab_navigation::TabNavigationPlugin,
            PanePlugin,
        ));
    }

    app.register_pane::<ExamplePlatformerPane>().add_systems(
        Update,
        (
            apply_bootstrapped_pane,
            sync_platformer_pane,
            update_platformer_pane_monitors,
        )
            .chain(),
    );
}

pub fn drive_keyboard_intent(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut movement_intents: Query<
        &mut saddle_character_platformer_controller::PlatformerMovementIntent,
        With<DemoPlayer>,
    >,
    mut dash_intents: Query<&mut PlatformerDashIntent, With<DemoPlayer>>,
) {
    let left = keyboard.any_pressed([KeyCode::KeyA, KeyCode::ArrowLeft]);
    let right = keyboard.any_pressed([KeyCode::KeyD, KeyCode::ArrowRight]);

    if let Ok(mut intent) = movement_intents.single_mut() {
        intent.move_axis = right as i8 as f32 - left as i8 as f32;
        intent.jump_pressed = keyboard.just_pressed(KeyCode::Space);
        intent.jump_held = keyboard.pressed(KeyCode::Space);
        intent.drop_pressed = keyboard.any_just_pressed([KeyCode::KeyS, KeyCode::ArrowDown]);
    }

    if let Ok(mut intent) = dash_intents.single_mut() {
        intent.pressed = keyboard.any_just_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
        intent.direction = Vec2::ZERO;
    }
}

fn apply_bootstrapped_pane(
    bootstrap: Option<Res<ExamplePlatformerPaneBootstrap>>,
    mut pane: ResMut<ExamplePlatformerPane>,
) {
    let Some(bootstrap) = bootstrap else {
        return;
    };

    if *pane == ExamplePlatformerPane::default() {
        *pane = ExamplePlatformerPane::from_scene(bootstrap.0);
    }
}

fn sync_platformer_pane(
    pane: Res<ExamplePlatformerPane>,
    mut controllers: Query<&mut PlatformerControllerConfig, With<DemoPlayer>>,
    mut dash_configs: Query<&mut PlatformerDashConfig, With<DemoPlayer>>,
    mut cameras: Query<&mut DemoCamera>,
) {
    for mut config in &mut controllers {
        config.movement.max_speed = pane.max_speed.max(0.0);
        config.jump.height = pane.jump_height.max(0.0);
        config.jump.time_to_apex = pane.time_to_apex.max(0.01);
        config.jump.coyote_time = pane.coyote_time.max(0.0);
        config.jump.jump_buffer_time = pane.jump_buffer_time.max(0.0);
    }

    for mut config in &mut dash_configs {
        config.distance = pane.dash_distance.max(0.0);
        config.duration = pane.dash_duration.max(0.01);
        config.cooldown = pane.dash_cooldown.max(0.0);
        config.allow_ground_dash = pane.allow_ground_dash;
    }

    for mut camera in &mut cameras {
        camera.smoothing = pane.camera_smoothing.max(0.1);
    }
}

fn update_platformer_pane_monitors(
    diagnostics: Res<DemoDiagnostics>,
    mut pane: ResMut<ExamplePlatformerPane>,
) {
    pane.player_x = diagnostics.player_position.x;
    pane.player_y = diagnostics.player_position.y;
    pane.velocity_x = diagnostics.player_velocity.x;
    pane.velocity_y = diagnostics.player_velocity.y;
    pane.grounded = diagnostics.grounded;
    pane.remaining_air_jumps = diagnostics.remaining_air_jumps as f32;
    pane.phase = display_phase(&diagnostics);
}

fn setup_scene(
    mut commands: Commands,
    demo: Res<DemoState>,
    mut diagnostics: ResMut<DemoDiagnostics>,
) {
    commands.spawn((
        Name::new("Demo Camera"),
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: demo.scene.viewport_height(),
            },
            ..OrthographicProjection::default_2d()
        }),
        Transform::from_xyz(0.0, 0.0, 999.0),
        DemoCamera { smoothing: 8.0 },
    ));

    commands.spawn((
        Name::new("Demo Overlay"),
        DemoOverlay,
        Text::new(String::new()),
        Node {
            position_type: PositionType::Absolute,
            left: px(18.0),
            top: px(16.0),
            width: px(620.0),
            padding: UiRect::axes(px(12.0), px(10.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.04, 0.05, 0.08, 0.82)),
        TextFont {
            font_size: 18.0,
            ..default()
        },
        TextColor(Color::WHITE),
    ));

    diagnostics.player_entity = Some(spawn_player(&mut commands, demo.scene));

    match demo.scene {
        DemoScene::Basic => spawn_basic_scene(&mut commands),
        DemoScene::WallJumps => spawn_wall_jump_scene(&mut commands),
        DemoScene::MovingPlatforms => spawn_moving_platform_scene(&mut commands),
        DemoScene::OneWayPlatforms => spawn_one_way_scene(&mut commands),
    }
}

fn spawn_player(commands: &mut Commands, scene: DemoScene) -> Entity {
    let position = scene.start_position();
    commands
        .spawn((
            Name::new("Demo Player"),
            DemoPlayer,
            Sprite {
                color: Color::srgb(0.94, 0.58, 0.22),
                custom_size: Some(PLAYER_SIZE),
                ..default()
            },
            PlatformerControllerBundle::with_config(
                Collider::rectangle(PLAYER_SIZE.x, PLAYER_SIZE.y),
                scene.controller_config(),
            )
            .with_transform(Transform::from_xyz(position.x, position.y, 10.0)),
            PlatformerDashBundle::default(),
        ))
        .id()
}

fn spawn_basic_scene(commands: &mut Commands) {
    spawn_block(
        commands,
        "Ground",
        Vec2::new(0.0, -150.0),
        Vec2::new(920.0, 38.0),
        Color::srgb(0.20, 0.22, 0.26),
    );
    spawn_block(
        commands,
        "Step",
        Vec2::new(-85.0, -86.0),
        Vec2::new(110.0, 28.0),
        Color::srgb(0.30, 0.36, 0.43),
    );
    spawn_block(
        commands,
        "Tower",
        Vec2::new(175.0, -28.0),
        Vec2::new(80.0, 160.0),
        Color::srgb(0.33, 0.40, 0.49),
    );
    spawn_slope(
        commands,
        "Ramp",
        Vec2::new(5.0, -118.0),
        Vec2::new(200.0, 20.0),
        0.32,
        Color::srgb(0.28, 0.48, 0.42),
    );
}

fn spawn_wall_jump_scene(commands: &mut Commands) {
    spawn_block(
        commands,
        "Ground",
        Vec2::new(0.0, -150.0),
        Vec2::new(520.0, 38.0),
        Color::srgb(0.20, 0.22, 0.26),
    );
    spawn_block(
        commands,
        "Left Wall",
        Vec2::new(-120.0, 30.0),
        Vec2::new(40.0, 360.0),
        Color::srgb(0.25, 0.30, 0.36),
    );
    spawn_block(
        commands,
        "Right Wall",
        Vec2::new(120.0, 30.0),
        Vec2::new(40.0, 360.0),
        Color::srgb(0.25, 0.30, 0.36),
    );
    spawn_block(
        commands,
        "Left Ledge",
        Vec2::new(-70.0, -5.0),
        Vec2::new(62.0, 18.0),
        Color::srgb(0.43, 0.34, 0.30),
    );
    spawn_block(
        commands,
        "Right Ledge",
        Vec2::new(70.0, 85.0),
        Vec2::new(62.0, 18.0),
        Color::srgb(0.43, 0.34, 0.30),
    );
    spawn_block(
        commands,
        "Finish Ledge",
        Vec2::new(0.0, 170.0),
        Vec2::new(150.0, 20.0),
        Color::srgb(0.38, 0.52, 0.40),
    );
}

fn spawn_moving_platform_scene(commands: &mut Commands) {
    spawn_block(
        commands,
        "Ground",
        Vec2::new(0.0, -150.0),
        Vec2::new(920.0, 38.0),
        Color::srgb(0.20, 0.22, 0.26),
    );
    spawn_moving_platform(
        commands,
        "Horizontal Platform",
        Vec2::new(-40.0, -20.0),
        Vec2::X,
        130.0,
        0.7,
        Vec2::new(120.0, 18.0),
        Color::srgb(0.76, 0.56, 0.20),
    );
    spawn_moving_platform(
        commands,
        "Vertical Platform",
        Vec2::new(170.0, -50.0),
        Vec2::Y,
        95.0,
        0.9,
        Vec2::new(100.0, 18.0),
        Color::srgb(0.32, 0.58, 0.74),
    );
    spawn_block(
        commands,
        "Landing Deck",
        Vec2::new(330.0, 80.0),
        Vec2::new(180.0, 20.0),
        Color::srgb(0.28, 0.40, 0.48),
    );
}

fn spawn_one_way_scene(commands: &mut Commands) {
    spawn_block(
        commands,
        "Ground",
        Vec2::new(0.0, -150.0),
        Vec2::new(920.0, 38.0),
        Color::srgb(0.20, 0.22, 0.26),
    );
    spawn_one_way_platform(
        commands,
        "One Way A",
        Vec2::new(-110.0, -60.0),
        Vec2::new(170.0, 12.0),
        Color::srgba(0.72, 0.82, 0.95, 0.35),
    );
    spawn_one_way_platform(
        commands,
        "One Way B",
        Vec2::new(40.0, 15.0),
        Vec2::new(170.0, 12.0),
        Color::srgba(0.72, 0.82, 0.95, 0.35),
    );
    spawn_one_way_platform(
        commands,
        "One Way C",
        Vec2::new(200.0, 90.0),
        Vec2::new(170.0, 12.0),
        Color::srgba(0.72, 0.82, 0.95, 0.35),
    );
    spawn_block(
        commands,
        "Backdrop Tower",
        Vec2::new(290.0, -8.0),
        Vec2::new(90.0, 160.0),
        Color::srgb(0.26, 0.28, 0.34),
    );
}

pub fn animate_platforms(
    time: Res<Time>,
    mut platforms: Query<(
        &MovingPlatform,
        &mut Position,
        &mut LinearVelocity,
        &mut Transform,
    )>,
) {
    let delta_secs = time.delta_secs().max(f32::EPSILON);

    for (platform, mut position, mut velocity, mut transform) in &mut platforms {
        let progress = time.elapsed_secs() * platform.speed + platform.phase;
        let next_position = platform.origin
            + platform.axis.normalize_or_zero() * platform.amplitude * progress.sin();
        velocity.0 = (next_position - position.0) / delta_secs;
        position.0 = next_position;
        transform.translation.x = next_position.x;
        transform.translation.y = next_position.y;
    }
}

fn follow_camera(
    time: Res<Time>,
    player: Single<&Transform, With<DemoPlayer>>,
    mut camera: Single<(&DemoCamera, &mut Transform), (With<DemoCamera>, Without<DemoPlayer>)>,
) {
    let target = player.translation.xy();
    let camera_height = target.y.max(-30.0);
    let desired = Vec3::new(target.x, camera_height, camera.1.translation.z);
    let blend = 1.0 - (-camera.0.smoothing * time.delta_secs()).exp();
    camera.1.translation = camera.1.translation.lerp(desired, blend);
}

fn tint_player(
    mut player: Single<
        (&PlatformerControllerState, Option<&PlatformerDashState>, &mut Sprite),
        With<DemoPlayer>,
    >,
) {
    player.2.color = if player.1.is_some_and(|dash| dash.active) {
        Color::srgb(0.98, 0.28, 0.48)
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

fn update_overlay(
    demo: Res<DemoState>,
    mut diagnostics: ResMut<DemoDiagnostics>,
    mut overlay: Single<&mut Text, With<DemoOverlay>>,
) {
    overlay.0 = format!(
        "{}\nphase: {}\ngrounded: {}\nvelocity: [{:.1}, {:.1}]\nsupport: {} @ [{:.1}, {:.1}]\ncoyote/buffer/lock: {:.2} / {:.2} / {:.2}\nair jumps left: {}\ndash charges: {}\nbuffered jump: {}\nwall: {:?}",
        demo.scene.instructions(demo.enhanced_input),
        display_phase(&diagnostics),
        diagnostics.grounded,
        diagnostics.player_velocity.x,
        diagnostics.player_velocity.y,
        format_entity(diagnostics.support_entity),
        diagnostics.support_velocity.x,
        diagnostics.support_velocity.y,
        diagnostics.coyote_time_remaining,
        diagnostics.jump_buffer_remaining,
        diagnostics.wall_jump_lock_remaining,
        diagnostics.remaining_air_jumps,
        diagnostics
            .dash_state
            .as_ref()
            .map_or(0, |dash| dash.remaining_charges),
        diagnostics.buffered_jump,
        diagnostics.wall_side,
    );
    diagnostics.overlay_text = overlay.0.clone();
}

fn update_diagnostics(
    player: Single<
        (&Transform, &PlatformerControllerState, Option<&PlatformerDashState>),
        With<DemoPlayer>,
    >,
    mut diagnostics: ResMut<DemoDiagnostics>,
) {
    diagnostics.player_position = player.0.translation.xy();
    diagnostics.player_velocity = player.1.velocity;
    diagnostics.support_velocity = player.1.support_velocity;
    diagnostics.support_entity = player.1.support_entity;
    diagnostics.grounded = player.1.is_grounded;
    diagnostics.phase = player.1.phase;
    diagnostics.remaining_air_jumps = player.1.remaining_air_jumps;
    diagnostics.buffered_jump = player.1.buffered_jump;
    diagnostics.coyote_time_remaining = player.1.coyote_time_remaining;
    diagnostics.jump_buffer_remaining = player.1.jump_buffer_remaining;
    diagnostics.wall_jump_lock_remaining = player.1.wall_jump_lock_remaining;
    diagnostics.wall_side = player.1.wall.as_ref().map(|wall| wall.side);
    diagnostics.controller_state = Some(player.1.clone());
    diagnostics.dash_state = player.2.cloned();
}

fn display_phase(diagnostics: &DemoDiagnostics) -> String {
    if diagnostics.dash_state.as_ref().is_some_and(|dash| dash.active) {
        "Dashing".to_string()
    } else {
        format!("{:?}", diagnostics.phase)
    }
}

fn auto_exit_if_requested(
    time: Res<Time>,
    timer: Option<ResMut<AutoExitTimer>>,
    mut exit: MessageWriter<AppExit>,
) {
    let Some(mut timer) = timer else {
        return;
    };

    if timer.0.tick(time.delta()).just_finished() {
        exit.write(AppExit::Success);
    }
}

fn format_entity(entity: Option<Entity>) -> String {
    entity
        .map(|entity| entity.to_bits().to_string())
        .unwrap_or_else(|| "None".to_string())
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

fn spawn_slope(
    commands: &mut Commands,
    name: &str,
    center: Vec2,
    size: Vec2,
    angle_radians: f32,
    color: Color,
) {
    commands.spawn((
        Name::new(name.to_string()),
        Sprite {
            color,
            custom_size: Some(size),
            ..default()
        },
        Transform::from_xyz(center.x, center.y, 0.0)
            .with_rotation(Quat::from_rotation_z(angle_radians)),
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
            phase: 0.0,
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

fn spawn_one_way_platform(
    commands: &mut Commands,
    name: &str,
    center: Vec2,
    size: Vec2,
    color: Color,
) {
    commands.spawn((
        Name::new(name.to_string()),
        PlatformerOneWayPlatform,
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
