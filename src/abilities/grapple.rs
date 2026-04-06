use avian2d::prelude::{LinearVelocity, Position};
use bevy::{
    ecs::{intern::Interned, schedule::ScheduleLabel},
    prelude::*,
};

use crate::{
    PlatformerController, PlatformerControllerConfig, PlatformerControllerDirectives,
    PlatformerControllerSystems, PlatformerMovementIntent, systems::activation::runtime_is_active,
};

use super::{
    PlatformerAbilityComposition, PlatformerAbilityConflictAction, PlatformerAbilityKind,
    ability_activity, dash::PlatformerDashRuntimeState,
    ground_pound::PlatformerGroundPoundRuntimeState,
};

pub struct PlatformerGrapplePlugin {
    pub update_schedule: Interned<dyn ScheduleLabel>,
}

impl PlatformerGrapplePlugin {
    pub fn new(update_schedule: impl ScheduleLabel) -> Self {
        Self {
            update_schedule: update_schedule.intern(),
        }
    }

    pub fn always_on(update_schedule: impl ScheduleLabel) -> Self {
        Self::new(update_schedule)
    }
}

impl Default for PlatformerGrapplePlugin {
    fn default() -> Self {
        Self::always_on(Update)
    }
}

#[derive(SystemSet, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum PlatformerGrappleSystems {
    ResolveDirectives,
    ApplyGrapple,
    SyncState,
}

impl Plugin for PlatformerGrapplePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlatformerAbilityComposition>()
            .add_message::<GrappleAttached>()
            .add_message::<GrappleDetached>()
            .register_type::<PlatformerGrappleConfig>()
            .register_type::<PlatformerGrappleIntent>()
            .register_type::<PlatformerGrapplePhase>()
            .register_type::<PlatformerGrapplePoint>()
            .register_type::<PlatformerGrappleState>()
            .configure_sets(
                self.update_schedule,
                (
                    PlatformerGrappleSystems::ResolveDirectives
                        .after(PlatformerControllerSystems::SenseContacts)
                        .before(PlatformerControllerSystems::ApplyMovement),
                    PlatformerGrappleSystems::ApplyGrapple
                        .after(PlatformerControllerSystems::ApplyMovement)
                        .after(super::ground_pound::PlatformerGroundPoundSystems::ApplyGroundPound)
                        .before(PlatformerControllerSystems::ApplyJump),
                    PlatformerGrappleSystems::SyncState
                        .after(PlatformerControllerSystems::SyncState),
                ),
            )
            .add_systems(
                self.update_schedule,
                (
                    resolve_grapple_directives.in_set(PlatformerGrappleSystems::ResolveDirectives),
                    apply_grapple
                        .in_set(PlatformerControllerSystems::ApplyAbilityMotion)
                        .in_set(PlatformerGrappleSystems::ApplyGrapple),
                    sync_grapple_state.in_set(PlatformerGrappleSystems::SyncState),
                    emit_grapple_messages.in_set(PlatformerGrappleSystems::SyncState),
                    clear_grapple_intents.in_set(PlatformerGrappleSystems::SyncState),
                )
                    .run_if(runtime_is_active),
            );
    }
}

#[derive(Clone, Debug, Message)]
pub struct GrappleAttached {
    pub entity: Entity,
    pub target: Vec2,
}

#[derive(Clone, Debug, Message)]
pub struct GrappleDetached {
    pub entity: Entity,
    pub velocity: Vec2,
}

#[derive(Component, Clone, Debug, Reflect, Default)]
#[reflect(Component, Default, Debug)]
pub struct PlatformerGrapplePoint;

#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component, Debug, Default)]
pub struct PlatformerGrappleConfig {
    pub max_range: f32,
    pub pull_speed: f32,
    pub detach_speed_boost: f32,
    pub aim_assist_angle: f32,
    pub min_rope_length: f32,
    pub retract_speed: f32,
    pub extend_speed: f32,
    pub swing_gravity_multiplier: f32,
    pub swing_input_force: f32,
}

impl Default for PlatformerGrappleConfig {
    fn default() -> Self {
        Self {
            max_range: 300.0,
            pull_speed: 400.0,
            detach_speed_boost: 1.3,
            aim_assist_angle: 0.35,
            min_rope_length: 20.0,
            retract_speed: 200.0,
            extend_speed: 100.0,
            swing_gravity_multiplier: 1.0,
            swing_input_force: 300.0,
        }
    }
}

#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component, Debug, Default)]
pub struct PlatformerGrappleIntent {
    pub pressed: bool,
    pub released: bool,
    pub direction: Vec2,
    pub retract: bool,
    pub extend: bool,
}

impl Default for PlatformerGrappleIntent {
    fn default() -> Self {
        Self {
            pressed: false,
            released: false,
            direction: Vec2::ZERO,
            retract: false,
            extend: false,
        }
    }
}

#[derive(Clone, Copy, Debug, Reflect, PartialEq, Default)]
#[reflect(Debug, PartialEq, Default)]
pub enum PlatformerGrapplePhase {
    #[default]
    Idle,
    Pulling {
        target: Vec2,
        rope_length: f32,
    },
}

#[derive(Component, Clone, Debug, Reflect, PartialEq)]
#[reflect(Component, Debug, Default, PartialEq)]
pub struct PlatformerGrappleState {
    pub phase: PlatformerGrapplePhase,
    pub target_entity: Option<Entity>,
}

impl Default for PlatformerGrappleState {
    fn default() -> Self {
        Self {
            phase: PlatformerGrapplePhase::Idle,
            target_entity: None,
        }
    }
}

#[derive(Bundle)]
pub struct PlatformerGrappleBundle {
    config: PlatformerGrappleConfig,
    intent: PlatformerGrappleIntent,
    state: PlatformerGrappleState,
    runtime: PlatformerGrappleRuntimeState,
}

impl Default for PlatformerGrappleBundle {
    fn default() -> Self {
        Self::with_config(PlatformerGrappleConfig::default())
    }
}

impl PlatformerGrappleBundle {
    pub fn with_config(config: PlatformerGrappleConfig) -> Self {
        Self {
            config,
            intent: PlatformerGrappleIntent::default(),
            state: PlatformerGrappleState::default(),
            runtime: PlatformerGrappleRuntimeState::default(),
        }
    }
}

#[derive(Component, Clone, Debug, Default)]
pub(crate) struct PlatformerGrappleRuntimeState {
    phase: PlatformerGrapplePhase,
    target_entity: Option<Entity>,
    pending_started: Option<Vec2>,
    pending_detached_velocity: Option<Vec2>,
}

impl PlatformerGrappleRuntimeState {
    pub(crate) fn active(&self) -> bool {
        !matches!(self.phase, PlatformerGrapplePhase::Idle)
    }

    pub(crate) fn cancel(&mut self) {
        self.phase = PlatformerGrapplePhase::Idle;
        self.target_entity = None;
    }
}

fn resolve_grapple_directives(
    mut controllers: Query<
        (
            &mut PlatformerControllerDirectives,
            &PlatformerGrappleRuntimeState,
        ),
        With<PlatformerController>,
    >,
) {
    for (mut directives, runtime) in &mut controllers {
        if runtime.active() {
            directives.suppress_horizontal_movement = true;
            directives.suppress_jump_logic = true;
            directives.suppress_wall_interactions = true;
        }
    }
}

fn apply_grapple(
    time: Res<Time>,
    composition: Res<PlatformerAbilityComposition>,
    grapple_points: Query<(Entity, &GlobalTransform), With<PlatformerGrapplePoint>>,
    mut controllers: Query<
        (
            &PlatformerControllerConfig,
            &PlatformerMovementIntent,
            &mut PlatformerControllerDirectives,
            &PlatformerGrappleConfig,
            &PlatformerGrappleIntent,
            &Position,
            &mut LinearVelocity,
            &crate::components::PlatformerControllerRuntimeState,
            &mut PlatformerGrappleRuntimeState,
            Option<&mut PlatformerDashRuntimeState>,
            Option<&mut PlatformerGroundPoundRuntimeState>,
        ),
        With<PlatformerController>,
    >,
) {
    let delta_secs = time.delta_secs();

    for (
        controller_config,
        controller_intent,
        mut directives,
        config,
        intent,
        position,
        mut velocity,
        controller_runtime,
        mut runtime,
        dash_runtime,
        ground_pound_runtime,
    ) in &mut controllers
    {
        runtime.pending_started = None;
        runtime.pending_detached_velocity = None;

        match runtime.phase {
            PlatformerGrapplePhase::Idle => {
                if !intent.pressed || config.max_range <= 0.0 {
                    continue;
                }

                let dash_active = dash_runtime
                    .as_ref()
                    .is_some_and(|runtime| runtime.active());
                let ground_pound_active = ground_pound_runtime
                    .as_ref()
                    .is_some_and(|runtime| runtime.active());
                let resolution = composition.0.resolve_activation(
                    PlatformerAbilityKind::Grapple,
                    ability_activity(dash_active, ground_pound_active, runtime.active()),
                );
                if !resolution.allow_requested {
                    continue;
                }

                if matches!(resolution.dash, PlatformerAbilityConflictAction::Cancel)
                    && let Some(mut runtime) = dash_runtime
                {
                    runtime.cancel();
                }
                if matches!(
                    resolution.ground_pound,
                    PlatformerAbilityConflictAction::Cancel
                ) && let Some(mut runtime) = ground_pound_runtime
                {
                    runtime.cancel();
                }

                let aim = if intent.direction.length_squared() > 0.01 {
                    intent.direction.normalize()
                } else {
                    Vec2::new(controller_runtime.facing_sign, 0.5).normalize()
                };

                let player_pos = position.0;
                let max_range_sq = config.max_range * config.max_range;
                let cos_threshold = config.aim_assist_angle.cos();

                let mut best_target = None;
                let mut best_dist_sq = f32::MAX;

                for (target_entity, global_transform) in &grapple_points {
                    let point = global_transform.translation().xy();
                    let offset = point - player_pos;
                    let dist_sq = offset.length_squared();

                    if dist_sq > max_range_sq || dist_sq < 1.0 {
                        continue;
                    }

                    let dir = offset.normalize();
                    if dir.dot(aim) < cos_threshold {
                        continue;
                    }

                    if dist_sq < best_dist_sq {
                        best_dist_sq = dist_sq;
                        best_target = Some((target_entity, point));
                    }
                }

                if let Some((target_entity, target)) = best_target {
                    runtime.phase = PlatformerGrapplePhase::Pulling {
                        target,
                        rope_length: (target - player_pos).length(),
                    };
                    runtime.target_entity = Some(target_entity);
                    runtime.pending_started = Some(target);
                    directives.suppress_jump_logic = true;
                    directives.suppress_wall_interactions = true;
                }
            }
            PlatformerGrapplePhase::Pulling {
                target,
                mut rope_length,
            } => {
                let should_detach_on_jump = controller_intent.jump_pressed
                    && composition.0.detach_grapple_on_jump(ability_activity(
                        dash_runtime
                            .as_ref()
                            .is_some_and(|runtime| runtime.active()),
                        ground_pound_runtime
                            .as_ref()
                            .is_some_and(|runtime| runtime.active()),
                        true,
                    ));

                if intent.released || should_detach_on_jump {
                    velocity.0 *= config.detach_speed_boost;
                    runtime.phase = PlatformerGrapplePhase::Idle;
                    runtime.target_entity = None;
                    runtime.pending_detached_velocity = Some(velocity.0);
                    if should_detach_on_jump {
                        directives.suppress_jump_logic = false;
                    }
                    continue;
                }

                directives.suppress_jump_logic = true;
                directives.suppress_wall_interactions = true;

                if intent.retract {
                    rope_length = (rope_length - config.retract_speed * delta_secs)
                        .max(config.min_rope_length);
                }
                if intent.extend {
                    rope_length += config.extend_speed * delta_secs;
                }

                let player_pos = position.0;
                let to_target = target - player_pos;
                let distance = to_target.length();

                if distance < config.min_rope_length {
                    runtime.phase = PlatformerGrapplePhase::Idle;
                    runtime.target_entity = None;
                    runtime.pending_detached_velocity = Some(velocity.0);
                    continue;
                }

                let radial_dir = to_target / distance;
                let gravity =
                    controller_config.jump.base_gravity() * config.swing_gravity_multiplier;
                velocity.y -= gravity * delta_secs;

                if controller_intent.move_axis.abs() > 0.01 {
                    let tangent = Vec2::new(-radial_dir.y, radial_dir.x);
                    velocity.0 += tangent
                        * controller_intent.move_axis
                        * config.swing_input_force
                        * delta_secs;
                }

                if config.pull_speed > 0.0 {
                    velocity.0 += radial_dir * config.pull_speed * delta_secs;
                }

                if distance >= rope_length {
                    let radial_speed = velocity.0.dot(radial_dir);
                    if radial_speed < 0.0 {
                        velocity.0 -= radial_dir * radial_speed;
                    }

                    let overshoot = distance - rope_length;
                    if overshoot > 0.1 {
                        velocity.0 += radial_dir * (overshoot / delta_secs.max(f32::EPSILON)) * 0.3;
                    }
                }

                runtime.phase = PlatformerGrapplePhase::Pulling {
                    target,
                    rope_length,
                };
            }
        }
    }
}

fn sync_grapple_state(
    mut controllers: Query<
        (&PlatformerGrappleRuntimeState, &mut PlatformerGrappleState),
        With<PlatformerController>,
    >,
) {
    for (runtime, mut state) in &mut controllers {
        state.phase = runtime.phase;
        state.target_entity = runtime.target_entity;
    }
}

fn emit_grapple_messages(
    mut query: Query<(Entity, &mut PlatformerGrappleRuntimeState), With<PlatformerController>>,
    mut attached: MessageWriter<GrappleAttached>,
    mut detached: MessageWriter<GrappleDetached>,
) {
    for (entity, mut runtime) in &mut query {
        if let Some(target) = runtime.pending_started.take() {
            attached.write(GrappleAttached { entity, target });
        }

        if let Some(velocity) = runtime.pending_detached_velocity.take() {
            detached.write(GrappleDetached { entity, velocity });
        }
    }
}

fn clear_grapple_intents(
    mut intents: Query<&mut PlatformerGrappleIntent, With<PlatformerController>>,
) {
    for mut intent in &mut intents {
        intent.pressed = false;
        intent.released = false;
        intent.retract = false;
        intent.extend = false;
    }
}
