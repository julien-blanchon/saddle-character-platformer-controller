use avian2d::prelude::LinearVelocity;
use bevy::{
    ecs::{intern::Interned, schedule::ScheduleLabel},
    prelude::*,
};

use crate::{
    PlatformerController, PlatformerControllerConfig, PlatformerControllerDirectives,
    PlatformerControllerSystems, PlatformerMovementIntent,
    components::PlatformerControllerRuntimeState, helpers::sign_or_fallback,
    systems::activation::runtime_is_active,
};

use super::{
    PlatformerAbilityComposition, PlatformerAbilityConflictAction, PlatformerAbilityKind,
    ability_activity, grapple::PlatformerGrappleRuntimeState,
    ground_pound::PlatformerGroundPoundRuntimeState,
};

#[derive(SystemSet, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum PlatformerDashSystems {
    ResolveDirectives,
    ApplyDash,
    SyncState,
}

pub struct PlatformerDashPlugin {
    pub update_schedule: Interned<dyn ScheduleLabel>,
}

impl PlatformerDashPlugin {
    pub fn new(update_schedule: impl ScheduleLabel) -> Self {
        Self {
            update_schedule: update_schedule.intern(),
        }
    }

    pub fn always_on(update_schedule: impl ScheduleLabel) -> Self {
        Self::new(update_schedule)
    }
}

impl Default for PlatformerDashPlugin {
    fn default() -> Self {
        Self::always_on(Update)
    }
}

impl Plugin for PlatformerDashPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlatformerAbilityComposition>()
            .add_message::<DashStarted>()
            .register_type::<PlatformerDashConfig>()
            .register_type::<PlatformerDashIntent>()
            .register_type::<PlatformerDashState>()
            .configure_sets(
                self.update_schedule,
                (
                    PlatformerDashSystems::ResolveDirectives
                        .after(PlatformerControllerSystems::SenseContacts)
                        .before(PlatformerControllerSystems::ApplyMovement),
                    PlatformerDashSystems::ApplyDash
                        .after(PlatformerControllerSystems::ApplyMovement)
                        .before(PlatformerControllerSystems::ApplyJump),
                    PlatformerDashSystems::SyncState.after(PlatformerControllerSystems::SyncState),
                ),
            )
            .add_systems(
                self.update_schedule,
                (
                    resolve_dash_directives.in_set(PlatformerDashSystems::ResolveDirectives),
                    apply_dash
                        .in_set(PlatformerControllerSystems::ApplyAbilityMotion)
                        .in_set(PlatformerDashSystems::ApplyDash),
                    sync_dash_state.in_set(PlatformerDashSystems::SyncState),
                    emit_dash_messages.in_set(PlatformerDashSystems::SyncState),
                    clear_dash_intents.in_set(PlatformerDashSystems::SyncState),
                )
                    .run_if(runtime_is_active),
            );
    }
}

#[derive(Clone, Debug, Message)]
pub struct DashStarted {
    pub entity: Entity,
    pub direction: Vec2,
    pub velocity: Vec2,
    pub remaining_charges: u32,
}

#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component, Debug, Default)]
pub struct PlatformerDashConfig {
    pub distance: f32,
    pub duration: f32,
    pub cooldown: f32,
    pub max_charges: u32,
    pub refill_on_ground: bool,
    pub allow_ground_dash: bool,
    pub preserve_vertical_velocity: bool,
    pub direction_input_threshold: f32,
    pub exit_speed_scale: f32,
}

impl Default for PlatformerDashConfig {
    fn default() -> Self {
        Self {
            distance: 84.0,
            duration: 0.16,
            cooldown: 0.12,
            max_charges: 1,
            refill_on_ground: true,
            allow_ground_dash: true,
            preserve_vertical_velocity: false,
            direction_input_threshold: 0.2,
            exit_speed_scale: 0.35,
        }
    }
}

impl PlatformerDashConfig {
    pub fn dash_speed(&self) -> f32 {
        self.distance.max(0.0) / self.duration.max(0.001)
    }
}

#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component, Debug, Default)]
pub struct PlatformerDashIntent {
    pub pressed: bool,
    pub direction: Vec2,
}

impl Default for PlatformerDashIntent {
    fn default() -> Self {
        Self {
            pressed: false,
            direction: Vec2::ZERO,
        }
    }
}

#[derive(Component, Clone, Debug, Reflect, PartialEq)]
#[reflect(Component, Debug, Default, PartialEq)]
pub struct PlatformerDashState {
    pub active: bool,
    pub remaining_charges: u32,
    pub time_remaining: f32,
    pub cooldown_remaining: f32,
    pub direction: Vec2,
}

impl Default for PlatformerDashState {
    fn default() -> Self {
        Self {
            active: false,
            remaining_charges: 0,
            time_remaining: 0.0,
            cooldown_remaining: 0.0,
            direction: Vec2::X,
        }
    }
}

#[derive(Bundle)]
pub struct PlatformerDashBundle {
    config: PlatformerDashConfig,
    intent: PlatformerDashIntent,
    state: PlatformerDashState,
    runtime: PlatformerDashRuntimeState,
}

impl Default for PlatformerDashBundle {
    fn default() -> Self {
        Self::with_config(PlatformerDashConfig::default())
    }
}

impl PlatformerDashBundle {
    pub fn with_config(config: PlatformerDashConfig) -> Self {
        let runtime = PlatformerDashRuntimeState::from_config(&config);
        Self {
            config,
            intent: PlatformerDashIntent::default(),
            state: PlatformerDashState::default(),
            runtime,
        }
    }
}

#[derive(Clone, Debug)]
struct PendingDashMessage {
    direction: Vec2,
    velocity: Vec2,
    remaining_charges: u32,
}

#[derive(Component, Clone, Debug)]
pub(crate) struct PlatformerDashRuntimeState {
    initialized: bool,
    time_remaining: f32,
    cooldown_remaining: f32,
    remaining_charges: u32,
    direction: Vec2,
    pending_dash: Option<PendingDashMessage>,
}

impl Default for PlatformerDashRuntimeState {
    fn default() -> Self {
        Self {
            initialized: false,
            time_remaining: 0.0,
            cooldown_remaining: 0.0,
            remaining_charges: 0,
            direction: Vec2::X,
            pending_dash: None,
        }
    }
}

impl PlatformerDashRuntimeState {
    fn from_config(config: &PlatformerDashConfig) -> Self {
        Self {
            remaining_charges: config.max_charges,
            ..default()
        }
    }

    pub(crate) fn active(&self) -> bool {
        self.time_remaining > 0.0
    }

    pub(crate) fn cancel(&mut self) {
        self.time_remaining = 0.0;
    }
}

fn resolve_dash_directives(
    time: Res<Time>,
    mut controllers: Query<
        (
            &PlatformerDashConfig,
            &mut PlatformerControllerDirectives,
            &mut LinearVelocity,
            &mut PlatformerDashRuntimeState,
        ),
        With<PlatformerController>,
    >,
) {
    let delta_secs = time.delta_secs();

    for (config, mut directives, mut velocity, mut runtime) in &mut controllers {
        if !runtime.initialized {
            *runtime = PlatformerDashRuntimeState::from_config(config);
            runtime.initialized = true;
        }

        runtime.pending_dash = None;
        let was_active = runtime.active();
        runtime.time_remaining = (runtime.time_remaining - delta_secs).max(0.0);
        runtime.cooldown_remaining = (runtime.cooldown_remaining - delta_secs).max(0.0);

        if was_active && !runtime.active() {
            velocity.x *= config.exit_speed_scale;
            if runtime.direction.y.abs() > 0.01 || !config.preserve_vertical_velocity {
                velocity.y *= config.exit_speed_scale;
            }
        }

        if runtime.active() {
            directives.suppress_horizontal_movement = true;
            directives.suppress_jump_logic = true;
            directives.suppress_wall_interactions = true;
        }
    }
}

fn apply_dash(
    composition: Res<PlatformerAbilityComposition>,
    mut controllers: Query<
        (
            &PlatformerControllerConfig,
            &PlatformerMovementIntent,
            &mut PlatformerControllerDirectives,
            &PlatformerDashConfig,
            &PlatformerDashIntent,
            &mut LinearVelocity,
            &mut PlatformerControllerRuntimeState,
            &mut PlatformerDashRuntimeState,
            Option<&mut PlatformerGroundPoundRuntimeState>,
            Option<&mut PlatformerGrappleRuntimeState>,
        ),
        With<PlatformerController>,
    >,
) {
    for (
        _controller_config,
        controller_intent,
        mut directives,
        config,
        intent,
        mut velocity,
        mut controller_runtime,
        mut dash_runtime,
        ground_pound_runtime,
        grapple_runtime,
    ) in &mut controllers
    {
        if dash_runtime.active() {
            directives.suppress_jump_logic = true;
            directives.suppress_wall_interactions = true;
            velocity.0 = dash_velocity(config, dash_runtime.direction, velocity.0);
            continue;
        }

        if !intent.pressed
            || config.max_charges == 0
            || dash_runtime.cooldown_remaining > 0.0
            || dash_runtime.remaining_charges == 0
        {
            continue;
        }

        let grounded = controller_runtime.pre_ground.is_some();
        if grounded && !config.allow_ground_dash {
            continue;
        }

        let ground_pound_active = match &ground_pound_runtime {
            Some(runtime) => runtime.active(),
            None => false,
        };
        let grapple_active = match &grapple_runtime {
            Some(runtime) => runtime.active(),
            None => false,
        };
        let resolution = composition.0.resolve_activation(
            PlatformerAbilityKind::Dash,
            ability_activity(dash_runtime.active(), ground_pound_active, grapple_active),
        );
        if !resolution.allow_requested {
            continue;
        }

        if matches!(
            resolution.ground_pound,
            PlatformerAbilityConflictAction::Cancel
        ) && let Some(mut runtime) = ground_pound_runtime
        {
            runtime.cancel();
        }
        if matches!(resolution.grapple, PlatformerAbilityConflictAction::Cancel)
            && let Some(mut runtime) = grapple_runtime
        {
            runtime.cancel();
        }

        let dash_direction = resolve_dash_direction(
            intent,
            controller_intent,
            &dash_runtime,
            velocity.0,
            config,
            controller_runtime.facing_sign,
        );
        velocity.0 = dash_velocity(config, dash_direction, velocity.0);
        dash_runtime.direction = dash_direction;
        dash_runtime.time_remaining = config.duration.max(0.0);
        dash_runtime.cooldown_remaining = config.cooldown.max(0.0);
        controller_runtime.jump_buffer_remaining = 0.0;
        controller_runtime.coyote_time_remaining = 0.0;
        dash_runtime.remaining_charges = dash_runtime.remaining_charges.saturating_sub(1);
        directives.suppress_jump_logic = true;
        directives.suppress_wall_interactions = true;
        dash_runtime.pending_dash = Some(PendingDashMessage {
            direction: dash_direction,
            velocity: velocity.0,
            remaining_charges: dash_runtime.remaining_charges,
        });
    }
}

fn sync_dash_state(
    mut controllers: Query<
        (
            &PlatformerDashConfig,
            &PlatformerControllerRuntimeState,
            &mut PlatformerDashRuntimeState,
            &mut PlatformerDashState,
        ),
        With<PlatformerController>,
    >,
) {
    for (config, controller_runtime, mut runtime, mut state) in &mut controllers {
        if controller_runtime.ground.is_some() && config.refill_on_ground {
            runtime.remaining_charges = config.max_charges;
        }

        state.active = runtime.active();
        state.remaining_charges = runtime.remaining_charges;
        state.time_remaining = runtime.time_remaining;
        state.cooldown_remaining = runtime.cooldown_remaining;
        state.direction = runtime.direction;
    }
}

fn emit_dash_messages(
    mut query: Query<(Entity, &mut PlatformerDashRuntimeState), With<PlatformerController>>,
    mut dash_started: MessageWriter<DashStarted>,
) {
    for (entity, mut runtime) in &mut query {
        if let Some(pending_dash) = runtime.pending_dash.take() {
            dash_started.write(DashStarted {
                entity,
                direction: pending_dash.direction,
                velocity: pending_dash.velocity,
                remaining_charges: pending_dash.remaining_charges,
            });
        }
    }
}

fn clear_dash_intents(mut intents: Query<&mut PlatformerDashIntent, With<PlatformerController>>) {
    for mut intent in &mut intents {
        intent.pressed = false;
    }
}

fn resolve_dash_direction(
    intent: &PlatformerDashIntent,
    controller_intent: &PlatformerMovementIntent,
    runtime: &PlatformerDashRuntimeState,
    velocity: Vec2,
    config: &PlatformerDashConfig,
    facing_sign: f32,
) -> Vec2 {
    if intent.direction.length_squared()
        >= config.direction_input_threshold * config.direction_input_threshold
    {
        return intent.direction.normalize();
    }

    if controller_intent.move_axis.abs() >= config.direction_input_threshold {
        return Vec2::new(controller_intent.move_axis.signum(), 0.0);
    }

    if velocity.x.abs() >= config.direction_input_threshold {
        return Vec2::new(velocity.x.signum(), 0.0);
    }

    if runtime.direction.x.abs() > 0.01 {
        return Vec2::new(runtime.direction.x.signum(), 0.0);
    }

    Vec2::new(sign_or_fallback(facing_sign, 1.0), 0.0)
}

fn dash_velocity(
    config: &PlatformerDashConfig,
    dash_direction: Vec2,
    current_velocity: Vec2,
) -> Vec2 {
    let mut velocity = dash_direction.normalize_or_zero() * config.dash_speed();
    if config.preserve_vertical_velocity && dash_direction.y.abs() <= 0.01 {
        velocity.y = current_velocity.y;
    }
    velocity
}
