use std::time::Duration;

use avian2d::prelude::{
    Collider, CustomPositionIntegration, LinearVelocity, PhysicsPlugins, Position, RigidBody,
    Rotation,
};
use bevy::{app::PostStartup, ecs::message::MessageCursor, prelude::*, time::TimeUpdateStrategy};

use crate::{
    AirJumpConsumed, JumpStarted, PlatformerControllerBundle, PlatformerControllerConfig,
    PlatformerControllerPlugin, PlatformerControllerState, PlatformerJumpKind,
    PlatformerMovementIntent, PlatformerOneWayPlatform, WallJumpStarted,
};

fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(AssetPlugin::default())
        .add_plugins(TransformPlugin)
        .add_plugins(PhysicsPlugins::default())
        .insert_resource(bevy::ecs::error::DefaultErrorHandler(
            bevy::ecs::error::ignore,
        ))
        .insert_resource(Time::<Fixed>::from_hz(60.0))
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
            1.0 / 60.0,
        )))
        .add_plugins(PlatformerControllerPlugin::default());

    app.finish();
    app.world_mut().run_schedule(PostStartup);
    app
}

fn spawn_ground(app: &mut App, center: Vec2, size: Vec2) {
    app.world_mut().spawn((
        Name::new("Ground"),
        RigidBody::Static,
        Collider::rectangle(size.x, size.y),
        Transform::from_xyz(center.x, center.y, 0.0),
        GlobalTransform::default(),
    ));
}

fn spawn_wall(app: &mut App, name: &str, center: Vec2, size: Vec2) {
    app.world_mut().spawn((
        Name::new(name.to_string()),
        RigidBody::Static,
        Collider::rectangle(size.x, size.y),
        Transform::from_xyz(center.x, center.y, 0.0),
        GlobalTransform::default(),
    ));
}

fn spawn_one_way_platform(app: &mut App, center: Vec2, size: Vec2) {
    app.world_mut().spawn((
        Name::new("One Way Platform"),
        PlatformerOneWayPlatform,
        RigidBody::Static,
        Collider::rectangle(size.x, size.y),
        Transform::from_xyz(center.x, center.y, 0.0),
        GlobalTransform::default(),
    ));
}

fn spawn_moving_platform(app: &mut App, center: Vec2, size: Vec2) -> Entity {
    app.world_mut()
        .spawn((
            Name::new("Moving Platform"),
            RigidBody::Kinematic,
            CustomPositionIntegration,
            Collider::rectangle(size.x, size.y),
            Position(center),
            Rotation::default(),
            Transform::from_xyz(center.x, center.y, 0.0),
            LinearVelocity::ZERO,
        ))
        .id()
}

fn spawn_player(app: &mut App, center: Vec2, config: PlatformerControllerConfig) -> Entity {
    app.world_mut()
        .spawn((
            Name::new("Player"),
            PlatformerControllerBundle::with_config(Collider::rectangle(18.0, 30.0), config)
                .with_transform(Transform::from_xyz(center.x, center.y, 0.0)),
        ))
        .id()
}

fn set_intent(
    app: &mut App,
    entity: Entity,
    move_axis: f32,
    jump_pressed: bool,
    jump_held: bool,
    drop_pressed: bool,
) {
    let mut intent = app
        .world_mut()
        .get_mut::<PlatformerMovementIntent>(entity)
        .expect("player should have PlatformerMovementIntent");
    intent.move_axis = move_axis;
    intent.jump_pressed = jump_pressed;
    intent.jump_held = jump_held;
    intent.drop_pressed = drop_pressed;
}

fn set_velocity(app: &mut App, entity: Entity, velocity: Vec2) {
    app.world_mut()
        .get_mut::<LinearVelocity>(entity)
        .expect("player should have LinearVelocity")
        .0 = velocity;
}

fn state(app: &App, entity: Entity) -> PlatformerControllerState {
    app.world()
        .get::<PlatformerControllerState>(entity)
        .expect("player should have PlatformerControllerState")
        .clone()
}

fn step(app: &mut App, frames: usize) {
    for _ in 0..frames {
        app.update();
    }
}

#[test]
fn walking_off_ledge_then_jumping_within_coyote_window_succeeds() {
    let mut app = test_app();
    let mut config = PlatformerControllerConfig::default();
    config.jump.max_air_jumps = 0;

    spawn_ground(&mut app, Vec2::new(-85.0, -86.0), Vec2::new(110.0, 28.0));
    let player = spawn_player(&mut app, Vec2::new(-36.0, -57.0), config);

    step(&mut app, 10);
    assert!(state(&app, player).is_grounded);

    let mut walked_off = false;
    for _ in 0..90 {
        set_intent(&mut app, player, 1.0, false, false, false);
        app.update();
        let controller = state(&app, player);
        if !controller.is_grounded && controller.can_use_coyote_jump {
            walked_off = true;
            break;
        }
    }

    assert!(
        walked_off,
        "player should leave the ledge during coyote time"
    );

    let mut jump_cursor = MessageCursor::<JumpStarted>::default();
    set_intent(&mut app, player, 1.0, true, true, false);
    app.update();
    let jumps: Vec<_> = jump_cursor
        .read(app.world().resource::<Messages<JumpStarted>>())
        .cloned()
        .collect();
    step(&mut app, 3);

    let controller = state(&app, player);
    assert!(
        controller.velocity.y > 100.0,
        "expected upward launch from coyote jump"
    );
    assert_eq!(
        jumps.last().map(|message| message.kind),
        Some(PlatformerJumpKind::Coyote)
    );
}

#[test]
fn jump_buffer_just_before_landing_fires_on_touchdown() {
    let mut app = test_app();
    let mut config = PlatformerControllerConfig::default();
    config.jump.max_air_jumps = 0;

    spawn_ground(&mut app, Vec2::new(0.0, -150.0), Vec2::new(420.0, 24.0));
    let player = spawn_player(&mut app, Vec2::new(0.0, -108.0), config);
    set_velocity(&mut app, player, Vec2::new(0.0, -320.0));

    let mut jump_cursor = MessageCursor::<JumpStarted>::default();
    set_intent(&mut app, player, 0.0, true, true, false);

    let mut buffered_jump = None;
    let mut relaunched = false;
    for _ in 0..6 {
        app.update();
        if buffered_jump.is_none() {
            buffered_jump = jump_cursor
                .read(app.world().resource::<Messages<JumpStarted>>())
                .last()
                .cloned();
        }
        let controller = state(&app, player);
        if controller.velocity.y > 100.0 {
            relaunched = true;
            break;
        }
    }

    assert!(
        relaunched,
        "buffered jump should relaunch the player on landing"
    );
    assert_eq!(
        buffered_jump.as_ref().map(|message| message.kind),
        Some(PlatformerJumpKind::Ground)
    );
    assert_eq!(
        buffered_jump.as_ref().map(|message| message.used_buffer),
        Some(true)
    );
}

#[test]
fn wall_slide_requires_valid_wall_contact_and_wall_jump_launches_away() {
    let mut app = test_app();
    let mut config = PlatformerControllerConfig::default();
    config.jump.max_air_jumps = 0;

    spawn_ground(&mut app, Vec2::new(0.0, -150.0), Vec2::new(520.0, 38.0));
    spawn_wall(
        &mut app,
        "Left Wall",
        Vec2::new(-120.0, 30.0),
        Vec2::new(40.0, 360.0),
    );
    spawn_wall(
        &mut app,
        "Right Wall",
        Vec2::new(120.0, 30.0),
        Vec2::new(40.0, 360.0),
    );
    let player = spawn_player(&mut app, Vec2::new(-92.0, 36.0), config);
    set_velocity(&mut app, player, Vec2::new(0.0, -20.0));

    let mut wall_sliding = false;
    for _ in 0..90 {
        set_intent(&mut app, player, -1.0, false, false, false);
        app.update();
        if state(&app, player).phase == crate::PlatformerMotionPhase::WallSliding {
            wall_sliding = true;
            break;
        }
    }

    assert!(wall_sliding, "expected a wall slide before jumping away");

    let mut wall_jump_cursor = MessageCursor::<WallJumpStarted>::default();
    set_intent(&mut app, player, -1.0, true, true, false);
    app.update();
    let wall_jumps: Vec<_> = wall_jump_cursor
        .read(app.world().resource::<Messages<WallJumpStarted>>())
        .cloned()
        .collect();
    step(&mut app, 3);

    let controller = state(&app, player);
    assert!(
        controller.velocity.x > 100.0,
        "wall jump should push away from the left wall"
    );
    assert!(
        controller.velocity.y > 100.0,
        "wall jump should add vertical lift"
    );
    assert_eq!(
        wall_jumps.last().map(|message| message.side),
        Some(crate::PlatformerWallSide::Left)
    );
}

#[test]
fn landing_resets_air_jumps_after_consumption() {
    let mut app = test_app();
    let mut config = PlatformerControllerConfig::default();
    config.jump.max_air_jumps = 1;

    spawn_ground(&mut app, Vec2::new(0.0, -150.0), Vec2::new(420.0, 24.0));
    let player = spawn_player(&mut app, Vec2::new(0.0, -120.0), config);

    let mut grounded = false;
    for _ in 0..20 {
        app.update();
        if state(&app, player).is_grounded {
            grounded = true;
            break;
        }
    }
    assert!(
        grounded,
        "player should settle onto the ground before testing jumps"
    );

    set_intent(&mut app, player, 0.0, true, true, false);
    let mut airborne = false;
    for _ in 0..40 {
        app.update();
        let controller = state(&app, player);
        if !controller.is_grounded
            && !controller.can_use_coyote_jump
            && controller.velocity.y > 60.0
        {
            airborne = true;
            break;
        }
    }
    assert!(
        airborne,
        "ground jump should launch the player into the air"
    );
    assert_eq!(state(&app, player).remaining_air_jumps, 1);

    let mut air_jump_cursor = MessageCursor::<AirJumpConsumed>::default();
    set_intent(&mut app, player, 0.0, true, true, false);
    app.update();
    let air_jumps: Vec<_> = air_jump_cursor
        .read(app.world().resource::<Messages<AirJumpConsumed>>())
        .cloned()
        .collect();
    step(&mut app, 3);
    assert_eq!(
        air_jumps.last().map(|message| message.remaining_air_jumps),
        Some(0)
    );
    assert_eq!(state(&app, player).remaining_air_jumps, 0);

    let mut landed = false;
    for _ in 0..180 {
        app.update();
        if state(&app, player).is_grounded {
            landed = true;
            break;
        }
    }

    assert!(landed, "player should land back on the ground");
    assert_eq!(state(&app, player).remaining_air_jumps, 1);
}

#[test]
fn moving_platform_velocity_is_inherited_on_jump() {
    let mut app = test_app();
    let mut config = PlatformerControllerConfig::default();
    config.platforms.velocity_inheritance = crate::PlatformVelocityInheritance::Full;

    spawn_ground(&mut app, Vec2::new(0.0, -150.0), Vec2::new(420.0, 24.0));
    let platform = spawn_moving_platform(&mut app, Vec2::new(-40.0, -20.0), Vec2::new(120.0, 18.0));
    let player = spawn_player(&mut app, Vec2::new(-40.0, 4.0), config);

    let mut platform_x = -40.0;
    let mut inherited = false;
    for _ in 0..30 {
        platform_x += 1.5;
        {
            let mut position = app.world_mut().get_mut::<Position>(platform).unwrap();
            position.0.x = platform_x;
        }
        {
            let mut transform = app.world_mut().get_mut::<Transform>(platform).unwrap();
            transform.translation.x = platform_x;
        }
        app.world_mut()
            .get_mut::<LinearVelocity>(platform)
            .unwrap()
            .0 = Vec2::new(90.0, 0.0);

        app.update();
        if state(&app, player).support_velocity.x > 5.0 {
            inherited = true;
            break;
        }
    }

    assert!(
        inherited,
        "expected support motion to be detected from moving platform"
    );

    set_intent(&mut app, player, 0.0, true, true, false);
    step(&mut app, 4);

    let controller = state(&app, player);
    assert!(
        controller.velocity.x.abs() > 10.0,
        "jumping from a moving platform should keep horizontal motion"
    );
}

#[test]
fn drop_through_one_way_platform_reaches_the_ground_below() {
    let mut app = test_app();

    spawn_ground(&mut app, Vec2::new(0.0, -150.0), Vec2::new(420.0, 24.0));
    spawn_one_way_platform(&mut app, Vec2::new(0.0, -60.0), Vec2::new(170.0, 12.0));
    let player = spawn_player(
        &mut app,
        Vec2::new(0.0, -116.0),
        PlatformerControllerConfig::default(),
    );

    set_intent(&mut app, player, 0.0, true, true, false);

    let mut settled_on_platform = false;
    for _ in 0..180 {
        app.update();
        let controller = state(&app, player);
        if controller.is_grounded
            && app
                .world()
                .get::<Position>(player)
                .is_some_and(|position| position.0.y > -70.0)
            && controller.velocity.y.abs() < 10.0
        {
            settled_on_platform = true;
            break;
        }
    }
    assert!(
        settled_on_platform,
        "player should settle on the one-way platform before dropping through"
    );

    set_intent(&mut app, player, 0.0, false, false, true);

    let mut reached_ground = false;
    for _ in 0..120 {
        app.update();
        let position = app
            .world()
            .get::<Position>(player)
            .expect("player should have Position")
            .0;
        let controller = state(&app, player);
        if controller.is_grounded && position.y < -85.0 {
            reached_ground = true;
            break;
        }
    }

    assert!(
        reached_ground,
        "drop-through should move the player to the ground below the one-way platform"
    );
}
