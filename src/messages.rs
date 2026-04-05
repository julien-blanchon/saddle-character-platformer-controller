use bevy::prelude::*;

use crate::PlatformerWallSide;

#[derive(Clone, Copy, Debug, Reflect, PartialEq, Eq)]
#[reflect(Debug, PartialEq)]
pub enum PlatformerJumpKind {
    Ground,
    Coyote,
    Air,
    Wall,
}

#[derive(Clone, Debug, Message)]
pub struct JumpStarted {
    pub entity: Entity,
    pub kind: PlatformerJumpKind,
    pub used_buffer: bool,
    pub velocity: Vec2,
}

#[derive(Clone, Debug, Message)]
pub struct WallJumpStarted {
    pub entity: Entity,
    pub side: PlatformerWallSide,
    pub velocity: Vec2,
}

#[derive(Clone, Debug, Message)]
pub struct DashStarted {
    pub entity: Entity,
    pub direction: Vec2,
    pub velocity: Vec2,
    pub remaining_charges: u32,
}

#[derive(Clone, Debug, Message)]
pub struct Landed {
    pub entity: Entity,
    pub impact_speed: f32,
    pub support_entity: Option<Entity>,
}

#[derive(Clone, Debug, Message)]
pub struct AirJumpConsumed {
    pub entity: Entity,
    pub remaining_air_jumps: u32,
}

#[derive(Clone, Debug, Message)]
pub struct GroundPoundStarted {
    pub entity: Entity,
}

#[derive(Clone, Debug, Message)]
pub struct GroundPoundImpact {
    pub entity: Entity,
    pub impact_speed: f32,
}

#[derive(Clone, Debug, Message)]
pub struct WallClingStarted {
    pub entity: Entity,
    pub side: PlatformerWallSide,
}

#[derive(Clone, Debug, Message)]
pub struct GrappleAttached {
    pub entity: Entity,
    pub target: Vec2,
}

#[derive(Clone, Debug, Message)]
pub struct GrappleDetached {
    pub entity: Entity,
    pub velocity: Vec2,
}
