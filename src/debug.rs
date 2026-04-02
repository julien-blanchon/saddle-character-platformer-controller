use bevy::prelude::*;

use crate::{PlatformerController, PlatformerControllerState};

#[derive(Resource, Reflect, Clone, Debug)]
#[reflect(Resource, Debug, Default)]
pub struct PlatformerControllerDebugSettings {
    pub enabled: bool,
    pub draw_ground_probe: bool,
    pub draw_wall_probes: bool,
    pub draw_velocity: bool,
}

impl Default for PlatformerControllerDebugSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            draw_ground_probe: true,
            draw_wall_probes: true,
            draw_velocity: true,
        }
    }
}

pub struct PlatformerControllerDebugPlugin;

impl Plugin for PlatformerControllerDebugPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlatformerControllerDebugSettings>()
            .register_type::<PlatformerControllerDebugSettings>()
            .add_systems(PostUpdate, debug_draw);
    }
}

fn debug_draw(
    settings: Res<PlatformerControllerDebugSettings>,
    query: Query<(&Transform, &PlatformerControllerState), With<PlatformerController>>,
    mut gizmos: Gizmos,
) {
    if !settings.enabled {
        return;
    }

    for (transform, state) in &query {
        let origin = transform.translation.xy();

        if settings.draw_velocity {
            gizmos.line_2d(
                origin,
                origin + state.velocity * 0.1,
                Color::srgb(0.92, 0.42, 0.18),
            );
        }

        if settings.draw_ground_probe {
            let color = if state.is_grounded {
                Color::srgb(0.18, 0.88, 0.48)
            } else {
                Color::srgb(0.84, 0.24, 0.24)
            };
            gizmos.line_2d(origin, origin + Vec2::NEG_Y * 18.0, color);
        }

        if settings.draw_wall_probes {
            let wall_color = if state.wall.is_some() {
                Color::srgb(0.38, 0.68, 0.98)
            } else {
                Color::srgb(0.34, 0.34, 0.42)
            };
            gizmos.line_2d(origin, origin + Vec2::X * 14.0, wall_color);
            gizmos.line_2d(origin, origin + Vec2::NEG_X * 14.0, wall_color);
        }
    }
}
