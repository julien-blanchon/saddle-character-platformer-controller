use avian2d::prelude::{LinearVelocity, Position, Rotation};
use bevy::prelude::*;

use crate::{
    PlatformerController, PlatformerControllerConfig, PlatformerControllerState,
    PlatformerMovementIntent,
    components::{PlatformerControllerRuntimeState, runtime_from_config},
    helpers::{clamp_axis, sign_or_fallback},
};

pub(crate) fn prepare_intents(
    time: Res<Time>,
    mut query: Query<
        (
            &PlatformerControllerConfig,
            &mut PlatformerMovementIntent,
            &mut PlatformerControllerRuntimeState,
            &mut PlatformerControllerState,
            &mut LinearVelocity,
            &mut Position,
            &mut Rotation,
            Option<&Transform>,
        ),
        With<PlatformerController>,
    >,
) {
    let delta_secs = time.delta_secs();

    for (
        config,
        mut intent,
        mut runtime,
        mut state,
        mut velocity,
        mut position,
        mut rotation,
        transform,
    ) in &mut query
    {
        if !runtime.initialized {
            *runtime = runtime_from_config(config);
            runtime.initialized = true;
            state.remaining_air_jumps = config.jump.max_air_jumps;
            state.remaining_dashes = config.dash.max_charges;

            if let Some(transform) = transform {
                position.0 = transform.translation.xy();
                *rotation = Rotation::radians(transform.rotation.to_euler(EulerRot::XYZ).2);
            }
        }

        runtime.previous_velocity = velocity.0;
        runtime.pending_jump = None;
        runtime.pending_wall_jump = None;
        runtime.pending_dash = None;
        runtime.pending_air_jump_consumed = None;
        runtime.pending_landed_impact_speed = None;
        runtime.pending_landed_support = None;
        runtime.pending_ground_pound_started = false;
        runtime.pending_ground_pound_impact_speed = None;
        runtime.pending_grapple_started = None;
        runtime.pending_grapple_detached = false;
        runtime.pending_wall_cling_started = None;

        runtime.jump_buffer_remaining = (runtime.jump_buffer_remaining - delta_secs).max(0.0);
        runtime.coyote_time_remaining = (runtime.coyote_time_remaining - delta_secs).max(0.0);
        runtime.wall_jump_lock_remaining = (runtime.wall_jump_lock_remaining - delta_secs).max(0.0);
        runtime.drop_through_remaining = (runtime.drop_through_remaining - delta_secs).max(0.0);
        let was_dashing = runtime.dash_time_remaining > 0.0;
        runtime.dash_time_remaining = (runtime.dash_time_remaining - delta_secs).max(0.0);
        runtime.dash_cooldown_remaining = (runtime.dash_cooldown_remaining - delta_secs).max(0.0);

        if was_dashing && runtime.dash_time_remaining == 0.0 {
            velocity.x *= config.dash.exit_speed_scale;
            if runtime.dash_direction.y.abs() > 0.01 || !config.dash.preserve_vertical_velocity {
                velocity.y *= config.dash.exit_speed_scale;
            }
        }

        intent.move_axis = clamp_axis(intent.move_axis);

        if intent.move_axis.abs() > 0.01 {
            runtime.facing_sign = sign_or_fallback(intent.move_axis, runtime.facing_sign);
        } else if velocity.x.abs() > 0.01 {
            runtime.facing_sign = sign_or_fallback(velocity.x, runtime.facing_sign);
        }

        if intent.jump_pressed {
            runtime.jump_buffer_remaining = config.jump.jump_buffer_time;
        }

        if intent.drop_pressed {
            runtime.drop_through_remaining = config.platforms.drop_through_duration;
        }
    }
}

pub(crate) fn clear_transient_intents(
    mut intents: Query<&mut PlatformerMovementIntent, With<PlatformerController>>,
) {
    for mut intent in &mut intents {
        intent.jump_pressed = false;
        intent.drop_pressed = false;
        intent.dash_pressed = false;
        intent.ground_pound_pressed = false;
        intent.grapple_pressed = false;
        intent.grapple_released = false;
    }
}
