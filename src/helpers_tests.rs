use bevy::prelude::*;

use crate::{
    MoveAndSlideTuning, PlatformVelocityInheritance, PlatformerContact, PlatformerWallContact,
    PlatformerWallSide,
    helpers::{
        clamp_axis, inherited_platform_velocity, is_walkable, move_and_slide_config,
        should_block_one_way, walkable_min_normal_y, wall_contact_from_hits, wall_input_matches,
    },
};

#[test]
fn walkable_threshold_matches_angle() {
    let angle = 45.0_f32.to_radians();
    let min_normal_y = walkable_min_normal_y(angle);

    assert!(is_walkable(Vec2::new(0.0, min_normal_y + 0.01), angle));
    assert!(!is_walkable(Vec2::new(0.0, min_normal_y - 0.01), angle));
}

#[test]
fn input_axis_is_clamped() {
    assert_eq!(clamp_axis(-4.0), -1.0);
    assert_eq!(clamp_axis(0.35), 0.35);
    assert_eq!(clamp_axis(2.5), 1.0);
}

#[test]
fn inherited_velocity_respects_mode() {
    let velocity = Vec2::new(3.0, 5.0);

    assert_eq!(
        inherited_platform_velocity(velocity, PlatformVelocityInheritance::Horizontal),
        Vec2::new(3.0, 0.0)
    );
    assert_eq!(
        inherited_platform_velocity(velocity, PlatformVelocityInheritance::Full),
        velocity
    );
    assert_eq!(
        inherited_platform_velocity(velocity, PlatformVelocityInheritance::None),
        Vec2::ZERO
    );
}

#[test]
fn one_way_filtering_requires_descending_motion() {
    assert!(should_block_one_way(
        false,
        Vec2::Y,
        Vec2::Y,
        Vec2::new(0.0, -1.0),
        0.0,
        0.7,
    ));
    assert!(!should_block_one_way(
        true,
        Vec2::Y,
        Vec2::Y,
        Vec2::new(0.0, 8.0),
        0.0,
        0.7,
    ));
    assert!(!should_block_one_way(
        true,
        Vec2::Y,
        Vec2::Y,
        Vec2::new(0.0, -1.0),
        0.2,
        0.7,
    ));
}

#[test]
fn move_and_slide_tuning_maps_to_avian_config() {
    let tuning = MoveAndSlideTuning {
        skin_width: 0.05,
        move_and_slide_iterations: 7,
        depenetration_iterations: 3,
        max_depenetration_error: 0.02,
        max_planes: 9,
    };
    let mapped = move_and_slide_config(&tuning);

    assert_eq!(mapped.skin_width, 0.05);
    assert_eq!(mapped.move_and_slide_iterations, 7);
    assert_eq!(mapped.depenetration_iterations, 3);
    assert_eq!(mapped.max_depenetration_error, 0.02);
    assert_eq!(mapped.max_planes, 9);
}

#[test]
fn wall_contact_prefers_left_then_right() {
    let left = PlatformerContact {
        entity: Entity::from_bits(1),
        point: Vec2::ZERO,
        normal: Vec2::X,
        distance: 1.0,
    };
    let right = PlatformerContact {
        entity: Entity::from_bits(2),
        point: Vec2::ZERO,
        normal: Vec2::NEG_X,
        distance: 1.0,
    };

    assert_eq!(
        wall_contact_from_hits(Some(left.clone()), Some(right.clone())),
        Some(PlatformerWallContact {
            side: PlatformerWallSide::Left,
            contact: left,
        })
    );
    assert_eq!(
        wall_contact_from_hits(None, Some(right.clone())),
        Some(PlatformerWallContact {
            side: PlatformerWallSide::Right,
            contact: right,
        })
    );
}

#[test]
fn wall_input_matching_tracks_contact_side() {
    assert!(wall_input_matches(PlatformerWallSide::Left, -1.0));
    assert!(!wall_input_matches(PlatformerWallSide::Left, 0.2));
    assert!(wall_input_matches(PlatformerWallSide::Right, 1.0));
    assert!(!wall_input_matches(PlatformerWallSide::Right, -0.3));
}
