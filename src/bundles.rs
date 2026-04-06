use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    PlatformerController, PlatformerControllerConfig, PlatformerControllerDirectives,
    PlatformerControllerState, PlatformerMovementIntent,
    components::{PlatformerControllerRuntimeState, runtime_from_config},
};

#[derive(Bundle)]
pub struct PlatformerControllerBundle {
    controller: PlatformerController,
    config: PlatformerControllerConfig,
    intent: PlatformerMovementIntent,
    directives: PlatformerControllerDirectives,
    state: PlatformerControllerState,
    runtime: PlatformerControllerRuntimeState,
    body: RigidBody,
    transform: Transform,
    global_transform: GlobalTransform,
    position: Position,
    rotation: Rotation,
    velocity: LinearVelocity,
    custom_position_integration: CustomPositionIntegration,
    locked_axes: LockedAxes,
    collider: Collider,
    /// Smooth visual interpolation between fixed physics steps.
    ///
    /// This eliminates the "ghost rectangle" artifact that appears when the
    /// sprite renders at the previous-frame `Transform` while `Position` has
    /// already advanced in the fixed timestep.
    interpolation: TransformInterpolation,
}

impl PlatformerControllerBundle {
    pub fn new(collider: Collider) -> Self {
        Self::with_config(collider, PlatformerControllerConfig::default())
    }

    pub fn with_config(collider: Collider, config: PlatformerControllerConfig) -> Self {
        let runtime = runtime_from_config(&config);
        Self {
            controller: PlatformerController,
            config,
            intent: PlatformerMovementIntent::default(),
            directives: PlatformerControllerDirectives::default(),
            state: PlatformerControllerState::default(),
            runtime,
            body: RigidBody::Kinematic,
            transform: Transform::default(),
            global_transform: GlobalTransform::default(),
            position: Position(Vec2::ZERO),
            rotation: Rotation::default(),
            velocity: LinearVelocity::ZERO,
            custom_position_integration: CustomPositionIntegration,
            locked_axes: LockedAxes::ROTATION_LOCKED,
            collider,
            interpolation: TransformInterpolation,
        }
    }

    pub fn with_transform(mut self, transform: Transform) -> Self {
        self.position = Position(transform.translation.xy());
        self.rotation = Rotation::radians(transform.rotation.to_euler(EulerRot::XYZ).2);
        self.global_transform = GlobalTransform::from(transform);
        self.transform = transform;
        self
    }
}
