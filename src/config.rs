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
}

impl Default for PlatformerCornerCorrectionConfig {
    fn default() -> Self {
        Self {
            max_distance: 10.0,
            step_size: 2.0,
            min_upward_speed: 18.0,
            min_height_gain: 1.0,
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
