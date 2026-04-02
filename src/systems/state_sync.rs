use avian2d::prelude::LinearVelocity;
use bevy::prelude::*;

use crate::{
    AirJumpConsumed, JumpStarted, Landed, PlatformerController, PlatformerControllerConfig,
    PlatformerControllerState, PlatformerMotionPhase, PlatformerMovementIntent, WallJumpStarted,
    components::PlatformerControllerRuntimeState,
    helpers::{wall_contact_from_hits, wall_input_matches},
};

pub(crate) fn sync_controller_state(
    mut controllers: Query<
        (
            &PlatformerControllerConfig,
            &PlatformerMovementIntent,
            &LinearVelocity,
            &mut PlatformerControllerRuntimeState,
            &mut PlatformerControllerState,
        ),
        With<PlatformerController>,
    >,
) {
    for (config, intent, velocity, mut runtime, mut state) in &mut controllers {
        let wall = wall_contact_from_hits(runtime.left_wall.clone(), runtime.right_wall.clone());
        let is_grounded = runtime.ground.is_some();
        let wall_sliding = !is_grounded
            && wall.as_ref().is_some_and(|wall| {
                velocity.y < 0.0
                    && (!config.walls.wall_slide_requires_input
                        || wall_input_matches(wall.side, intent.move_axis))
            });

        state.ground = runtime.ground.clone();
        state.wall = wall;
        state.support_entity = runtime.ground.as_ref().map(|contact| contact.entity);
        state.support_velocity = runtime.support_velocity;
        state.velocity = velocity.0;
        state.is_grounded = is_grounded;
        state.can_use_coyote_jump = !is_grounded && runtime.coyote_time_remaining > 0.0;
        state.buffered_jump = runtime.jump_buffer_remaining > 0.0;
        state.remaining_air_jumps = runtime.remaining_air_jumps;
        state.coyote_time_remaining = runtime.coyote_time_remaining;
        state.jump_buffer_remaining = runtime.jump_buffer_remaining;
        state.wall_jump_lock_remaining = runtime.wall_jump_lock_remaining;
        state.phase = if is_grounded {
            PlatformerMotionPhase::Grounded
        } else if wall_sliding {
            PlatformerMotionPhase::WallSliding
        } else if velocity.y > config.jump.apex_velocity_threshold {
            PlatformerMotionPhase::Rising
        } else if velocity.y < -config.jump.apex_velocity_threshold {
            PlatformerMotionPhase::Falling
        } else {
            PlatformerMotionPhase::Apex
        };

        if is_grounded {
            runtime.last_support_entity = runtime.ground.as_ref().map(|contact| contact.entity);
            runtime.last_support_position = runtime.support_position;
        } else {
            runtime.last_support_entity = None;
            runtime.last_support_position = None;
        }
    }
}

pub(crate) fn emit_messages(
    mut query: Query<(Entity, &mut PlatformerControllerRuntimeState), With<PlatformerController>>,
    mut jump_started: MessageWriter<JumpStarted>,
    mut wall_jump_started: MessageWriter<WallJumpStarted>,
    mut landed: MessageWriter<Landed>,
    mut air_jump_consumed: MessageWriter<AirJumpConsumed>,
) {
    for (entity, mut runtime) in &mut query {
        if let Some(pending_jump) = runtime.pending_jump.take() {
            jump_started.write(JumpStarted {
                entity,
                kind: pending_jump.kind,
                used_buffer: pending_jump.used_buffer,
                velocity: pending_jump.velocity,
            });
        }

        if let Some(pending_wall_jump) = runtime.pending_wall_jump.take() {
            wall_jump_started.write(WallJumpStarted {
                entity,
                side: pending_wall_jump.side,
                velocity: pending_wall_jump.velocity,
            });
        }

        if let Some(remaining_air_jumps) = runtime.pending_air_jump_consumed.take() {
            air_jump_consumed.write(AirJumpConsumed {
                entity,
                remaining_air_jumps,
            });
        }

        if let Some(impact_speed) = runtime.pending_landed_impact_speed.take() {
            landed.write(Landed {
                entity,
                impact_speed,
                support_entity: runtime.pending_landed_support.take(),
            });
        }
    }
}
