use bevy::prelude::*;

use crate::PlatformVelocityInheritance;

#[derive(Component, Clone, Debug, Reflect, Default)]
#[reflect(Component, Debug, Default)]
pub struct PlatformerControllerConfig {
    pub movement: MovementConfig,
    pub jump: PlatformerJumpConfig,
    pub dash: PlatformerDashConfig,
    pub corner_correction: PlatformerCornerCorrectionConfig,
    pub walls: PlatformerWallConfig,
    pub sensing: PlatformerSensingConfig,
    pub platforms: PlatformInteractionConfig,
    pub move_and_slide: MoveAndSlideTuning,
    pub ground_pound: PlatformerGroundPoundConfig,
    pub grapple: PlatformerGrappleConfig,
}

#[derive(Clone, Debug, Reflect)]
#[reflect(Debug, Default)]
pub struct MovementConfig {
    pub max_speed: f32,
    pub ground_acceleration: f32,
    pub ground_deceleration: f32,
    pub air_acceleration: f32,
    pub air_deceleration: f32,
    pub apex_air_control_multiplier: f32,
}

impl Default for MovementConfig {
    fn default() -> Self {
        Self {
            max_speed: 220.0,
            ground_acceleration: 1800.0,
            ground_deceleration: 2400.0,
            air_acceleration: 1100.0,
            air_deceleration: 700.0,
            apex_air_control_multiplier: 1.15,
        }
    }
}

#[derive(Clone, Debug, Reflect)]
#[reflect(Debug, Default)]
pub struct PlatformerJumpConfig {
    pub height: f32,
    pub time_to_apex: f32,
    pub rise_gravity_multiplier: f32,
    pub fall_gravity_multiplier: f32,
    pub low_jump_gravity_multiplier: f32,
    pub apex_gravity_multiplier: f32,
    pub apex_velocity_threshold: f32,
    pub coyote_time: f32,
    pub jump_buffer_time: f32,
    pub max_air_jumps: u32,
    /// Maximum downward speed (terminal velocity). Zero disables the cap.
    pub max_fall_speed: f32,
}

impl Default for PlatformerJumpConfig {
    fn default() -> Self {
        Self {
            height: 78.0,
            time_to_apex: 0.42,
            rise_gravity_multiplier: 1.0,
            fall_gravity_multiplier: 1.7,
            low_jump_gravity_multiplier: 2.35,
            apex_gravity_multiplier: 0.8,
            apex_velocity_threshold: 26.0,
            coyote_time: 0.1,
            jump_buffer_time: 0.12,
            max_air_jumps: 1,
            max_fall_speed: 0.0,
        }
    }
}

impl PlatformerJumpConfig {
    pub fn base_gravity(&self) -> f32 {
        let apex_time = self.time_to_apex.max(0.001);
        (2.0 * self.height.max(0.0)) / (apex_time * apex_time)
    }

    pub fn jump_speed(&self) -> f32 {
        self.base_gravity() * self.time_to_apex.max(0.001)
    }
}

#[derive(Clone, Debug, Reflect)]
#[reflect(Debug, Default)]
pub struct PlatformerDashConfig {
    pub distance: f32,
    pub duration: f32,
    pub cooldown: f32,
    pub max_charges: u32,
    pub refill_on_ground: bool,
    pub allow_ground_dash: bool,
    pub preserve_vertical_velocity: bool,
    pub direction_input_threshold: f32,
    pub exit_speed_scale: f32,
}

impl Default for PlatformerDashConfig {
    fn default() -> Self {
        Self {
            distance: 84.0,
            duration: 0.16,
            cooldown: 0.12,
            max_charges: 1,
            refill_on_ground: true,
            allow_ground_dash: true,
            preserve_vertical_velocity: false,
            direction_input_threshold: 0.2,
            exit_speed_scale: 0.35,
        }
    }
}

impl PlatformerDashConfig {
    pub fn dash_speed(&self) -> f32 {
        self.distance.max(0.0) / self.duration.max(0.001)
    }
}

#[derive(Clone, Debug, Reflect)]
#[reflect(Debug, Default)]
pub struct PlatformerCornerCorrectionConfig {
    pub max_distance: f32,
    pub step_size: f32,
    pub min_upward_speed: f32,
    pub min_height_gain: f32,
    /// Maximum horizontal nudge distance for ledge assist (landing on ledge edges).
    /// Zero disables ledge assist.
    pub ledge_assist_distance: f32,
}

impl Default for PlatformerCornerCorrectionConfig {
    fn default() -> Self {
        Self {
            max_distance: 10.0,
            step_size: 2.0,
            min_upward_speed: 18.0,
            min_height_gain: 1.0,
            ledge_assist_distance: 4.0,
        }
    }
}

#[derive(Clone, Debug, Reflect)]
#[reflect(Debug, Default)]
pub struct PlatformerWallConfig {
    pub probe_distance: f32,
    pub min_normal_x: f32,
    pub max_vertical_normal_y: f32,
    pub max_contact_height_ratio: f32,
    pub wall_slide_terminal_speed: f32,
    pub wall_slide_gravity_multiplier: f32,
    pub wall_jump_horizontal_speed: f32,
    pub wall_jump_vertical_speed: f32,
    pub wall_jump_steering_lock_time: f32,
    pub wall_jump_steering_factor: f32,
    pub wall_slide_requires_input: bool,
    /// Maximum time the character can cling to a wall (zero disables cling).
    pub wall_cling_max_duration: f32,
    /// Gravity multiplier while clinging (0.0 = full stop).
    pub wall_cling_gravity_multiplier: f32,
}

impl Default for PlatformerWallConfig {
    fn default() -> Self {
        Self {
            probe_distance: 8.0,
            min_normal_x: 0.7,
            max_vertical_normal_y: 0.3,
            max_contact_height_ratio: 0.82,
            wall_slide_terminal_speed: 110.0,
            wall_slide_gravity_multiplier: 0.55,
            wall_jump_horizontal_speed: 235.0,
            wall_jump_vertical_speed: 285.0,
            wall_jump_steering_lock_time: 0.14,
            wall_jump_steering_factor: 0.2,
            wall_slide_requires_input: true,
            wall_cling_max_duration: 0.0,
            wall_cling_gravity_multiplier: 0.0,
        }
    }
}

#[derive(Clone, Debug, Reflect)]
#[reflect(Debug, Default)]
pub struct PlatformerSensingConfig {
    pub max_walkable_angle: f32,
    pub ground_probe_distance: f32,
    pub ground_snap_distance: f32,
    pub one_way_normal_alignment: f32,
}

impl Default for PlatformerSensingConfig {
    fn default() -> Self {
        Self {
            max_walkable_angle: 46.0_f32.to_radians(),
            ground_probe_distance: 10.0,
            ground_snap_distance: 8.0,
            one_way_normal_alignment: 0.7,
        }
    }
}

#[derive(Clone, Debug, Reflect)]
#[reflect(Debug, Default)]
pub struct PlatformInteractionConfig {
    pub velocity_inheritance: PlatformVelocityInheritance,
    pub drop_through_duration: f32,
}

impl Default for PlatformInteractionConfig {
    fn default() -> Self {
        Self {
            velocity_inheritance: PlatformVelocityInheritance::Horizontal,
            drop_through_duration: 0.18,
        }
    }
}

#[derive(Clone, Debug, Reflect)]
#[reflect(Debug, Default)]
pub struct MoveAndSlideTuning {
    pub skin_width: f32,
    pub move_and_slide_iterations: usize,
    pub depenetration_iterations: usize,
    pub max_depenetration_error: f32,
    pub max_planes: usize,
}

impl Default for MoveAndSlideTuning {
    fn default() -> Self {
        Self {
            skin_width: 0.02,
            move_and_slide_iterations: 4,
            depenetration_iterations: 4,
            max_depenetration_error: 0.001,
            max_planes: 16,
        }
    }
}

#[derive(Clone, Debug, Reflect)]
#[reflect(Debug, Default)]
pub struct PlatformerGroundPoundConfig {
    /// Brief hover before slamming (seconds).
    pub hover_duration: f32,
    /// Downward speed during the slam.
    pub fall_speed: f32,
    /// Whether to cancel horizontal velocity on activation.
    pub cancel_horizontal_speed: bool,
    /// Brief freeze on impact before movement resumes.
    pub impact_stun_duration: f32,
}

impl Default for PlatformerGroundPoundConfig {
    fn default() -> Self {
        Self {
            hover_duration: 0.08,
            fall_speed: 600.0,
            cancel_horizontal_speed: true,
            impact_stun_duration: 0.1,
        }
    }
}

#[derive(Clone, Debug, Reflect)]
#[reflect(Debug, Default)]
pub struct PlatformerGrappleConfig {
    /// Maximum distance to search for a grapple point.
    pub max_range: f32,
    /// Speed at which the character is pulled toward the point (0.0 = pure swing).
    pub pull_speed: f32,
    /// Velocity scale applied on detach (momentum boost).
    pub detach_speed_boost: f32,
    /// Angle tolerance (radians) for aim-assist when finding grapple points.
    pub aim_assist_angle: f32,
    /// Minimum rope length (how close you can reel in).
    pub min_rope_length: f32,
    /// Speed at which the rope retracts per second.
    pub retract_speed: f32,
    /// Speed at which the rope extends per second.
    pub extend_speed: f32,
    /// Gravity multiplier while swinging on the rope (1.0 = normal gravity).
    pub swing_gravity_multiplier: f32,
    /// How much horizontal input affects the swing (0.0 = no influence).
    pub swing_input_force: f32,
}

impl Default for PlatformerGrappleConfig {
    fn default() -> Self {
        Self {
            max_range: 300.0,
            pull_speed: 400.0,
            detach_speed_boost: 1.3,
            aim_assist_angle: 0.35,
            min_rope_length: 20.0,
            retract_speed: 200.0,
            extend_speed: 100.0,
            swing_gravity_multiplier: 1.0,
            swing_input_force: 300.0,
        }
    }
}
