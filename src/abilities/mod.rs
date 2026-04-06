mod dash;
mod grapple;
mod ground_pound;

use std::sync::Arc;

use bevy::prelude::*;

pub use dash::{
    DashStarted, PlatformerDashBundle, PlatformerDashConfig, PlatformerDashIntent,
    PlatformerDashPlugin, PlatformerDashState, PlatformerDashSystems,
};
pub use grapple::{
    GrappleAttached, GrappleDetached, PlatformerGrappleBundle, PlatformerGrappleConfig,
    PlatformerGrappleIntent, PlatformerGrapplePhase, PlatformerGrapplePlugin,
    PlatformerGrapplePoint, PlatformerGrappleState, PlatformerGrappleSystems,
};
pub use ground_pound::{
    GroundPoundImpact, GroundPoundStarted, PlatformerGroundPoundBundle,
    PlatformerGroundPoundConfig, PlatformerGroundPoundIntent, PlatformerGroundPoundPhase,
    PlatformerGroundPoundPlugin, PlatformerGroundPoundState, PlatformerGroundPoundSystems,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PlatformerAbilityActivity {
    pub dash: bool,
    pub ground_pound: bool,
    pub grapple: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PlatformerAbilityKind {
    Dash,
    GroundPound,
    Grapple,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PlatformerAbilityConflictAction {
    #[default]
    Keep,
    Cancel,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlatformerAbilityActivationResolution {
    pub allow_requested: bool,
    pub dash: PlatformerAbilityConflictAction,
    pub ground_pound: PlatformerAbilityConflictAction,
    pub grapple: PlatformerAbilityConflictAction,
}

impl PlatformerAbilityActivationResolution {
    pub const fn allow() -> Self {
        Self {
            allow_requested: true,
            dash: PlatformerAbilityConflictAction::Keep,
            ground_pound: PlatformerAbilityConflictAction::Keep,
            grapple: PlatformerAbilityConflictAction::Keep,
        }
    }

    pub const fn block() -> Self {
        Self {
            allow_requested: false,
            dash: PlatformerAbilityConflictAction::Keep,
            ground_pound: PlatformerAbilityConflictAction::Keep,
            grapple: PlatformerAbilityConflictAction::Keep,
        }
    }
}

pub trait PlatformerAbilityCompositionPolicy: Send + Sync + 'static {
    fn resolve_activation(
        &self,
        requested: PlatformerAbilityKind,
        active: PlatformerAbilityActivity,
    ) -> PlatformerAbilityActivationResolution;

    fn detach_grapple_on_jump(&self, _active: PlatformerAbilityActivity) -> bool {
        true
    }
}

#[derive(Resource, Clone)]
pub struct PlatformerAbilityComposition(pub Arc<dyn PlatformerAbilityCompositionPolicy>);

impl Default for PlatformerAbilityComposition {
    fn default() -> Self {
        Self(Arc::new(DefaultPlatformerAbilityComposition))
    }
}

#[derive(Debug, Default)]
struct DefaultPlatformerAbilityComposition;

impl PlatformerAbilityCompositionPolicy for DefaultPlatformerAbilityComposition {
    fn resolve_activation(
        &self,
        requested: PlatformerAbilityKind,
        active: PlatformerAbilityActivity,
    ) -> PlatformerAbilityActivationResolution {
        match requested {
            PlatformerAbilityKind::Dash => {
                if active.ground_pound || active.grapple {
                    PlatformerAbilityActivationResolution::block()
                } else {
                    PlatformerAbilityActivationResolution::allow()
                }
            }
            PlatformerAbilityKind::GroundPound => {
                if active.dash || active.grapple {
                    PlatformerAbilityActivationResolution::block()
                } else {
                    PlatformerAbilityActivationResolution::allow()
                }
            }
            PlatformerAbilityKind::Grapple => {
                let mut resolution = PlatformerAbilityActivationResolution::allow();
                if active.dash || active.grapple {
                    return PlatformerAbilityActivationResolution::block();
                }
                if active.ground_pound {
                    resolution.ground_pound = PlatformerAbilityConflictAction::Cancel;
                }
                resolution
            }
        }
    }
}

pub(crate) fn ability_activity(
    dash_active: bool,
    ground_pound_active: bool,
    grapple_active: bool,
) -> PlatformerAbilityActivity {
    PlatformerAbilityActivity {
        dash: dash_active,
        ground_pound: ground_pound_active,
        grapple: grapple_active,
    }
}
