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
}

impl Default for PlatformerMovementIntent {
    fn default() -> Self {
        Self {
            move_axis: 0.0,
            jump_pressed: false,
            jump_held: false,
            drop_pressed: false,
        }
    }
}

/// Public hook that lets optional ability plugins or downstream code suppress
/// parts of the core locomotion pipeline for a single frame.
#[derive(Component, Reflect, Clone, Debug, Default, PartialEq, Eq)]
#[reflect(Component, Default, Debug, PartialEq)]
pub struct PlatformerControllerDirectives {
    pub suppress_horizontal_movement: bool,
    pub suppress_jump_logic: bool,
    pub suppress_wall_interactions: bool,
}

#[derive(Component, Reflect, Clone, Debug, Default)]
#[reflect(Component, Default, Debug)]
pub struct PlatformerOneWayPlatform;

/// Attach to a ground/platform entity to modify movement physics on contact.
///
/// Example: ice surface (`friction_multiplier: 0.15`), conveyor belt
/// (`surface_velocity: Vec2::new(120.0, 0.0)`), mud (`speed_multiplier: 0.5`).
#[derive(Component, Reflect, Clone, Debug, PartialEq)]
#[reflect(Component, Default, Debug)]
pub struct PlatformerSurfaceModifier {
    /// Multiplier on acceleration and deceleration (0.0 = no friction / ice, 1.0 = normal).
    pub friction_multiplier: f32,
    /// Constant velocity added to the character while on this surface (conveyor belt).
    pub surface_velocity: Vec2,
    /// Multiplier on maximum speed while on this surface.
    pub speed_multiplier: f32,
}

impl Default for PlatformerSurfaceModifier {
    fn default() -> Self {
        Self {
            friction_multiplier: 1.0,
            surface_velocity: Vec2::ZERO,
            speed_multiplier: 1.0,
        }
    }
}

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
    Rising,
    Apex,
    Falling,
    WallSliding,
    WallClinging,
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
    pub coyote_time_remaining: f32,
    pub jump_buffer_remaining: f32,
    pub wall_jump_lock_remaining: f32,
    pub wall_cling_remaining: f32,
    /// The active surface modifier from the ground entity (if any).
    pub surface_modifier: Option<PlatformerSurfaceModifier>,
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
            coyote_time_remaining: 0.0,
            jump_buffer_remaining: 0.0,
            wall_jump_lock_remaining: 0.0,
            wall_cling_remaining: 0.0,
            surface_modifier: None,
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

#[derive(Component, Clone, Debug)]
pub(crate) struct PlatformerControllerRuntimeState {
    pub initialized: bool,
    pub previous_velocity: Vec2,
    pub jump_buffer_remaining: f32,
    pub coyote_time_remaining: f32,
    pub wall_jump_lock_remaining: f32,
    pub drop_through_remaining: f32,
    pub remaining_air_jumps: u32,
    pub support_velocity: Vec2,
    pub support_position: Option<Vec2>,
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
    pub pending_landed_impact_speed: Option<f32>,
    pub pending_landed_support: Option<Entity>,
    pub pending_air_jump_consumed: Option<u32>,
    pub wall_cling_remaining: f32,
    pub was_wall_clinging: bool,
    pub pending_wall_cling_started: Option<PlatformerWallSide>,
    pub surface_modifier: Option<PlatformerSurfaceModifier>,
    pub directives: PlatformerControllerDirectives,
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
            support_velocity: Vec2::ZERO,
            support_position: None,
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
            pending_landed_impact_speed: None,
            pending_landed_support: None,
            pending_air_jump_consumed: None,
            wall_cling_remaining: 0.0,
            was_wall_clinging: false,
            pending_wall_cling_started: None,
            surface_modifier: None,
            directives: PlatformerControllerDirectives::default(),
        }
    }
}

pub(crate) fn runtime_from_config(
    config: &PlatformerControllerConfig,
) -> PlatformerControllerRuntimeState {
    PlatformerControllerRuntimeState {
        remaining_air_jumps: config.jump.max_air_jumps,
        ..default()
    }
}
