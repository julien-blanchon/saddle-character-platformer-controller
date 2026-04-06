use std::{sync::Arc, time::Duration};

use avian2d::prelude::{Collider, PhysicsPlugins, Position, RigidBody, Rotation};
use bevy::{
    app::PostStartup,
    ecs::{message::MessageCursor, schedule::ScheduleLabel},
    prelude::*,
    time::TimeUpdateStrategy,
};

use crate::{
    AirJumpConsumed, JumpStarted, Landed, PlatformerAbilityActivationResolution,
    PlatformerAbilityActivity, PlatformerAbilityComposition, PlatformerAbilityCompositionPolicy,
    PlatformerAbilityConflictAction, PlatformerAbilityKind, PlatformerControllerBundle,
    PlatformerControllerPlugin, PlatformerControllerState, PlatformerControllerSystems,
    PlatformerDashPlugin, PlatformerGrapplePlugin, PlatformerGroundPoundPlugin,
    PlatformerJumpConfig, PlatformerJumpKind, PlatformerMovementIntent,
    components::{PendingJumpMessage, PlatformerControllerRuntimeState},
    systems::{activation::PlatformerControllerRuntime, state_sync},
};

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
struct ActivateSchedule;

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
struct DeactivateSchedule;

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
struct SimulationSchedule;

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct AfterControllerState;

#[derive(Resource, Default, Debug, PartialEq, Eq)]
struct OrderLog(Vec<&'static str>);

#[derive(Debug)]
struct DashCancelsGroundPoundPolicy;

impl PlatformerAbilityCompositionPolicy for DashCancelsGroundPoundPolicy {
    fn resolve_activation(
        &self,
        requested: PlatformerAbilityKind,
        _active: PlatformerAbilityActivity,
    ) -> PlatformerAbilityActivationResolution {
        match requested {
            PlatformerAbilityKind::Dash => PlatformerAbilityActivationResolution {
                allow_requested: true,
                dash: PlatformerAbilityConflictAction::Keep,
                ground_pound: PlatformerAbilityConflictAction::Cancel,
                grapple: PlatformerAbilityConflictAction::Keep,
            },
            PlatformerAbilityKind::GroundPound | PlatformerAbilityKind::Grapple => {
                PlatformerAbilityActivationResolution::allow()
            }
        }
    }

    fn detach_grapple_on_jump(&self, _active: PlatformerAbilityActivity) -> bool {
        false
    }
}

fn push_controller_marker(mut log: ResMut<OrderLog>) {
    log.0.push("controller");
}

fn push_after_marker(mut log: ResMut<OrderLog>) {
    log.0.push("after");
}

#[test]
fn plugin_builds_with_custom_schedule_labels_and_ordering_points() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, PhysicsPlugins::default()))
        .init_schedule(ActivateSchedule)
        .init_schedule(DeactivateSchedule)
        .init_schedule(SimulationSchedule)
        .init_resource::<OrderLog>()
        .add_plugins(PlatformerControllerPlugin::new(
            ActivateSchedule,
            DeactivateSchedule,
            SimulationSchedule,
        ))
        .configure_sets(
            SimulationSchedule,
            PlatformerControllerSystems::SyncState.before(AfterControllerState),
        )
        .add_systems(
            SimulationSchedule,
            (
                push_controller_marker.in_set(PlatformerControllerSystems::SyncState),
                push_after_marker.in_set(AfterControllerState),
            ),
        );

    app.finish();
    app.world_mut().run_schedule(ActivateSchedule);
    app.world_mut().run_schedule(SimulationSchedule);

    assert_eq!(
        app.world().resource::<OrderLog>().0,
        vec!["controller", "after"]
    );
}

#[test]
fn always_on_constructor_activates_runtime_after_startup() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, PhysicsPlugins::default()))
        .add_plugins(PlatformerControllerPlugin::always_on(Update));

    app.finish();
    assert!(!app.world().resource::<PlatformerControllerRuntime>().0);

    app.world_mut().run_schedule(PostStartup);

    assert!(app.world().resource::<PlatformerControllerRuntime>().0);
}

#[test]
fn bundle_contains_runtime_components() {
    let mut world = World::new();
    let entity = world
        .spawn(PlatformerControllerBundle::new(Collider::rectangle(
            14.0, 22.0,
        )))
        .id();
    let entity_ref = world.entity(entity);

    assert!(entity_ref.contains::<PlatformerControllerState>());
    assert!(entity_ref.contains::<PlatformerMovementIntent>());
    assert!(entity_ref.contains::<PlatformerControllerRuntimeState>());
    assert!(entity_ref.contains::<Position>());
    assert!(entity_ref.contains::<Rotation>());
}

#[test]
fn jump_config_derives_gravity_and_jump_speed() {
    let config = PlatformerJumpConfig {
        height: 64.0,
        time_to_apex: 0.4,
        ..default()
    };

    assert!((config.base_gravity() - 800.0).abs() < 0.001);
    assert!((config.jump_speed() - 320.0).abs() < 0.001);
}

#[test]
fn emit_messages_flushes_pending_runtime_events() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_message::<JumpStarted>()
        .add_message::<Landed>()
        .add_message::<crate::WallJumpStarted>()
        .add_message::<AirJumpConsumed>()
        .add_message::<crate::WallClingStarted>()
        .add_systems(Update, state_sync::emit_messages);

    let entity = app
        .world_mut()
        .spawn(PlatformerControllerBundle::new(Collider::rectangle(
            14.0, 22.0,
        )))
        .id();
    {
        let mut runtime = app
            .world_mut()
            .get_mut::<PlatformerControllerRuntimeState>(entity)
            .expect("bundle should include runtime state");
        runtime.pending_jump = Some(PendingJumpMessage {
            kind: PlatformerJumpKind::Air,
            velocity: Vec2::new(12.0, 34.0),
            used_buffer: true,
        });
        runtime.pending_landed_impact_speed = Some(18.0);
        runtime.pending_landed_support = Some(Entity::from_bits(9));
        runtime.pending_air_jump_consumed = Some(0);
    }

    app.update();

    let mut jump_cursor = MessageCursor::<JumpStarted>::default();
    let jumps: Vec<_> = jump_cursor
        .read(app.world().resource::<Messages<JumpStarted>>())
        .cloned()
        .collect();
    assert_eq!(jumps.len(), 1);
    assert_eq!(jumps[0].entity, entity);
    assert_eq!(jumps[0].kind, PlatformerJumpKind::Air);
    assert!(jumps[0].used_buffer);

    let mut landed_cursor = MessageCursor::<Landed>::default();
    let landings: Vec<_> = landed_cursor
        .read(app.world().resource::<Messages<Landed>>())
        .cloned()
        .collect();
    assert_eq!(landings.len(), 1);
    assert_eq!(landings[0].impact_speed, 18.0);
    assert_eq!(landings[0].support_entity, Some(Entity::from_bits(9)));

    let mut air_cursor = MessageCursor::<AirJumpConsumed>::default();
    let air_jumps: Vec<_> = air_cursor
        .read(app.world().resource::<Messages<AirJumpConsumed>>())
        .cloned()
        .collect();
    assert_eq!(air_jumps.len(), 1);
    assert_eq!(air_jumps[0].remaining_air_jumps, 0);
}

#[test]
fn plugin_initializes_in_a_simple_physics_scene() {
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
    app.world_mut().spawn((
        Name::new("Ground"),
        RigidBody::Static,
        Collider::rectangle(300.0, 20.0),
        Transform::from_xyz(0.0, -16.0, 0.0),
        GlobalTransform::default(),
    ));
    let entity = app
        .world_mut()
        .spawn((
            PlatformerControllerBundle::new(Collider::rectangle(14.0, 22.0))
                .with_transform(Transform::from_xyz(0.0, 40.0, 0.0)),
        ))
        .id();

    for _ in 0..10 {
        app.update();
    }

    assert!(
        app.world()
            .entity(entity)
            .contains::<PlatformerControllerState>()
    );
}

#[test]
fn default_ability_policy_blocks_conflicting_activations() {
    let composition = PlatformerAbilityComposition::default();

    let dash_during_ground_pound = composition.0.resolve_activation(
        PlatformerAbilityKind::Dash,
        PlatformerAbilityActivity {
            ground_pound: true,
            ..default()
        },
    );
    assert!(!dash_during_ground_pound.allow_requested);

    let grapple_during_ground_pound = composition.0.resolve_activation(
        PlatformerAbilityKind::Grapple,
        PlatformerAbilityActivity {
            ground_pound: true,
            ..default()
        },
    );
    assert!(grapple_during_ground_pound.allow_requested);
    assert_eq!(
        grapple_during_ground_pound.ground_pound,
        PlatformerAbilityConflictAction::Cancel
    );
    assert!(
        composition
            .0
            .detach_grapple_on_jump(PlatformerAbilityActivity {
                grapple: true,
                ..default()
            })
    );
}

#[test]
fn injected_ability_policy_is_preserved_when_ability_plugins_build() {
    let mut app = App::new();
    app.insert_resource(PlatformerAbilityComposition(Arc::new(
        DashCancelsGroundPoundPolicy,
    )));
    app.add_plugins(MinimalPlugins)
        .add_plugins(PlatformerControllerPlugin::default())
        .add_plugins(PlatformerDashPlugin::default())
        .add_plugins(PlatformerGroundPoundPlugin::default())
        .add_plugins(PlatformerGrapplePlugin::default());

    let composition = app
        .world()
        .resource::<PlatformerAbilityComposition>()
        .clone();
    let resolution = composition.0.resolve_activation(
        PlatformerAbilityKind::Dash,
        PlatformerAbilityActivity {
            ground_pound: true,
            ..default()
        },
    );

    assert!(resolution.allow_requested);
    assert_eq!(
        resolution.ground_pound,
        PlatformerAbilityConflictAction::Cancel
    );
    assert!(
        !composition
            .0
            .detach_grapple_on_jump(PlatformerAbilityActivity {
                grapple: true,
                ..default()
            })
    );
}
