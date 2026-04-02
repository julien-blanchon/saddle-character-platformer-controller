use avian2d::{character_controller::move_and_slide::MoveAndSlideHitResponse, prelude::*};
use bevy::{ecs::query::Has, prelude::*};

use crate::{
    PlatformVelocityInheritance, PlatformerController, PlatformerControllerConfig,
    PlatformerJumpKind, PlatformerMovementIntent, PlatformerOneWayPlatform, PlatformerWallSide,
    components::{PendingJumpMessage, PendingWallJumpMessage, PlatformerControllerRuntimeState},
    helpers::{
        approach_scalar, inherited_platform_velocity, move_and_slide_config,
        wall_contact_from_hits, wall_input_matches,
    },
};

use super::sensing::probe_contacts;

type SurfaceQueryFilter = (
    Option<&'static Position>,
    Option<&'static Rotation>,
    Option<&'static LinearVelocity>,
    Has<PlatformerOneWayPlatform>,
);

pub(crate) fn apply_horizontal_movement(
    time: Res<Time>,
    mut controllers: Query<
        (
            &PlatformerControllerConfig,
            &PlatformerMovementIntent,
            &mut LinearVelocity,
            &PlatformerControllerRuntimeState,
        ),
        With<PlatformerController>,
    >,
) {
    let delta_secs = time.delta_secs();

    for (config, intent, mut velocity, runtime) in &mut controllers {
        let grounded = runtime.pre_ground.is_some();
        let inherited_velocity = if grounded {
            inherited_platform_velocity(
                runtime.support_velocity,
                config.platforms.velocity_inheritance,
            )
        } else {
            Vec2::ZERO
        };

        let mut local_velocity = velocity.0 - inherited_velocity;
        let mut move_axis = intent.move_axis;

        if runtime.wall_jump_lock_remaining > 0.0 {
            move_axis *= config.walls.wall_jump_steering_factor;
        }

        let accelerating = move_axis.abs() > 0.01;
        let acceleration = if grounded {
            if accelerating {
                config.movement.ground_acceleration
            } else {
                config.movement.ground_deceleration
            }
        } else if accelerating {
            config.movement.air_acceleration
        } else {
            config.movement.air_deceleration
        };

        let mut target_speed = move_axis * config.movement.max_speed;
        if !grounded && local_velocity.y.abs() <= config.jump.apex_velocity_threshold {
            target_speed *= config.movement.apex_air_control_multiplier;
        }

        local_velocity.x =
            approach_scalar(local_velocity.x, target_speed, acceleration * delta_secs);
        velocity.x = local_velocity.x + inherited_velocity.x;

        if grounded && velocity.y <= inherited_velocity.y {
            velocity.y = inherited_velocity.y;
        }
    }
}

pub(crate) fn apply_jump_logic(
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
        let grounded = runtime.pre_ground.is_some();
        let wall_contact = wall_contact_from_hits(
            runtime.pre_left_wall.clone(),
            runtime.pre_right_wall.clone(),
        );
        let wall_slide_active = wall_contact.as_ref().is_some_and(|wall| {
            velocity.y < 0.0
                && (!config.walls.wall_slide_requires_input
                    || wall_input_matches(wall.side, intent.move_axis))
        });
        let jump_requested = intent.jump_pressed || runtime.jump_buffer_remaining > 0.0;

        if jump_requested {
            let used_buffer = !intent.jump_pressed && runtime.jump_buffer_remaining > 0.0;

            if grounded {
                start_ground_or_air_jump(
                    &mut velocity,
                    &mut runtime,
                    PlatformerJumpKind::Ground,
                    config.jump.jump_speed(),
                    used_buffer,
                );
            } else if let Some(wall) = wall_contact.as_ref().filter(|wall| {
                !config.walls.wall_slide_requires_input
                    || wall_input_matches(wall.side, intent.move_axis)
            }) {
                let horizontal_speed = match wall.side {
                    PlatformerWallSide::Left => config.walls.wall_jump_horizontal_speed,
                    PlatformerWallSide::Right => -config.walls.wall_jump_horizontal_speed,
                };
                velocity.x = horizontal_speed;
                velocity.y = config.walls.wall_jump_vertical_speed;
                runtime.jump_buffer_remaining = 0.0;
                runtime.coyote_time_remaining = 0.0;
                runtime.wall_jump_lock_remaining = config.walls.wall_jump_steering_lock_time;
                runtime.pending_jump = Some(PendingJumpMessage {
                    kind: PlatformerJumpKind::Wall,
                    velocity: velocity.0,
                    used_buffer,
                });
                runtime.pending_wall_jump = Some(PendingWallJumpMessage {
                    side: wall.side,
                    velocity: velocity.0,
                });
            } else if runtime.coyote_time_remaining > 0.0 {
                start_ground_or_air_jump(
                    &mut velocity,
                    &mut runtime,
                    PlatformerJumpKind::Coyote,
                    config.jump.jump_speed(),
                    used_buffer,
                );
            } else if runtime.remaining_air_jumps > 0 {
                runtime.remaining_air_jumps -= 1;
                runtime.pending_air_jump_consumed = Some(runtime.remaining_air_jumps);
                start_ground_or_air_jump(
                    &mut velocity,
                    &mut runtime,
                    PlatformerJumpKind::Air,
                    config.jump.jump_speed(),
                    used_buffer,
                );
            }
        }

        let grounded_inherited_y = if grounded {
            inherited_platform_velocity(runtime.support_velocity, PlatformVelocityInheritance::Full)
                .y
        } else {
            0.0
        };
        let skip_gravity = grounded && velocity.y <= grounded_inherited_y + 0.1;

        if skip_gravity {
            continue;
        }

        let mut gravity_multiplier = if velocity.y > config.jump.apex_velocity_threshold {
            if intent.jump_held {
                config.jump.rise_gravity_multiplier
            } else {
                config.jump.low_jump_gravity_multiplier
            }
        } else if velocity.y < -config.jump.apex_velocity_threshold {
            config.jump.fall_gravity_multiplier
        } else {
            config.jump.apex_gravity_multiplier
        };

        if wall_slide_active && velocity.y <= 0.0 {
            gravity_multiplier = gravity_multiplier.min(config.walls.wall_slide_gravity_multiplier);
        }

        velocity.y -= config.jump.base_gravity() * gravity_multiplier * delta_secs;
    }
}

pub(crate) fn apply_wall_interactions(
    mut controllers: Query<
        (
            &PlatformerControllerConfig,
            &PlatformerMovementIntent,
            &mut LinearVelocity,
            &PlatformerControllerRuntimeState,
        ),
        With<PlatformerController>,
    >,
) {
    for (config, intent, mut velocity, runtime) in &mut controllers {
        if runtime.pre_ground.is_some() {
            continue;
        }

        let wall_contact = wall_contact_from_hits(
            runtime.pre_left_wall.clone(),
            runtime.pre_right_wall.clone(),
        );
        let wall_slide_active = wall_contact.as_ref().is_some_and(|wall| {
            velocity.y < 0.0
                && (!config.walls.wall_slide_requires_input
                    || wall_input_matches(wall.side, intent.move_axis))
        });

        if wall_slide_active {
            velocity.y = velocity.y.max(-config.walls.wall_slide_terminal_speed);
        }
    }
}

pub(crate) fn move_controllers(
    time: Res<Time>,
    one_way_platforms: Query<
        Entity,
        (
            With<PlatformerOneWayPlatform>,
            Without<PlatformerController>,
        ),
    >,
    surfaces: Query<SurfaceQueryFilter, Without<PlatformerController>>,
    mut controller_params: ParamSet<(
        MoveAndSlide,
        Query<
            (
                Entity,
                &Collider,
                &mut Position,
                &Rotation,
                &mut LinearVelocity,
                &PlatformerControllerConfig,
                &mut PlatformerControllerRuntimeState,
                &mut Transform,
            ),
            With<PlatformerController>,
        >,
    )>,
) {
    let delta = time.delta();
    let delta_secs = time.delta_secs().max(f32::EPSILON);
    let controller_entities: Vec<_> = controller_params
        .p1()
        .iter()
        .map(|(entity, ..)| entity)
        .collect();

    for entity in controller_entities {
        let (
            collider,
            mut next_position,
            rotation,
            mut next_velocity,
            config,
            runtime_snapshot,
            mut next_translation,
        ) = {
            let mut controllers = controller_params.p1();
            let (_, collider, position, rotation, velocity, config, runtime, transform) =
                controllers
                    .get_mut(entity)
                    .expect("controller entity should remain alive during movement");
            (
                collider.clone(),
                position.0,
                *rotation,
                velocity.0,
                config.clone(),
                runtime.clone(),
                transform.translation,
            )
        };

        let mut excluded_entities = vec![entity];
        if next_velocity.y > 0.0 || runtime_snapshot.drop_through_remaining > 0.0 {
            excluded_entities.extend(one_way_platforms.iter());
        }
        let filter = SpatialQueryFilter::from_excluded_entities(excluded_entities);

        let output = {
            let move_and_slide = controller_params.p0();
            move_and_slide.move_and_slide(
                &collider,
                next_position,
                rotation.as_radians(),
                next_velocity,
                delta,
                &move_and_slide_config(&config.move_and_slide),
                &filter,
                |_| MoveAndSlideHitResponse::Accept,
            )
        };

        next_position = output.position;
        next_translation.x = output.position.x;
        next_translation.y = output.position.y;
        next_velocity = output.projected_velocity;

        let was_grounded = runtime_snapshot.pre_ground.is_some();
        let mut contacts = probe_contacts(
            entity,
            &collider,
            next_position,
            rotation,
            next_velocity,
            &config,
            &runtime_snapshot,
            &controller_params.p0().spatial_query,
            &surfaces,
            config.sensing.ground_probe_distance,
            runtime_snapshot.drop_through_remaining > 0.0 || next_velocity.y > 0.0,
            delta_secs,
        );

        if contacts.ground.is_none() && next_velocity.y <= 0.0 {
            let snap_contacts = probe_contacts(
                entity,
                &collider,
                next_position,
                rotation,
                next_velocity,
                &config,
                &runtime_snapshot,
                &controller_params.p0().spatial_query,
                &surfaces,
                config
                    .sensing
                    .ground_snap_distance
                    .max(config.sensing.ground_probe_distance),
                runtime_snapshot.drop_through_remaining > 0.0,
                delta_secs,
            );

            if let Some(snap_ground) = snap_contacts.ground.as_ref() {
                next_position.y -= snap_ground.distance;
                next_translation.y = next_position.y;
                contacts = probe_contacts(
                    entity,
                    &collider,
                    next_position,
                    rotation,
                    next_velocity,
                    &config,
                    &runtime_snapshot,
                    &controller_params.p0().spatial_query,
                    &surfaces,
                    config.sensing.ground_probe_distance,
                    runtime_snapshot.drop_through_remaining > 0.0,
                    delta_secs,
                );
            }
        }

        let mut controllers = controller_params.p1();
        let (_, _, mut position, _, mut velocity, _, mut runtime, mut transform) = controllers
            .get_mut(entity)
            .expect("controller entity should remain alive during movement writeback");
        position.0 = next_position;
        transform.translation = next_translation;
        velocity.0 = next_velocity;
        runtime.ground = contacts.ground.clone();
        runtime.left_wall = contacts.left_wall.clone();
        runtime.right_wall = contacts.right_wall.clone();
        runtime.support_velocity = contacts.support_velocity;
        runtime.support_position = contacts.support_position;

        if runtime.ground.is_some() {
            runtime.coyote_time_remaining = config.jump.coyote_time;
            runtime.remaining_air_jumps = config.jump.max_air_jumps;
            velocity.y = velocity.y.max(
                inherited_platform_velocity(
                    runtime.support_velocity,
                    config.platforms.velocity_inheritance,
                )
                .y,
            );
        }

        if !was_grounded && runtime.ground.is_some() {
            runtime.pending_landed_impact_speed = Some(runtime.previous_velocity.y.abs());
            runtime.pending_landed_support = runtime.ground.as_ref().map(|contact| contact.entity);
        }

        if runtime.ground.is_some()
            && runtime.jump_buffer_remaining > 0.0
            && runtime.previous_velocity.y < -1.0
        {
            runtime.jump_buffer_remaining = 0.0;
            runtime.coyote_time_remaining = 0.0;
            runtime.pending_landed_impact_speed = None;
            runtime.pending_landed_support = None;
            runtime.ground = None;
            runtime.support_velocity = Vec2::ZERO;
            velocity.y = config.jump.jump_speed();
            runtime.pending_jump = Some(PendingJumpMessage {
                kind: PlatformerJumpKind::Ground,
                velocity: velocity.0,
                used_buffer: true,
            });
        }
    }
}

fn start_ground_or_air_jump(
    velocity: &mut LinearVelocity,
    runtime: &mut PlatformerControllerRuntimeState,
    kind: PlatformerJumpKind,
    jump_speed: f32,
    used_buffer: bool,
) {
    velocity.y = jump_speed;
    runtime.jump_buffer_remaining = 0.0;
    runtime.coyote_time_remaining = 0.0;
    runtime.pending_jump = Some(PendingJumpMessage {
        kind,
        velocity: velocity.0,
        used_buffer,
    });
}

#[cfg(test)]
#[path = "movement_tests.rs"]
mod tests;
