use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    MoveAndSlideTuning, PlatformVelocityInheritance, PlatformerContact, PlatformerWallContact,
    PlatformerWallSide,
};

pub(crate) fn approach_scalar(current: f32, target: f32, max_delta: f32) -> f32 {
    if current < target {
        (current + max_delta).min(target)
    } else {
        (current - max_delta).max(target)
    }
}

pub(crate) fn clamp_axis(axis: f32) -> f32 {
    axis.clamp(-1.0, 1.0)
}

pub(crate) fn walkable_min_normal_y(max_walkable_angle: f32) -> f32 {
    max_walkable_angle.cos()
}

pub(crate) fn is_walkable(normal: Vec2, max_walkable_angle: f32) -> bool {
    normal.y >= walkable_min_normal_y(max_walkable_angle)
}

pub(crate) fn collider_half_extents(collider: &Collider) -> Vec2 {
    collider.aabb(Vec2::ZERO, 0.0).size() * 0.5
}

pub(crate) fn move_and_slide_config(
    tuning: &MoveAndSlideTuning,
) -> avian2d::character_controller::move_and_slide::MoveAndSlideConfig {
    avian2d::character_controller::move_and_slide::MoveAndSlideConfig {
        move_and_slide_iterations: tuning.move_and_slide_iterations,
        depenetration_iterations: tuning.depenetration_iterations,
        max_depenetration_error: tuning.max_depenetration_error,
        max_planes: tuning.max_planes,
        skin_width: tuning.skin_width,
        ..default()
    }
}

pub(crate) fn inherited_platform_velocity(
    velocity: Vec2,
    mode: PlatformVelocityInheritance,
) -> Vec2 {
    match mode {
        PlatformVelocityInheritance::Horizontal => Vec2::new(velocity.x, 0.0),
        PlatformVelocityInheritance::Full => velocity,
        PlatformVelocityInheritance::None => Vec2::ZERO,
    }
}

pub(crate) fn should_block_one_way(
    one_way: bool,
    platform_up: Vec2,
    hit_normal: Vec2,
    velocity: Vec2,
    drop_through_remaining: f32,
    normal_alignment: f32,
) -> bool {
    if !one_way {
        return true;
    }
    if drop_through_remaining > 0.0 {
        return false;
    }
    if hit_normal
        .normalize_or_zero()
        .dot(platform_up.normalize_or_zero())
        < normal_alignment
    {
        return false;
    }
    velocity.dot(platform_up.normalize_or_zero()) <= 0.0
}

pub(crate) fn wall_contact_from_hits(
    left: Option<PlatformerContact>,
    right: Option<PlatformerContact>,
) -> Option<PlatformerWallContact> {
    if let Some(contact) = left {
        return Some(PlatformerWallContact {
            side: PlatformerWallSide::Left,
            contact,
        });
    }
    right.map(|contact| PlatformerWallContact {
        side: PlatformerWallSide::Right,
        contact,
    })
}

pub(crate) fn wall_input_matches(side: PlatformerWallSide, move_axis: f32) -> bool {
    match side {
        PlatformerWallSide::Left => move_axis < -0.05,
        PlatformerWallSide::Right => move_axis > 0.05,
    }
}

pub(crate) fn sign_or_fallback(value: f32, fallback: f32) -> f32 {
    if value.abs() > 0.001 {
        value.signum()
    } else if fallback.abs() > 0.001 {
        fallback.signum()
    } else {
        1.0
    }
}

#[cfg(test)]
#[path = "helpers_tests.rs"]
mod helpers_tests;
