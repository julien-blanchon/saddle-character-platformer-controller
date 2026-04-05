use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    PlatformerController, PlatformerControllerConfig, PlatformerMovementIntent,
    components::PlatformerControllerRuntimeState,
};

/// Handles ground-pound activation, hover phase, slam phase, and impact stun.
///
/// During the hover, velocity is zeroed. During the slam, velocity is set to
/// `(0, -fall_speed)`. On landing while ground-pounding, impact stun freezes
/// movement for a configurable duration.
pub(crate) fn apply_ground_pound(
    time: Res<Time>,
    mut controllers: Query<
        (
            &PlatformerControllerConfig,
            &PlatformerMovementIntent,
            &mut LinearVelocity,
            &mut PlatformerControllerRuntimeState,
        ),
        With<PlatformerController>,
    >,
) {
    let delta_secs = time.delta_secs();

    for (config, intent, mut velocity, mut runtime) in &mut controllers {
        // Tick down impact stun
        if runtime.ground_pound_impact_stun > 0.0 {
            runtime.ground_pound_impact_stun =
                (runtime.ground_pound_impact_stun - delta_secs).max(0.0);
            velocity.0 = Vec2::ZERO;
            continue;
        }

        // If grounded and ground pound is active, land the ground pound
        if runtime.ground_pound_active && runtime.pre_ground.is_some() {
            let impact_speed = velocity.y.abs();
            runtime.ground_pound_active = false;
            runtime.ground_pound_hover_remaining = 0.0;
            runtime.ground_pound_impact_stun = config.ground_pound.impact_stun_duration;
            runtime.pending_ground_pound_impact_speed = Some(impact_speed);
            velocity.0 = Vec2::ZERO;
            continue;
        }

        // Hover phase
        if runtime.ground_pound_hover_remaining > 0.0 {
            runtime.ground_pound_hover_remaining =
                (runtime.ground_pound_hover_remaining - delta_secs).max(0.0);
            velocity.0 = Vec2::ZERO;
            if runtime.ground_pound_hover_remaining == 0.0 {
                // Transition to slam
                runtime.ground_pound_active = true;
            }
            continue;
        }

        // Active slam phase — override velocity
        if runtime.ground_pound_active {
            if config.ground_pound.cancel_horizontal_speed {
                velocity.x = 0.0;
            }
            velocity.y = -config.ground_pound.fall_speed;
            continue;
        }

        // Activation — must be airborne (not grounded, not dashing)
        if intent.ground_pound_pressed
            && runtime.pre_ground.is_none()
            && runtime.dash_time_remaining <= 0.0
        {
            let hover = config.ground_pound.hover_duration;
            if hover > 0.0 {
                runtime.ground_pound_hover_remaining = hover;
                velocity.0 = Vec2::ZERO;
            } else {
                runtime.ground_pound_active = true;
                if config.ground_pound.cancel_horizontal_speed {
                    velocity.x = 0.0;
                }
                velocity.y = -config.ground_pound.fall_speed;
            }
            runtime.pending_ground_pound_started = true;
        }
    }
}
