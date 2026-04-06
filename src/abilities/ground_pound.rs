use avian2d::prelude::LinearVelocity;
use bevy::{
    ecs::{intern::Interned, schedule::ScheduleLabel},
    prelude::*,
};

use crate::{
    PlatformerController, PlatformerControllerDirectives, PlatformerControllerSystems,
    systems::activation::runtime_is_active,
};

use super::{
    PlatformerAbilityComposition, PlatformerAbilityConflictAction, PlatformerAbilityKind,
    ability_activity, dash::PlatformerDashRuntimeState, grapple::PlatformerGrappleRuntimeState,
};

pub struct PlatformerGroundPoundPlugin {
    pub update_schedule: Interned<dyn ScheduleLabel>,
}

impl PlatformerGroundPoundPlugin {
    pub fn new(update_schedule: impl ScheduleLabel) -> Self {
        Self {
            update_schedule: update_schedule.intern(),
        }
    }

    pub fn always_on(update_schedule: impl ScheduleLabel) -> Self {
        Self::new(update_schedule)
    }
}

impl Default for PlatformerGroundPoundPlugin {
    fn default() -> Self {
        Self::always_on(Update)
    }
}

#[derive(SystemSet, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum PlatformerGroundPoundSystems {
    ResolveDirectives,
    ApplyGroundPound,
    SyncState,
}

impl Plugin for PlatformerGroundPoundPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlatformerAbilityComposition>()
            .add_message::<GroundPoundStarted>()
            .add_message::<GroundPoundImpact>()
            .register_type::<PlatformerGroundPoundConfig>()
            .register_type::<PlatformerGroundPoundIntent>()
            .register_type::<PlatformerGroundPoundPhase>()
            .register_type::<PlatformerGroundPoundState>()
            .configure_sets(
                self.update_schedule,
                (
                    PlatformerGroundPoundSystems::ResolveDirectives
                        .after(PlatformerControllerSystems::SenseContacts)
                        .before(PlatformerControllerSystems::ApplyMovement),
                    PlatformerGroundPoundSystems::ApplyGroundPound
                        .after(PlatformerGroundPoundSystems::ResolveDirectives)
                        .after(PlatformerControllerSystems::ApplyMovement)
                        .before(PlatformerControllerSystems::ApplyJump),
                    PlatformerGroundPoundSystems::SyncState
                        .after(PlatformerControllerSystems::SyncState),
                ),
            )
            .add_systems(
                self.update_schedule,
                (
                    resolve_ground_pound_directives
                        .in_set(PlatformerGroundPoundSystems::ResolveDirectives),
                    apply_ground_pound
                        .in_set(PlatformerControllerSystems::ApplyAbilityMotion)
                        .in_set(PlatformerGroundPoundSystems::ApplyGroundPound),
                    sync_ground_pound_state.in_set(PlatformerGroundPoundSystems::SyncState),
                    emit_ground_pound_messages.in_set(PlatformerGroundPoundSystems::SyncState),
                    clear_ground_pound_intents.in_set(PlatformerGroundPoundSystems::SyncState),
                )
                    .run_if(runtime_is_active),
            );
    }
}

#[derive(Clone, Debug, Message)]
pub struct GroundPoundStarted {
    pub entity: Entity,
}

#[derive(Clone, Debug, Message)]
pub struct GroundPoundImpact {
    pub entity: Entity,
    pub impact_speed: f32,
}

#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component, Debug, Default)]
pub struct PlatformerGroundPoundConfig {
    pub hover_duration: f32,
    pub fall_speed: f32,
    pub cancel_horizontal_speed: bool,
    pub impact_stun_duration: f32,
}

impl Default for PlatformerGroundPoundConfig {
    fn default() -> Self {
        Self {
            hover_duration: 0.08,
            fall_speed: 600.0,
            cancel_horizontal_speed: true,
            impact_stun_duration: 0.1,
        }
    }
}

#[derive(Component, Clone, Debug, Reflect, Default)]
#[reflect(Component, Debug, Default)]
pub struct PlatformerGroundPoundIntent {
    pub pressed: bool,
}

#[derive(Clone, Copy, Debug, Reflect, PartialEq, Eq, Default)]
#[reflect(Debug, PartialEq, Default)]
pub enum PlatformerGroundPoundPhase {
    #[default]
    Idle,
    Hovering,
    Slamming,
    ImpactStun,
}

#[derive(Component, Clone, Debug, Reflect, PartialEq)]
#[reflect(Component, Debug, Default, PartialEq)]
pub struct PlatformerGroundPoundState {
    pub phase: PlatformerGroundPoundPhase,
    pub hover_remaining: f32,
    pub impact_stun_remaining: f32,
}

impl Default for PlatformerGroundPoundState {
    fn default() -> Self {
        Self {
            phase: PlatformerGroundPoundPhase::Idle,
            hover_remaining: 0.0,
            impact_stun_remaining: 0.0,
        }
    }
}

#[derive(Bundle)]
pub struct PlatformerGroundPoundBundle {
    config: PlatformerGroundPoundConfig,
    intent: PlatformerGroundPoundIntent,
    state: PlatformerGroundPoundState,
    runtime: PlatformerGroundPoundRuntimeState,
}

impl Default for PlatformerGroundPoundBundle {
    fn default() -> Self {
        Self::with_config(PlatformerGroundPoundConfig::default())
    }
}

impl PlatformerGroundPoundBundle {
    pub fn with_config(config: PlatformerGroundPoundConfig) -> Self {
        Self {
            config,
            intent: PlatformerGroundPoundIntent::default(),
            state: PlatformerGroundPoundState::default(),
            runtime: PlatformerGroundPoundRuntimeState::default(),
        }
    }
}

#[derive(Component, Clone, Debug, Default)]
pub(crate) struct PlatformerGroundPoundRuntimeState {
    phase: PlatformerGroundPoundPhase,
    hover_remaining: f32,
    impact_stun_remaining: f32,
    pending_started: bool,
    pending_impact_speed: Option<f32>,
}

impl PlatformerGroundPoundRuntimeState {
    pub(crate) fn active(&self) -> bool {
        self.phase != PlatformerGroundPoundPhase::Idle
    }

    pub(crate) fn cancel(&mut self) {
        self.phase = PlatformerGroundPoundPhase::Idle;
        self.hover_remaining = 0.0;
        self.impact_stun_remaining = 0.0;
    }
}

fn resolve_ground_pound_directives(
    mut controllers: Query<
        (
            &mut PlatformerControllerDirectives,
            &PlatformerGroundPoundRuntimeState,
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

fn apply_ground_pound(
    time: Res<Time>,
    composition: Res<PlatformerAbilityComposition>,
    mut controllers: Query<
        (
            &mut PlatformerControllerDirectives,
            &PlatformerGroundPoundConfig,
            &PlatformerGroundPoundIntent,
            &mut LinearVelocity,
            &crate::components::PlatformerControllerRuntimeState,
            &mut PlatformerGroundPoundRuntimeState,
            Option<&mut PlatformerDashRuntimeState>,
            Option<&mut PlatformerGrappleRuntimeState>,
        ),
        With<PlatformerController>,
    >,
) {
    let delta_secs = time.delta_secs();

    for (
        mut directives,
        config,
        intent,
        mut velocity,
        controller_runtime,
        mut runtime,
        dash_runtime,
        grapple_runtime,
    ) in &mut controllers
    {
        runtime.pending_started = false;
        runtime.pending_impact_speed = None;

        match runtime.phase {
            PlatformerGroundPoundPhase::ImpactStun => {
                runtime.impact_stun_remaining =
                    (runtime.impact_stun_remaining - delta_secs).max(0.0);
                velocity.0 = Vec2::ZERO;
                directives.suppress_jump_logic = true;
                directives.suppress_wall_interactions = true;
                if runtime.impact_stun_remaining == 0.0 {
                    runtime.phase = PlatformerGroundPoundPhase::Idle;
                }
                continue;
            }
            PlatformerGroundPoundPhase::Slamming if controller_runtime.pre_ground.is_some() => {
                let impact_speed = velocity.y.abs();
                runtime.phase = PlatformerGroundPoundPhase::ImpactStun;
                runtime.hover_remaining = 0.0;
                runtime.impact_stun_remaining = config.impact_stun_duration;
                runtime.pending_impact_speed = Some(impact_speed);
                velocity.0 = Vec2::ZERO;
                directives.suppress_jump_logic = true;
                directives.suppress_wall_interactions = true;
                continue;
            }
            PlatformerGroundPoundPhase::Hovering => {
                runtime.hover_remaining = (runtime.hover_remaining - delta_secs).max(0.0);
                velocity.0 = Vec2::ZERO;
                directives.suppress_jump_logic = true;
                directives.suppress_wall_interactions = true;
                if runtime.hover_remaining == 0.0 {
                    runtime.phase = PlatformerGroundPoundPhase::Slamming;
                }
                continue;
            }
            PlatformerGroundPoundPhase::Slamming => {
                if config.cancel_horizontal_speed {
                    velocity.x = 0.0;
                }
                velocity.y = -config.fall_speed;
                directives.suppress_jump_logic = true;
                directives.suppress_wall_interactions = true;
                continue;
            }
            PlatformerGroundPoundPhase::Idle => {}
        }

        if !intent.pressed || controller_runtime.pre_ground.is_some() {
            continue;
        }

        let dash_active = dash_runtime
            .as_ref()
            .is_some_and(|runtime| runtime.active());
        let grapple_active = grapple_runtime
            .as_ref()
            .is_some_and(|runtime| runtime.active());
        let resolution = composition.0.resolve_activation(
            PlatformerAbilityKind::GroundPound,
            ability_activity(dash_active, runtime.active(), grapple_active),
        );
        if !resolution.allow_requested {
            continue;
        }

        if matches!(resolution.dash, PlatformerAbilityConflictAction::Cancel)
            && let Some(mut runtime) = dash_runtime
        {
            runtime.cancel();
        }
        if matches!(resolution.grapple, PlatformerAbilityConflictAction::Cancel)
            && let Some(mut runtime) = grapple_runtime
        {
            runtime.cancel();
        }

        if config.hover_duration > 0.0 {
            runtime.phase = PlatformerGroundPoundPhase::Hovering;
            runtime.hover_remaining = config.hover_duration;
            velocity.0 = Vec2::ZERO;
        } else {
            runtime.phase = PlatformerGroundPoundPhase::Slamming;
            if config.cancel_horizontal_speed {
                velocity.x = 0.0;
            }
            velocity.y = -config.fall_speed;
        }
        runtime.pending_started = true;
        directives.suppress_jump_logic = true;
        directives.suppress_wall_interactions = true;
    }
}

fn sync_ground_pound_state(
    mut controllers: Query<
        (
            &PlatformerGroundPoundRuntimeState,
            &mut PlatformerGroundPoundState,
        ),
        With<PlatformerController>,
    >,
) {
    for (runtime, mut state) in &mut controllers {
        state.phase = runtime.phase;
        state.hover_remaining = runtime.hover_remaining;
        state.impact_stun_remaining = runtime.impact_stun_remaining;
    }
}

fn emit_ground_pound_messages(
    mut query: Query<(Entity, &mut PlatformerGroundPoundRuntimeState), With<PlatformerController>>,
    mut started: MessageWriter<GroundPoundStarted>,
    mut impact: MessageWriter<GroundPoundImpact>,
) {
    for (entity, mut runtime) in &mut query {
        if runtime.pending_started {
            runtime.pending_started = false;
            started.write(GroundPoundStarted { entity });
        }

        if let Some(impact_speed) = runtime.pending_impact_speed.take() {
            impact.write(GroundPoundImpact {
                entity,
                impact_speed,
            });
        }
    }
}

fn clear_ground_pound_intents(
    mut intents: Query<&mut PlatformerGroundPoundIntent, With<PlatformerController>>,
) {
    for mut intent in &mut intents {
        intent.pressed = false;
    }
}
