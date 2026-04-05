use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    PlatformerController, PlatformerControllerConfig, PlatformerMovementIntent,
    components::{
        PlatformerControllerRuntimeState, PlatformerGrapplePhase, PlatformerGrapplePoint,
    },
};

/// Handles grapple hook firing, pendulum swing physics, and detachment.
///
/// When attached, the character swings on a rope constraint with gravity.
/// The rope acts as a maximum-distance constraint: velocity is projected
/// tangentially when the character reaches the rope length, creating
/// natural pendulum motion. An optional `pull_speed` pulls the character
/// toward the anchor. Horizontal input adds tangential force for active
/// swing control.
pub(crate) fn apply_grapple(
    time: Res<Time>,
    grapple_points: Query<(Entity, &GlobalTransform), With<PlatformerGrapplePoint>>,
    mut controllers: Query<
        (
            &PlatformerControllerConfig,
            &PlatformerMovementIntent,
            &Position,
            &mut LinearVelocity,
            &mut PlatformerControllerRuntimeState,
        ),
        With<PlatformerController>,
    >,
) {
    let delta_secs = time.delta_secs();

    for (config, intent, position, mut velocity, mut runtime) in &mut controllers {
        match runtime.grapple_phase {
            PlatformerGrapplePhase::Idle => {
                if !intent.grapple_pressed || config.grapple.max_range <= 0.0 {
                    continue;
                }

                // Find the best grapple point within range and aim direction
                let aim = if intent.grapple_direction.length_squared() > 0.01 {
                    intent.grapple_direction.normalize()
                } else {
                    Vec2::new(runtime.facing_sign, 0.5).normalize()
                };

                let player_pos = position.0;
                let max_range_sq = config.grapple.max_range * config.grapple.max_range;
                let cos_threshold = config.grapple.aim_assist_angle.cos();

                let mut best_target = None;
                let mut best_dist_sq = f32::MAX;

                for (_entity, global_transform) in &grapple_points {
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
                        best_target = Some(point);
                    }
                }

                if let Some(target) = best_target {
                    let rope_length = (target - player_pos).length();
                    runtime.grapple_phase = PlatformerGrapplePhase::Pulling {
                        target,
                        rope_length,
                    };
                    runtime.pending_grapple_started = Some(target);
                    // Cancel ground pound when grappling
                    runtime.ground_pound_active = false;
                    runtime.ground_pound_hover_remaining = 0.0;
                }
            }
            PlatformerGrapplePhase::Pulling {
                target,
                mut rope_length,
            } => {
                // Detach on release or jump
                if intent.grapple_released || intent.jump_pressed {
                    velocity.0 *= config.grapple.detach_speed_boost;
                    runtime.grapple_phase = PlatformerGrapplePhase::Idle;
                    runtime.grapple_target_entity = None;
                    runtime.pending_grapple_detached = true;
                    continue;
                }

                // Retract / extend rope
                if intent.grapple_retract {
                    rope_length = (rope_length - config.grapple.retract_speed * delta_secs)
                        .max(config.grapple.min_rope_length);
                }
                if intent.grapple_extend {
                    rope_length += config.grapple.extend_speed * delta_secs;
                }

                let player_pos = position.0;
                let to_target = target - player_pos;
                let distance = to_target.length();

                if distance < config.grapple.min_rope_length {
                    runtime.grapple_phase = PlatformerGrapplePhase::Idle;
                    runtime.grapple_target_entity = None;
                    runtime.pending_grapple_detached = true;
                    continue;
                }

                let radial_dir = to_target / distance;

                // --- Pendulum swing physics ---

                // 1. Apply gravity
                let gravity = config.jump.base_gravity() * config.grapple.swing_gravity_multiplier;
                velocity.y -= gravity * delta_secs;

                // 2. Apply horizontal input as tangential force
                if intent.move_axis.abs() > 0.01 {
                    let tangent = Vec2::new(-radial_dir.y, radial_dir.x);
                    velocity.0 +=
                        tangent * intent.move_axis * config.grapple.swing_input_force * delta_secs;
                }

                // 3. Optional pull toward target
                if config.grapple.pull_speed > 0.0 {
                    velocity.0 += radial_dir * config.grapple.pull_speed * delta_secs;
                }

                // 4. Rope constraint: if at or beyond rope length, project velocity
                //    to be tangential (remove radial-outward component)
                if distance >= rope_length {
                    let radial_speed = velocity.0.dot(radial_dir);
                    if radial_speed < 0.0 {
                        // Moving away from target (outward) — remove that component
                        velocity.0 -= radial_dir * radial_speed;
                    }

                    // Snap inward slightly to prevent drift past rope length
                    let overshoot = distance - rope_length;
                    if overshoot > 0.1 {
                        velocity.0 += radial_dir * (overshoot / delta_secs.max(f32::EPSILON)) * 0.3;
                    }
                }

                runtime.grapple_phase = PlatformerGrapplePhase::Pulling {
                    target,
                    rope_length,
                };
            }
        }
    }
}
