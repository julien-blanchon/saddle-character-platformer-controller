use avian2d::prelude::{LinearVelocity, Position, Rotation};
use bevy::prelude::*;

use crate::{
    PlatformerController, PlatformerControllerConfig, PlatformerControllerState,
    PlatformerMovementIntent,
    components::{PlatformerControllerRuntimeState, runtime_from_config},
    helpers::clamp_axis,
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
        velocity,
        mut position,
        mut rotation,
        transform,
    ) in &mut query
    {
        if !runtime.initialized {
            *runtime = runtime_from_config(config);
            runtime.initialized = true;
            state.remaining_air_jumps = config.jump.max_air_jumps;

            if let Some(transform) = transform {
                position.0 = transform.translation.xy();
                *rotation = Rotation::radians(transform.rotation.to_euler(EulerRot::XYZ).2);
            }
        }

        runtime.previous_velocity = velocity.0;
        runtime.pending_jump = None;
        runtime.pending_wall_jump = None;
        runtime.pending_air_jump_consumed = None;
        runtime.pending_landed_impact_speed = None;
        runtime.pending_landed_support = None;

        runtime.jump_buffer_remaining = (runtime.jump_buffer_remaining - delta_secs).max(0.0);
        runtime.coyote_time_remaining = (runtime.coyote_time_remaining - delta_secs).max(0.0);
        runtime.wall_jump_lock_remaining = (runtime.wall_jump_lock_remaining - delta_secs).max(0.0);
        runtime.drop_through_remaining = (runtime.drop_through_remaining - delta_secs).max(0.0);

        intent.move_axis = clamp_axis(intent.move_axis);

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
    }
}
