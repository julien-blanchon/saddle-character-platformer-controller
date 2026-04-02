mod bundles;
mod components;
mod config;
mod debug;
mod helpers;
mod messages;
mod systems;

pub use bundles::PlatformerControllerBundle;
pub use components::{
    PlatformVelocityInheritance, PlatformerContact, PlatformerController,
    PlatformerControllerState, PlatformerMotionPhase, PlatformerMovementIntent,
    PlatformerOneWayPlatform, PlatformerWallContact, PlatformerWallSide,
};
pub use config::{
    MoveAndSlideTuning, MovementConfig, PlatformInteractionConfig, PlatformerControllerConfig,
    PlatformerJumpConfig, PlatformerSensingConfig, PlatformerWallConfig,
};
pub use debug::{PlatformerControllerDebugPlugin, PlatformerControllerDebugSettings};
pub use messages::{AirJumpConsumed, JumpStarted, Landed, PlatformerJumpKind, WallJumpStarted};

use bevy::{
    app::PostStartup,
    ecs::{intern::Interned, schedule::ScheduleLabel},
    prelude::*,
};

#[derive(SystemSet, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum PlatformerControllerSystems {
    ReadIntent,
    SenseContacts,
    ApplyMovement,
    ApplyJump,
    WallInteractions,
    MoveControllers,
    SyncState,
}

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
struct NeverDeactivateSchedule;

pub struct PlatformerControllerPlugin {
    pub activate_schedule: Interned<dyn ScheduleLabel>,
    pub deactivate_schedule: Interned<dyn ScheduleLabel>,
    pub update_schedule: Interned<dyn ScheduleLabel>,
}

impl PlatformerControllerPlugin {
    pub fn new(
        activate_schedule: impl ScheduleLabel,
        deactivate_schedule: impl ScheduleLabel,
        update_schedule: impl ScheduleLabel,
    ) -> Self {
        Self {
            activate_schedule: activate_schedule.intern(),
            deactivate_schedule: deactivate_schedule.intern(),
            update_schedule: update_schedule.intern(),
        }
    }

    pub fn always_on(update_schedule: impl ScheduleLabel) -> Self {
        Self::new(PostStartup, NeverDeactivateSchedule, update_schedule)
    }
}

impl Default for PlatformerControllerPlugin {
    fn default() -> Self {
        Self::always_on(Update)
    }
}

impl Plugin for PlatformerControllerPlugin {
    fn build(&self, app: &mut App) {
        if self.deactivate_schedule == NeverDeactivateSchedule.intern() {
            app.init_schedule(NeverDeactivateSchedule);
        }

        app.init_resource::<systems::activation::PlatformerControllerRuntime>()
            .add_message::<JumpStarted>()
            .add_message::<Landed>()
            .add_message::<WallJumpStarted>()
            .add_message::<AirJumpConsumed>()
            .register_type::<MovementConfig>()
            .register_type::<MoveAndSlideTuning>()
            .register_type::<PlatformInteractionConfig>()
            .register_type::<PlatformVelocityInheritance>()
            .register_type::<PlatformerContact>()
            .register_type::<PlatformerController>()
            .register_type::<PlatformerControllerConfig>()
            .register_type::<PlatformerControllerState>()
            .register_type::<PlatformerJumpConfig>()
            .register_type::<PlatformerJumpKind>()
            .register_type::<PlatformerMotionPhase>()
            .register_type::<PlatformerMovementIntent>()
            .register_type::<PlatformerOneWayPlatform>()
            .register_type::<PlatformerSensingConfig>()
            .register_type::<PlatformerWallConfig>()
            .register_type::<PlatformerWallContact>()
            .register_type::<PlatformerWallSide>()
            .add_systems(
                self.activate_schedule,
                systems::activation::activate_runtime,
            )
            .add_systems(
                self.deactivate_schedule,
                systems::activation::deactivate_runtime,
            )
            .configure_sets(
                self.update_schedule,
                (
                    PlatformerControllerSystems::ReadIntent,
                    PlatformerControllerSystems::SenseContacts,
                    PlatformerControllerSystems::ApplyMovement,
                    PlatformerControllerSystems::ApplyJump,
                    PlatformerControllerSystems::WallInteractions,
                    PlatformerControllerSystems::MoveControllers,
                    PlatformerControllerSystems::SyncState,
                )
                    .chain(),
            )
            .add_systems(
                self.update_schedule,
                (
                    systems::intent::prepare_intents
                        .in_set(PlatformerControllerSystems::ReadIntent),
                    systems::sensing::sense_pre_movement_contacts
                        .in_set(PlatformerControllerSystems::SenseContacts),
                    systems::movement::apply_horizontal_movement
                        .in_set(PlatformerControllerSystems::ApplyMovement),
                    systems::movement::apply_jump_logic
                        .in_set(PlatformerControllerSystems::ApplyJump),
                    systems::movement::apply_wall_interactions
                        .in_set(PlatformerControllerSystems::WallInteractions),
                    systems::movement::move_controllers
                        .in_set(PlatformerControllerSystems::MoveControllers),
                    systems::state_sync::sync_controller_state
                        .in_set(PlatformerControllerSystems::SyncState),
                    systems::state_sync::emit_messages
                        .in_set(PlatformerControllerSystems::SyncState),
                    systems::intent::clear_transient_intents
                        .in_set(PlatformerControllerSystems::SyncState),
                )
                    .run_if(systems::activation::runtime_is_active),
            );
    }
}

#[cfg(test)]
#[path = "plugin_tests.rs"]
mod plugin_tests;
