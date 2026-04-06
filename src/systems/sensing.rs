use avian2d::prelude::*;
use bevy::{ecs::query::Has, prelude::*};

use crate::{
    PlatformerContact, PlatformerController, PlatformerControllerConfig, PlatformerOneWayPlatform,
    PlatformerWallSide,
    components::{PlatformerControllerRuntimeState, PlatformerSurfaceModifier},
    helpers::{collider_half_extents, is_walkable, should_block_one_way},
};

#[derive(Clone, Debug, Default)]
pub(crate) struct ProbeResults {
    pub ground: Option<PlatformerContact>,
    pub left_wall: Option<PlatformerContact>,
    pub right_wall: Option<PlatformerContact>,
    pub support_velocity: Vec2,
    pub support_position: Option<Vec2>,
}

type SurfaceQueryFilter = (
    Option<&'static Position>,
    Option<&'static Rotation>,
    Option<&'static LinearVelocity>,
    Has<PlatformerOneWayPlatform>,
);

pub(crate) fn sense_pre_movement_contacts(
    time: Res<Time>,
    spatial_query: SpatialQuery,
    mut controllers: Query<
        (
            Entity,
            &Collider,
            &Position,
            &Rotation,
            &LinearVelocity,
            &PlatformerControllerConfig,
            &mut PlatformerControllerRuntimeState,
        ),
        With<PlatformerController>,
    >,
    surfaces: Query<SurfaceQueryFilter, Without<PlatformerController>>,
    surface_modifiers: Query<&PlatformerSurfaceModifier, Without<PlatformerController>>,
) {
    let delta_secs = time.delta_secs().max(f32::EPSILON);

    for (entity, collider, position, rotation, velocity, config, mut runtime) in &mut controllers {
        let contacts = probe_contacts(
            entity,
            collider,
            position.0,
            *rotation,
            velocity.0,
            config,
            &runtime,
            &spatial_query,
            &surfaces,
            config.sensing.ground_probe_distance,
            false,
            delta_secs,
        );

        runtime.pre_ground = contacts.ground.clone();
        runtime.pre_left_wall = contacts.left_wall.clone();
        runtime.pre_right_wall = contacts.right_wall.clone();
        runtime.support_velocity = contacts.support_velocity;
        runtime.support_position = contacts.support_position;

        // Resolve surface modifier from ground contact entity
        runtime.surface_modifier = contacts
            .ground
            .as_ref()
            .and_then(|contact| surface_modifiers.get(contact.entity).ok().cloned());

        if runtime.pre_ground.is_some() {
            runtime.coyote_time_remaining = config.jump.coyote_time;
            runtime.remaining_air_jumps = config.jump.max_air_jumps;
        }
    }
}

pub(crate) fn probe_contacts(
    entity: Entity,
    collider: &Collider,
    position: Vec2,
    rotation: Rotation,
    velocity: Vec2,
    config: &PlatformerControllerConfig,
    runtime: &PlatformerControllerRuntimeState,
    spatial_query: &SpatialQuery,
    surfaces: &Query<SurfaceQueryFilter, Without<PlatformerController>>,
    ground_distance: f32,
    ignore_one_way: bool,
    delta_secs: f32,
) -> ProbeResults {
    let filter = SpatialQueryFilter::from_excluded_entities([entity]);
    let shape_rotation = rotation.as_radians();

    let ground = best_ground_contact(
        collider,
        position,
        shape_rotation,
        velocity,
        runtime,
        config,
        spatial_query,
        surfaces,
        &filter,
        ground_distance,
        ignore_one_way,
    );

    let left_wall = best_wall_contact(
        collider,
        position,
        shape_rotation,
        config,
        spatial_query,
        surfaces,
        &filter,
        PlatformerWallSide::Left,
    );
    let right_wall = best_wall_contact(
        collider,
        position,
        shape_rotation,
        config,
        spatial_query,
        surfaces,
        &filter,
        PlatformerWallSide::Right,
    );

    let mut support_velocity = Vec2::ZERO;
    let mut support_position = None;

    if let Some(contact) = ground.as_ref() {
        if let Ok((surface_position, _, surface_velocity, _)) = surfaces.get(contact.entity) {
            if let Some(surface_velocity) = surface_velocity {
                support_velocity = surface_velocity.0;
            } else if let Some(surface_position) = surface_position {
                support_position = Some(surface_position.0);

                if runtime.last_support_entity == Some(contact.entity) {
                    if let Some(previous_position) = runtime.last_support_position {
                        support_velocity = (surface_position.0 - previous_position) / delta_secs;
                    }
                }
            }

            if support_position.is_none() {
                support_position = surface_position.map(|surface_position| surface_position.0);
            }
        }
    }

    ProbeResults {
        ground,
        left_wall,
        right_wall,
        support_velocity,
        support_position,
    }
}

fn best_ground_contact(
    collider: &Collider,
    position: Vec2,
    shape_rotation: f32,
    velocity: Vec2,
    runtime: &PlatformerControllerRuntimeState,
    config: &PlatformerControllerConfig,
    spatial_query: &SpatialQuery,
    surfaces: &Query<SurfaceQueryFilter, Without<PlatformerController>>,
    filter: &SpatialQueryFilter,
    ground_distance: f32,
    ignore_one_way: bool,
) -> Option<PlatformerContact> {
    let cast_config = ShapeCastConfig::from_max_distance(ground_distance.max(0.0) + 0.001);
    let hits = spatial_query.shape_hits(
        collider,
        position,
        shape_rotation,
        Dir2::NEG_Y,
        8,
        &cast_config,
        filter,
    );

    hits.into_iter()
        .filter_map(|hit| {
            let normal = hit.normal1;
            if !is_walkable(normal, config.sensing.max_walkable_angle) {
                return None;
            }

            let Ok((_, surface_rotation, _, is_one_way)) = surfaces.get(hit.entity) else {
                return None;
            };
            if is_one_way && ignore_one_way {
                return None;
            }
            let platform_up = surface_rotation.copied().unwrap_or_default() * Vec2::Y;
            let one_way_blocks = should_block_one_way(
                is_one_way,
                platform_up,
                normal,
                velocity,
                runtime.drop_through_remaining,
                config.sensing.one_way_normal_alignment,
            );

            if is_one_way && !one_way_blocks {
                return None;
            }

            Some(PlatformerContact {
                entity: hit.entity,
                point: hit.point1,
                normal,
                distance: hit.distance,
            })
        })
        .min_by(|left, right| left.distance.total_cmp(&right.distance))
}

fn best_wall_contact(
    collider: &Collider,
    position: Vec2,
    shape_rotation: f32,
    config: &PlatformerControllerConfig,
    spatial_query: &SpatialQuery,
    surfaces: &Query<SurfaceQueryFilter, Without<PlatformerController>>,
    filter: &SpatialQueryFilter,
    side: PlatformerWallSide,
) -> Option<PlatformerContact> {
    let direction = match side {
        PlatformerWallSide::Left => Dir2::NEG_X,
        PlatformerWallSide::Right => Dir2::X,
    };
    let cast_config =
        ShapeCastConfig::from_max_distance(config.walls.probe_distance.max(0.0) + 0.001);
    let half_extents = collider_half_extents(collider);

    spatial_query
        .shape_hits(
            collider,
            position,
            shape_rotation,
            direction,
            6,
            &cast_config,
            filter,
        )
        .into_iter()
        .filter_map(|hit| {
            let Ok((_, _, _, is_one_way)) = surfaces.get(hit.entity) else {
                return None;
            };
            if is_one_way {
                return None;
            }

            let normal = hit.normal1;
            let normal_matches = match side {
                PlatformerWallSide::Left => normal.x >= config.walls.min_normal_x,
                PlatformerWallSide::Right => normal.x <= -config.walls.min_normal_x,
            };

            if !normal_matches || normal.y.abs() > config.walls.max_vertical_normal_y {
                return None;
            }

            let relative_height =
                ((hit.point2.y - position.y).abs() / half_extents.y.max(0.001)).min(1.0);
            if relative_height > config.walls.max_contact_height_ratio {
                return None;
            }

            Some(PlatformerContact {
                entity: hit.entity,
                point: hit.point1,
                normal,
                distance: hit.distance,
            })
        })
        .min_by(|left, right| left.distance.total_cmp(&right.distance))
}
