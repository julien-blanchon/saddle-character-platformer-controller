use bevy::prelude::*;

use crate::{PlatformerControllerConfig, PlatformerJumpKind};

#[derive(Component, Reflect, Default, Clone, Debug)]
#[reflect(Component, Default, Debug)]
pub struct PlatformerController;

#[derive(Component, Reflect, Clone, Debug)]
#[reflect(Component, Default, Debug)]
pub struct PlatformerMovementIntent {
    pub move_axis: f32,
    pub jump_pressed: bool,
    pub jump_held: bool,
    pub drop_pressed: bool,
    pub dash_pressed: bool,
    pub dash_direction: Vec2,
}

impl Default for PlatformerMovementIntent {
    fn default() -> Self {
        Self {
            move_axis: 0.0,
            jump_pressed: false,
            jump_held: false,
            drop_pressed: false,
            dash_pressed: false,
            dash_direction: Vec2::ZERO,
        }
    }
}

#[derive(Component, Reflect, Clone, Debug, Default)]
#[reflect(Component, Default, Debug)]
pub struct PlatformerOneWayPlatform;

#[derive(Clone, Copy, Debug, Reflect, PartialEq, Eq, Default)]
#[reflect(Debug, PartialEq, Default)]
pub enum PlatformVelocityInheritance {
    #[default]
    Horizontal,
    Full,
    None,
}

#[derive(Clone, Copy, Debug, Reflect, PartialEq, Eq, Default)]
#[reflect(Debug, PartialEq, Default)]
pub enum PlatformerWallSide {
    Left,
    #[default]
    Right,
}

#[derive(Clone, Copy, Debug, Reflect, PartialEq, Eq, Default)]
#[reflect(Debug, PartialEq, Default)]
pub enum PlatformerMotionPhase {
    Grounded,
    Dashing,
    Rising,
    Apex,
    Falling,
    WallSliding,
    #[default]
    Airborne,
}

#[derive(Clone, Debug, Reflect, PartialEq)]
#[reflect(Debug, PartialEq, Default)]
pub struct PlatformerContact {
    pub entity: Entity,
    pub point: Vec2,
    pub normal: Vec2,
    pub distance: f32,
}

impl Default for PlatformerContact {
    fn default() -> Self {
        Self {
            entity: Entity::PLACEHOLDER,
            point: Vec2::ZERO,
            normal: Vec2::Y,
            distance: 0.0,
        }
    }
}

#[derive(Clone, Debug, Reflect, PartialEq, Default)]
#[reflect(Debug, PartialEq, Default)]
pub struct PlatformerWallContact {
    pub side: PlatformerWallSide,
    pub contact: PlatformerContact,
}

#[derive(Component, Reflect, Clone, Debug, PartialEq)]
#[reflect(Component, Debug, PartialEq)]
pub struct PlatformerControllerState {
    pub phase: PlatformerMotionPhase,
    pub ground: Option<PlatformerContact>,
    pub wall: Option<PlatformerWallContact>,
    pub support_entity: Option<Entity>,
    pub support_velocity: Vec2,
    pub velocity: Vec2,
    pub is_grounded: bool,
    pub can_use_coyote_jump: bool,
    pub buffered_jump: bool,
    pub remaining_air_jumps: u32,
    pub remaining_dashes: u32,
    pub coyote_time_remaining: f32,
    pub jump_buffer_remaining: f32,
    pub wall_jump_lock_remaining: f32,
    pub dash_time_remaining: f32,
    pub dash_cooldown_remaining: f32,
}

impl Default for PlatformerControllerState {
    fn default() -> Self {
        Self {
            phase: PlatformerMotionPhase::Airborne,
            ground: None,
            wall: None,
            support_entity: None,
            support_velocity: Vec2::ZERO,
            velocity: Vec2::ZERO,
            is_grounded: false,
            can_use_coyote_jump: false,
            buffered_jump: false,
            remaining_air_jumps: 0,
            remaining_dashes: 0,
            coyote_time_remaining: 0.0,
            jump_buffer_remaining: 0.0,
            wall_jump_lock_remaining: 0.0,
            dash_time_remaining: 0.0,
            dash_cooldown_remaining: 0.0,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct PendingJumpMessage {
    pub kind: PlatformerJumpKind,
    pub velocity: Vec2,
    pub used_buffer: bool,
}

#[derive(Clone, Debug)]
pub(crate) struct PendingWallJumpMessage {
    pub side: PlatformerWallSide,
    pub velocity: Vec2,
}

#[derive(Clone, Debug)]
pub(crate) struct PendingDashMessage {
    pub direction: Vec2,
    pub velocity: Vec2,
    pub remaining_charges: u32,
}

#[derive(Component, Clone, Debug)]
pub(crate) struct PlatformerControllerRuntimeState {
    pub initialized: bool,
    pub previous_velocity: Vec2,
    pub jump_buffer_remaining: f32,
    pub coyote_time_remaining: f32,
    pub wall_jump_lock_remaining: f32,
    pub drop_through_remaining: f32,
    pub remaining_air_jumps: u32,
    pub remaining_dashes: u32,
    pub support_velocity: Vec2,
    pub support_position: Option<Vec2>,
    pub dash_time_remaining: f32,
    pub dash_cooldown_remaining: f32,
    pub dash_direction: Vec2,
    pub facing_sign: f32,
    pub last_support_entity: Option<Entity>,
    pub last_support_position: Option<Vec2>,
    pub pre_ground: Option<PlatformerContact>,
    pub pre_left_wall: Option<PlatformerContact>,
    pub pre_right_wall: Option<PlatformerContact>,
    pub ground: Option<PlatformerContact>,
    pub left_wall: Option<PlatformerContact>,
    pub right_wall: Option<PlatformerContact>,
    pub pending_jump: Option<PendingJumpMessage>,
    pub pending_wall_jump: Option<PendingWallJumpMessage>,
    pub pending_dash: Option<PendingDashMessage>,
    pub pending_landed_impact_speed: Option<f32>,
    pub pending_landed_support: Option<Entity>,
    pub pending_air_jump_consumed: Option<u32>,
}

impl Default for PlatformerControllerRuntimeState {
    fn default() -> Self {
        Self {
            initialized: false,
            previous_velocity: Vec2::ZERO,
            jump_buffer_remaining: 0.0,
            coyote_time_remaining: 0.0,
            wall_jump_lock_remaining: 0.0,
            drop_through_remaining: 0.0,
            remaining_air_jumps: 0,
            remaining_dashes: 0,
            support_velocity: Vec2::ZERO,
            support_position: None,
            dash_time_remaining: 0.0,
            dash_cooldown_remaining: 0.0,
            dash_direction: Vec2::X,
            facing_sign: 1.0,
            last_support_entity: None,
            last_support_position: None,
            pre_ground: None,
            pre_left_wall: None,
            pre_right_wall: None,
            ground: None,
            left_wall: None,
            right_wall: None,
            pending_jump: None,
            pending_wall_jump: None,
            pending_dash: None,
            pending_landed_impact_speed: None,
            pending_landed_support: None,
            pending_air_jump_consumed: None,
        }
    }
}

pub(crate) fn runtime_from_config(
    config: &PlatformerControllerConfig,
) -> PlatformerControllerRuntimeState {
    PlatformerControllerRuntimeState {
        remaining_air_jumps: config.jump.max_air_jumps,
        remaining_dashes: config.dash.max_charges,
        ..default()
    }
}
