use avian2d::prelude::*;
use bevy::{
    app::AppExit,
    asset::RenderAssetUsages,
    image::{ImageAddressMode, ImageFilterMode, ImageSampler, ImageSamplerDescriptor},
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
    window::WindowResolution,
};
use saddle_animation_spritesheet::{
    AnimationController, AnimationLibrary, AnimationTarget, SpritesheetAnimationBundle,
    SpritesheetAnimator, SpritesheetPlugin,
};
use saddle_camera_pixel_camera::{
    PixelCamera, PixelCameraInner, PixelCameraPlugin, PixelCameraTransform, PixelSnap,
};
use saddle_character_platformer_controller::{
    DashStarted, PlatformerControllerBundle, PlatformerControllerConfig, PlatformerDashBundle,
    PlatformerDashConfig, PlatformerDashPlugin, PlatformerDashState,
    PlatformerControllerPlugin, PlatformerControllerState,
    PlatformerMotionPhase,
};
use saddle_character_platformer_controller_example_support as platformer_support;
use saddle_pane::prelude::*;
use saddle_rendering_parallax_scroller::{
    ParallaxAxes, ParallaxCameraTarget, ParallaxLayer, ParallaxLayerBundle, ParallaxLayerStrategy,
    ParallaxRigBundle, ParallaxScrollerPlugin, ParallaxSegmented,
};
use saddle_rendering_sprite_effects::{
    FlashConfig, FlashEffect, OutlineConfig, OutlineEffect, SilhouetteConfig, SilhouetteEffect,
    SpriteEffectsPlugin, SquashStretchConfig, SquashStretchEffect,
};

const PLAYER_SPRITE_SCALE: f32 = 1.0;
const FRAME_SIZE: UVec2 = UVec2::new(24, 24);
const ASEPRITE_JSON: &str = r#"
{
  "frames": {
    "frame_0": { "frame": { "x": 0, "y": 0, "w": 24, "h": 24 }, "duration": 160 },
    "frame_1": { "frame": { "x": 24, "y": 0, "w": 24, "h": 24 }, "duration": 160 },
    "frame_2": { "frame": { "x": 48, "y": 0, "w": 24, "h": 24 }, "duration": 90 },
    "frame_3": { "frame": { "x": 72, "y": 0, "w": 24, "h": 24 }, "duration": 90 },
    "frame_4": { "frame": { "x": 96, "y": 0, "w": 24, "h": 24 }, "duration": 90 },
    "frame_5": { "frame": { "x": 120, "y": 0, "w": 24, "h": 24 }, "duration": 120 },
    "frame_6": { "frame": { "x": 144, "y": 0, "w": 24, "h": 24 }, "duration": 120 },
    "frame_7": { "frame": { "x": 168, "y": 0, "w": 24, "h": 24 }, "duration": 80 }
  },
  "meta": {
    "size": { "w": 192, "h": 24 },
    "frameTags": [
      { "name": "idle", "from": 0, "to": 1, "direction": "pingpong" },
      { "name": "run", "from": 2, "to": 4, "direction": "forward" },
      { "name": "rise", "from": 5, "to": 5, "direction": "forward" },
      { "name": "fall", "from": 6, "to": 6, "direction": "forward" },
      { "name": "dash", "from": 7, "to": 7, "direction": "forward" }
    ]
  }
}
"#;

#[derive(Component)]
struct DemoPlayerSprite;

#[derive(Component)]
struct CameraRoot;

#[derive(Component)]
struct OverlayText;

#[derive(Clone)]
struct DemoAtlas {
    image: Handle<Image>,
    layout: Handle<TextureAtlasLayout>,
}

#[derive(Resource)]
struct DemoAssets {
    parallax: DemoTextures,
}

struct DemoTextures {
    sky: Handle<Image>,
    mountains: Handle<Image>,
    canopy: Handle<Image>,
}

#[derive(Resource, Default)]
struct ParallaxSpawned(bool);

#[derive(Resource)]
struct AutoExitAfter(Timer);

#[derive(Resource, Debug, Clone, PartialEq, Pane)]
#[pane(title = "Full Demo", position = "top-right")]
struct FullDemoPane {
    #[pane(slider, min = 48.0, max = 140.0, step = 1.0)]
    jump_height: f32,
    #[pane(slider, min = 0.2, max = 0.7, step = 0.01)]
    time_to_apex: f32,
    #[pane(slider, min = 0.0, max = 140.0, step = 1.0)]
    dash_distance: f32,
    #[pane(slider, min = 0.05, max = 0.35, step = 0.01)]
    dash_duration: f32,
    #[pane(slider, min = 1.0, max = 6.0, step = 1.0)]
    pixel_zoom: f32,
    #[pane(slider, min = 2.0, max = 18.0, step = 0.25)]
    follow_smoothing: f32,
    #[pane(slider, min = 0.2, max = 1.4, step = 0.01)]
    mountain_factor: f32,
    #[pane(slider, min = 0.2, max = 1.4, step = 0.01)]
    canopy_factor: f32,
    #[pane(slider, min = 0.4, max = 2.0, step = 0.05)]
    spritesheet_speed: f32,
    #[pane(slider, min = 0.0, max = 4.0, step = 0.1)]
    outline_width: f32,
    #[pane(slider, min = 0.0, max = 1.0, step = 0.01)]
    silhouette_tint: f32,
    #[pane(monitor)]
    phase: String,
    #[pane(monitor)]
    player_x: f32,
    #[pane(monitor)]
    velocity_x: f32,
}

impl Default for FullDemoPane {
    fn default() -> Self {
        Self {
            jump_height: 92.0,
            time_to_apex: 0.38,
            dash_distance: 92.0,
            dash_duration: 0.16,
            pixel_zoom: 3.0,
            follow_smoothing: 9.0,
            mountain_factor: 0.86,
            canopy_factor: 1.08,
            spritesheet_speed: 1.0,
            outline_width: 1.4,
            silhouette_tint: 0.8,
            phase: "Grounded".to_string(),
            player_x: 0.0,
            velocity_x: 0.0,
        }
    }
}

fn main() -> AppExit {
    let mut app = App::new();
    app.insert_resource(ClearColor(Color::srgb(0.07, 0.08, 0.10)));
    app.insert_resource(Time::<Fixed>::from_hz(60.0));
    app.insert_resource(ParallaxSpawned::default());
    app.insert_resource(FullDemoPane::default());
    app.add_plugins(
        DefaultPlugins
            .set(ImagePlugin::default_nearest())
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "platformer_controller/full_demo".into(),
                    resolution: WindowResolution::new(1440, 900),
                    ..default()
                }),
                ..default()
            }),
    );
    app.add_plugins((
        PhysicsPlugins::default().with_length_unit(20.0),
        PlatformerControllerPlugin::always_on(FixedUpdate),
        PlatformerDashPlugin::always_on(FixedUpdate),
        SpritesheetPlugin::default(),
        SpriteEffectsPlugin::default(),
        ParallaxScrollerPlugin::default(),
        PixelCameraPlugin::default(),
        bevy_flair::FlairPlugin,
        bevy_input_focus::InputDispatchPlugin,
        bevy_ui_widgets::UiWidgetsPlugins,
        bevy_input_focus::tab_navigation::TabNavigationPlugin,
        PanePlugin,
    ));
    app.register_pane::<FullDemoPane>();
    app.add_systems(Startup, setup);
    app.add_systems(Update, platformer_support::drive_keyboard_intent);
    app.add_systems(
        Update,
        (
            maybe_spawn_parallax,
            tag_new_parallax_layers,
            sync_demo_pane,
            follow_player_camera,
            drive_player_animation,
            pulse_player_on_dash,
            update_overlay,
            auto_exit_after,
        ),
    );
    maybe_install_auto_exit(&mut app);
    app.run()
}

fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut libraries: ResMut<Assets<AnimationLibrary>>,
) {
    let parallax = demo_textures(&mut images);
    commands.insert_resource(DemoAssets { parallax });

    let atlas = make_demo_atlas(&mut images, &mut layouts);
    let library = libraries.add(
        AnimationLibrary::from_aseprite_json("full_demo", ASEPRITE_JSON)
            .expect("embedded aseprite library should parse"),
    );

    let camera = PixelCamera::new(480, 300);
    commands.spawn((
        Name::new("Pixel Camera Root"),
        CameraRoot,
        camera,
        PixelCameraTransform {
            logical_position: Vec2::new(-110.0, -40.0),
        },
    ));

    commands.spawn((
        Name::new("Overlay"),
        OverlayText,
        Node {
            position_type: PositionType::Absolute,
            left: px(18.0),
            top: px(18.0),
            width: px(420.0),
            padding: UiRect::all(px(12.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.04, 0.05, 0.08, 0.78)),
        Text::default(),
        TextFont {
            font_size: 17.0,
            ..default()
        },
        TextColor(Color::WHITE),
    ));

    spawn_level(&mut commands);
    spawn_player(&mut commands, atlas, library);
}

fn spawn_level(commands: &mut Commands) {
    spawn_block(
        commands,
        "Ground",
        Vec2::new(120.0, -82.0),
        Vec2::new(980.0, 36.0),
        Color::srgb(0.15, 0.16, 0.18),
    );
    spawn_block(
        commands,
        "Left Step",
        Vec2::new(-150.0, -34.0),
        Vec2::new(100.0, 22.0),
        Color::srgb(0.20, 0.24, 0.28),
    );
    spawn_block(
        commands,
        "Middle Platform",
        Vec2::new(40.0, -6.0),
        Vec2::new(124.0, 18.0),
        Color::srgb(0.24, 0.28, 0.30),
    );
    spawn_block(
        commands,
        "High Platform",
        Vec2::new(212.0, 28.0),
        Vec2::new(120.0, 18.0),
        Color::srgb(0.20, 0.25, 0.22),
    );
    commands.spawn((
        Name::new("Foreground Ruin"),
        Sprite::from_color(Color::srgb(0.08, 0.10, 0.12), Vec2::new(120.0, 180.0)),
        Transform::from_xyz(270.0, -10.0, 3.0),
        saddle_camera_pixel_camera::PIXEL_LAYERS.clone(),
    ));
    commands.spawn((
        Name::new("Lantern Glow"),
        Sprite::from_color(Color::srgba(1.0, 0.76, 0.34, 0.28), Vec2::new(54.0, 54.0)),
        Transform::from_xyz(40.0, 28.0, 1.5),
        saddle_camera_pixel_camera::PIXEL_LAYERS.clone(),
    ));
}

fn spawn_player(commands: &mut Commands, atlas: DemoAtlas, library: Handle<AnimationLibrary>) {
    let mut config = platformer_support::DemoScene::Basic.controller_config();
    config.jump.height = 92.0;
    config.jump.time_to_apex = 0.38;
    let dash_config = PlatformerDashConfig {
        distance: 92.0,
        duration: 0.16,
        cooldown: 0.12,
        ..default()
    };

    commands.spawn((
        Name::new("Demo Player"),
        platformer_support::DemoPlayer,
        DemoPlayerSprite,
        Sprite::from_atlas_image(
            atlas.image,
            TextureAtlas {
                layout: atlas.layout,
                index: 0,
            },
        ),
        PlatformerControllerBundle::with_config(
            Collider::rectangle(
                platformer_support::PLAYER_SIZE.x,
                platformer_support::PLAYER_SIZE.y,
            ),
            config,
        )
        .with_transform(Transform::from_xyz(-220.0, -44.0, 1.0)),
        PlatformerDashBundle::with_config(dash_config),
        SpritesheetAnimationBundle::new(library, AnimationTarget::state("idle")),
        OutlineEffect::new(OutlineConfig {
            color: Color::srgba(0.04, 0.05, 0.06, 1.0),
            width_pixels: 1.4,
            alpha_threshold: 0.08,
        }),
        SilhouetteEffect::new(SilhouetteConfig {
            color: Color::srgba(0.26, 0.86, 1.0, 0.88),
            tint_strength: 0.8,
            alpha_threshold: 0.08,
            sort_offset: 2.6,
        }),
        PixelSnap,
        saddle_camera_pixel_camera::PIXEL_LAYERS.clone(),
    ));
}

fn spawn_block(commands: &mut Commands, name: &str, center: Vec2, size: Vec2, color: Color) {
    commands.spawn((
        Name::new(name.to_string()),
        Sprite::from_color(color, size),
        Transform::from_xyz(center.x, center.y, 0.5),
        RigidBody::Static,
        Collider::rectangle(size.x, size.y),
        saddle_camera_pixel_camera::PIXEL_LAYERS.clone(),
    ));
}

fn maybe_spawn_parallax(
    mut commands: Commands,
    assets: Res<DemoAssets>,
    mut spawned: ResMut<ParallaxSpawned>,
    inner_cameras: Query<(Entity, &PixelCameraInner)>,
    roots: Query<Entity, With<CameraRoot>>,
) {
    if spawned.0 {
        return;
    }

    let Ok(root) = roots.single() else {
        return;
    };

    let Some(inner_camera) = inner_cameras
        .iter()
        .find_map(|(entity, inner)| (inner.root == root).then_some(entity))
    else {
        return;
    };

    let rig = commands
        .spawn((
            Name::new("Full Demo Parallax"),
            ParallaxRigBundle {
                transform: Transform::from_translation(Vec3::new(0.0, 18.0, -40.0)),
                ..default()
            },
            ParallaxCameraTarget::new(inner_camera),
        ))
        .id();
    add_forest_stack(&mut commands, rig, &assets.parallax);
    spawned.0 = true;
}

fn tag_new_parallax_layers(mut commands: Commands, layers: Query<Entity, Added<ParallaxLayer>>) {
    for entity in &layers {
        commands
            .entity(entity)
            .insert(saddle_camera_pixel_camera::PIXEL_LAYERS.clone());
    }
}

fn sync_demo_pane(
    mut pane: ResMut<FullDemoPane>,
    mut controllers: Query<&mut PlatformerControllerConfig, With<platformer_support::DemoPlayer>>,
    mut dash_configs: Query<&mut PlatformerDashConfig, With<platformer_support::DemoPlayer>>,
    mut pixel_camera: Query<&mut PixelCamera, With<CameraRoot>>,
    mut animators: Query<&mut SpritesheetAnimator, With<DemoPlayerSprite>>,
    mut outlines: Query<&mut OutlineEffect, With<DemoPlayerSprite>>,
    mut silhouettes: Query<&mut SilhouetteEffect, With<DemoPlayerSprite>>,
    mut layers: Query<(&Name, &mut ParallaxLayer)>,
    player_state: Query<
        (&PlatformerControllerState, &LinearVelocity, &Transform),
        With<DemoPlayerSprite>,
    >,
) {
    for mut config in &mut controllers {
        config.jump.height = pane.jump_height.max(0.0);
        config.jump.time_to_apex = pane.time_to_apex.max(0.01);
    }

    for mut config in &mut dash_configs {
        config.distance = pane.dash_distance.max(0.0);
        config.duration = pane.dash_duration.max(0.01);
    }

    for mut camera in &mut pixel_camera {
        camera.zoom = pane.pixel_zoom.round().clamp(1.0, 4.0) as u32;
    }

    for mut animator in &mut animators {
        animator.speed_multiplier = pane.spritesheet_speed.max(0.1);
    }

    for mut outline in &mut outlines {
        outline.config.width_pixels = pane.outline_width.max(0.0);
    }

    for mut silhouette in &mut silhouettes {
        silhouette.config.tint_strength = pane.silhouette_tint.clamp(0.0, 1.0);
    }

    for (name, mut layer) in &mut layers {
        if name.as_str().contains("Mountain") {
            layer.camera_factor.x = pane.mountain_factor.max(0.0);
        }
        if name.as_str().contains("Canopy") {
            layer.camera_factor.x = pane.canopy_factor.max(0.0);
        }
    }

    if let Ok((state, velocity, transform)) = player_state.single() {
        pane.phase = format!("{:?}", state.phase);
        pane.player_x = transform.translation.x;
        pane.velocity_x = velocity.x;
    }
}

fn follow_player_camera(
    time: Res<Time>,
    pane: Res<FullDemoPane>,
    player: Query<&Transform, With<DemoPlayerSprite>>,
    mut cameras: Query<&mut PixelCameraTransform, With<CameraRoot>>,
) {
    let Ok(player) = player.single() else {
        return;
    };
    let Ok(mut camera) = cameras.single_mut() else {
        return;
    };

    let target = Vec2::new(
        player.translation.x + 32.0,
        (player.translation.y + 12.0).max(-30.0),
    );
    let blend = 1.0 - (-pane.follow_smoothing.max(0.1) * time.delta_secs()).exp();
    camera.logical_position = camera.logical_position.lerp(target, blend);
}

fn drive_player_animation(
    mut player: Query<
        (
            &PlatformerControllerState,
            &PlatformerDashState,
            &LinearVelocity,
            &mut AnimationController,
            &mut Transform,
        ),
        With<DemoPlayerSprite>,
    >,
) {
    let Ok((state, dash, velocity, mut controller, mut transform)) = player.single_mut() else {
        return;
    };

    let target = if dash.active {
        "dash"
    } else {
        match state.phase {
        PlatformerMotionPhase::Rising | PlatformerMotionPhase::Apex => "rise",
        PlatformerMotionPhase::Falling | PlatformerMotionPhase::Airborne => "fall",
        _ if velocity.x.abs() > 24.0 => "run",
        _ => "idle",
        }
    };
    controller.set_target(AnimationTarget::state(target));

    if velocity.x.abs() > 1.0 {
        transform.scale.x = velocity.x.signum() * PLAYER_SPRITE_SCALE;
    }
}

fn pulse_player_on_dash(
    mut commands: Commands,
    mut dashes: MessageReader<DashStarted>,
    player: Query<Entity, With<DemoPlayerSprite>>,
) {
    let Ok(player) = player.single() else {
        return;
    };

    if dashes.read().next().is_some() {
        commands.entity(player).insert((
            FlashEffect::new(FlashConfig::damage()),
            SquashStretchEffect::new(SquashStretchConfig::landing()),
        ));
    }
}

fn update_overlay(
    player: Query<
        (&PlatformerControllerState, &PlatformerDashState, &SpritesheetAnimator),
        With<DemoPlayerSprite>,
    >,
    mut overlay: Query<&mut Text, With<OverlayText>>,
) {
    let Ok((state, dash, animator)) = player.single() else {
        return;
    };
    let Ok(mut overlay) = overlay.single_mut() else {
        return;
    };

    *overlay = Text::new(format!(
        "A/D move  Space jump  Shift dash\nPixel camera + parallax + spritesheet + sprite effects\n\nphase: {:?}\nclip: {}\nframe: {}\ndash charges: {}\nTry dashing behind the ruin on the right.",
        state.phase,
        animator
            .current_clip
            .as_ref()
            .map_or("none", |clip| clip.as_str()),
        animator.current_frame,
        dash.remaining_charges,
    ));
}

fn maybe_install_auto_exit(app: &mut App) {
    let Some(seconds) = std::env::var("CHARACTER_2D_FULL_DEMO_AUTO_EXIT_SECONDS")
        .ok()
        .and_then(|value| value.parse::<f32>().ok())
    else {
        return;
    };

    app.insert_resource(AutoExitAfter(Timer::from_seconds(
        seconds.max(0.1),
        TimerMode::Once,
    )));
}

fn auto_exit_after(
    time: Res<Time>,
    timer: Option<ResMut<AutoExitAfter>>,
    mut exit: MessageWriter<AppExit>,
) {
    let Some(mut timer) = timer else {
        return;
    };

    if timer.0.tick(time.delta()).just_finished() {
        exit.write(AppExit::Success);
    }
}

fn make_demo_atlas(
    images: &mut Assets<Image>,
    layouts: &mut Assets<TextureAtlasLayout>,
) -> DemoAtlas {
    let frame_colors: [[u8; 4]; 8] = [
        [244, 191, 117, 255],
        [247, 144, 96, 255],
        [123, 201, 176, 255],
        [84, 160, 212, 255],
        [110, 117, 240, 255],
        [252, 236, 97, 255],
        [255, 115, 87, 255],
        [248, 248, 248, 255],
    ];
    let width = FRAME_SIZE.x * frame_colors.len() as u32;
    let height = FRAME_SIZE.y;
    let mut data = vec![0u8; (width * height * 4) as usize];

    for (frame_index, color) in frame_colors.iter().enumerate() {
        let frame_origin = frame_index as u32 * FRAME_SIZE.x;
        for y in 0..FRAME_SIZE.y {
            for x in 0..FRAME_SIZE.x {
                let gx = frame_origin + x;
                let offset = ((y * width + gx) * 4) as usize;
                let border = x < 2 || y < 2 || x >= FRAME_SIZE.x - 2 || y >= FRAME_SIZE.y - 2;
                let checker = ((x / 4) + (y / 4) + frame_index as u32).is_multiple_of(2);
                let eye_band = y > 7 && y < 12 && x > 5 && x < 18;

                let (r, g, b) = if border {
                    (20, 20, 22)
                } else if eye_band && checker {
                    (
                        color[0].saturating_sub(60),
                        color[1].saturating_sub(40),
                        color[2].saturating_sub(20),
                    )
                } else if checker {
                    (
                        color[0].saturating_sub(18),
                        color[1].saturating_sub(18),
                        color[2].saturating_sub(18),
                    )
                } else {
                    (color[0], color[1], color[2])
                };

                data[offset] = r;
                data[offset + 1] = g;
                data[offset + 2] = b;
                data[offset + 3] = 255;
            }
        }
    }

    let image = images.add(Image::new_fill(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    ));
    let layout = layouts.add(TextureAtlasLayout::from_grid(
        FRAME_SIZE,
        frame_colors.len() as u32,
        1,
        None,
        None,
    ));

    DemoAtlas { image, layout }
}

fn demo_textures(images: &mut Assets<Image>) -> DemoTextures {
    DemoTextures {
        sky: images.add(pattern_image(
            UVec2::new(256, 256),
            Color::srgb(0.72, 0.88, 0.98),
            Color::srgb(0.96, 0.98, 1.0),
            6.0,
            false,
        )),
        mountains: images.add(mountain_strip(UVec2::new(320, 96))),
        canopy: images.add(stripe_strip(
            UVec2::new(256, 64),
            Color::srgba(0.10, 0.24, 0.12, 1.0),
            Color::srgba(0.18, 0.34, 0.16, 1.0),
            10,
        )),
    }
}

fn add_forest_stack(commands: &mut Commands, rig: Entity, textures: &DemoTextures) {
    spawn_parallax_layer(
        commands,
        rig,
        "Sky Layer",
        textures.sky.clone(),
        ParallaxLayer::tiled()
            .with_camera_factor(Vec2::ONE)
            .with_repeat(ParallaxAxes::both())
            .with_coverage_margin(Vec2::new(96.0, 48.0))
            .with_tint(Color::srgba(0.95, 0.98, 1.0, 0.92))
            .with_scale(Vec2::splat(2.0))
            .with_origin(Vec2::new(0.0, 24.0)),
    );

    spawn_parallax_layer(
        commands,
        rig,
        "Mountain Layer",
        textures.mountains.clone(),
        ParallaxLayer {
            strategy: ParallaxLayerStrategy::Segmented(ParallaxSegmented::default()),
            camera_factor: Vec2::new(0.84, 1.0),
            repeat: ParallaxAxes::horizontal(),
            origin: Vec2::new(0.0, -96.0),
            depth: 1.0,
            scale: Vec2::splat(1.4),
            tint: Color::srgb(0.34, 0.47, 0.56),
            source_size: Some(Vec2::new(320.0, 96.0)),
            ..default()
        },
    );

    spawn_parallax_layer(
        commands,
        rig,
        "Canopy Layer",
        textures.canopy.clone(),
        ParallaxLayer {
            strategy: ParallaxLayerStrategy::Segmented(ParallaxSegmented {
                extra_rings: UVec2::new(2, 0),
            }),
            camera_factor: Vec2::new(1.08, 1.0),
            repeat: ParallaxAxes::horizontal(),
            origin: Vec2::new(0.0, -200.0),
            depth: 2.0,
            scale: Vec2::new(1.5, 2.0),
            tint: Color::srgb(0.14, 0.28, 0.14),
            source_size: Some(Vec2::new(256.0, 64.0)),
            ..default()
        },
    );
}

fn spawn_parallax_layer(
    commands: &mut Commands,
    rig: Entity,
    name: &str,
    image: Handle<Image>,
    layer: ParallaxLayer,
) {
    commands.spawn((
        Name::new(name.to_string()),
        ChildOf(rig),
        ParallaxLayerBundle {
            layer,
            sprite: Sprite::from_image(image),
            ..default()
        },
    ));
}

fn pattern_image(size: UVec2, a: Color, b: Color, band_height: f32, nearest: bool) -> Image {
    let mut bytes = Vec::with_capacity((size.x * size.y * 4) as usize);
    let a = a.to_srgba();
    let b = b.to_srgba();
    for y in 0..size.y {
        let t = ((y as f32 / band_height).sin() * 0.5 + 0.5).clamp(0.0, 1.0);
        let r = a.red + (b.red - a.red) * t;
        let g = a.green + (b.green - a.green) * t;
        let bl = a.blue + (b.blue - a.blue) * t;
        for _ in 0..size.x {
            bytes.extend_from_slice(&[
                (r * 255.0) as u8,
                (g * 255.0) as u8,
                (bl * 255.0) as u8,
                255,
            ]);
        }
    }
    make_image(size, bytes, nearest)
}

fn mountain_strip(size: UVec2) -> Image {
    let mut bytes = Vec::with_capacity((size.x * size.y * 4) as usize);
    for y in 0..size.y {
        for x in 0..size.x {
            let normalized_x = x as f32 / size.x as f32;
            let ridge = 0.34 + (normalized_x * std::f32::consts::TAU * 2.0).sin() * 0.14;
            let ridge = ridge + (normalized_x * std::f32::consts::TAU * 6.0).sin() * 0.05;
            let y_normalized = y as f32 / size.y as f32;
            let alpha = if y_normalized > ridge { 255 } else { 0 };
            let shade = if y_normalized > ridge + 0.12 { 120 } else { 88 };
            bytes.extend_from_slice(&[shade, shade + 20, shade + 30, alpha]);
        }
    }
    make_image(size, bytes, false)
}

fn stripe_strip(size: UVec2, dark: Color, light: Color, stripe_width: u32) -> Image {
    let dark = dark.to_srgba();
    let light = light.to_srgba();
    let mut bytes = Vec::with_capacity((size.x * size.y * 4) as usize);
    for y in 0..size.y {
        for x in 0..size.x {
            let use_light = ((x / stripe_width) + (y / stripe_width.max(1))).is_multiple_of(2);
            let color = if use_light { light } else { dark };
            bytes.extend_from_slice(&[
                (color.red * 255.0) as u8,
                (color.green * 255.0) as u8,
                (color.blue * 255.0) as u8,
                255,
            ]);
        }
    }
    make_image(size, bytes, false)
}

fn make_image(size: UVec2, bytes: Vec<u8>, nearest: bool) -> Image {
    let mut image = Image::new(
        Extent3d {
            width: size.x,
            height: size.y,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        bytes,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        mag_filter: if nearest {
            ImageFilterMode::Nearest
        } else {
            ImageFilterMode::Linear
        },
        min_filter: if nearest {
            ImageFilterMode::Nearest
        } else {
            ImageFilterMode::Linear
        },
        mipmap_filter: if nearest {
            ImageFilterMode::Nearest
        } else {
            ImageFilterMode::Linear
        },
        ..default()
    });
    image
}
