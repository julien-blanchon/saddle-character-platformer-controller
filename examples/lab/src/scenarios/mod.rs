mod support;

use bevy::prelude::{Vec2, World};
use saddle_bevy_e2e::{action::Action, actions::assertions, scenario::Scenario};
use saddle_character_platformer_controller::{
    PlatformerJumpKind, PlatformerMotionPhase, PlatformerWallSide,
};

use crate::LabMessageLog;

fn hard_assert(
    label: &'static str,
    check: impl Fn(&World) -> bool + Send + Sync + 'static,
) -> Action {
    Action::Custom(Box::new(move |world| {
        assert!(check(world), "{label}");
    }))
}

pub fn list_scenarios() -> Vec<&'static str> {
    vec![
        "platformer_controller_smoke",
        "platformer_controller_coyote_jump",
        "platformer_controller_jump_buffer",
        "platformer_controller_wall_jump",
        "platformer_controller_moving_platform",
        "platformer_controller_one_way",
    ]
}

pub fn scenario_by_name(name: &str) -> Option<Scenario> {
    match name {
        "platformer_controller_smoke" => Some(platformer_controller_smoke()),
        "platformer_controller_coyote_jump" => Some(platformer_controller_coyote_jump()),
        "platformer_controller_jump_buffer" => Some(platformer_controller_jump_buffer()),
        "platformer_controller_wall_jump" => Some(platformer_controller_wall_jump()),
        "platformer_controller_moving_platform" => Some(platformer_controller_moving_platform()),
        "platformer_controller_one_way" => Some(platformer_controller_one_way()),
        _ => None,
    }
}

fn platformer_controller_smoke() -> Scenario {
    Scenario::builder("platformer_controller_smoke")
        .description("Boot the basic scene, let the player settle onto the floor, and capture a readable baseline screenshot.")
        .then(Action::WaitFrames(90))
        .then(hard_assert("player grounded after settle", |world| {
            world.resource::<support::DiagnosticsResource>().grounded
        }))
        .then(assertions::resource_satisfies::<support::DiagnosticsResource>(
            "player grounded after settle",
            |diagnostics| diagnostics.grounded,
        ))
        .then(hard_assert("player entity exists", |world| {
            support::player_entity(world).is_some()
        }))
        .then(assertions::custom("player entity exists", |world| {
            support::player_entity(world).is_some()
        }))
        .then(hard_assert("overlay text populated", |world| {
            support::overlay_text(world).is_some_and(|text| text.contains("phase:"))
        }))
        .then(assertions::custom("overlay text populated", |world| {
            support::overlay_text(world).is_some_and(|text| text.contains("phase:"))
        }))
        .then(assertions::log_summary("platformer_controller_smoke summary"))
        .then(Action::Screenshot("platformer_controller_smoke".into()))
        .build()
}

fn platformer_controller_coyote_jump() -> Scenario {
    Scenario::builder("platformer_controller_coyote_jump")
        .description("Walk off a ledge in the basic scene, jump during the coyote window, and verify the jump is classified correctly.")
        .then(Action::Custom(Box::new(|world| {
            support::teleport_player(world, Vec2::new(-36.0, -57.0), Vec2::ZERO);
            support::set_scripted_control(world, 1.0, false, false, false);
        })))
        .then(Action::WaitFrames(8))
        .then(Action::Screenshot("coyote_setup".into()))
        .then(Action::WaitUntil {
            label: "walk off ledge".into(),
            condition: Box::new(|world| {
                support::player_state(world)
                    .is_some_and(|state| !state.is_grounded && state.can_use_coyote_jump)
            }),
            max_frames: 90,
        })
        .then(Action::Custom(Box::new(|world| {
            support::set_scripted_control(world, 1.0, true, true, false);
        })))
        .then(Action::WaitUntil {
            label: "coyote jump launched".into(),
            condition: Box::new(|world| {
                let diagnostics = world.resource::<support::DiagnosticsResource>();
                let log = world.resource::<LabMessageLog>();
                log.last_jump_kind == Some(PlatformerJumpKind::Coyote)
                    && diagnostics.player_velocity.y > 100.0
            }),
            max_frames: 45,
        })
        .then(hard_assert("coyote jump launched", |world| {
            let diagnostics = world.resource::<support::DiagnosticsResource>();
            let log = world.resource::<LabMessageLog>();
            log.last_jump_kind == Some(PlatformerJumpKind::Coyote)
                && diagnostics.player_velocity.y > 100.0
        }))
        .then(assertions::custom("coyote jump classified", |world| {
            let log = world.resource::<LabMessageLog>();
            log.last_jump_kind == Some(PlatformerJumpKind::Coyote) && !log.last_jump_used_buffer
        }))
        .then(assertions::custom("player is rising after coyote jump", |world| {
            support::player_state(world).is_some_and(|state| {
                state.phase == PlatformerMotionPhase::Rising && state.velocity.y > 100.0
            })
        }))
        .then(assertions::log_summary("platformer_controller_coyote_jump summary"))
        .then(Action::Screenshot("coyote_jump".into()))
        .build()
}

fn platformer_controller_jump_buffer() -> Scenario {
    Scenario::builder("platformer_controller_jump_buffer")
        .description("Press jump just before landing and verify the buffered jump fires on the first grounded frame.")
        .then(Action::Custom(Box::new(|world| {
            let mut config = world.resource::<crate::support::DemoState>().scene.controller_config();
            config.jump.max_air_jumps = 0;
            support::teleport_player_with_config(
                world,
                Vec2::new(-240.0, -80.0),
                Vec2::new(0.0, -220.0),
                config,
            );
            support::set_scripted_control(world, 0.0, false, false, false);
        })))
        .then(Action::WaitUntil {
            label: "pre-landing fall".into(),
            condition: Box::new(|world| {
                let diagnostics = world.resource::<support::DiagnosticsResource>();
                !diagnostics.grounded
                    && diagnostics.player_velocity.y < -50.0
                    && diagnostics.player_position.y < -88.0
            }),
            max_frames: 20,
        })
        .then(Action::Screenshot("jump_buffer_fall".into()))
        .then(Action::Custom(Box::new(|world| {
            support::set_scripted_control(world, 0.0, true, true, false);
        })))
        .then(Action::WaitUntil {
            label: "buffered jump fired".into(),
            condition: Box::new(|world| {
                let diagnostics = world.resource::<support::DiagnosticsResource>();
                let log = world.resource::<LabMessageLog>();
                log.last_jump_kind == Some(PlatformerJumpKind::Ground)
                    && log.last_jump_used_buffer
                    && diagnostics.player_velocity.y > 100.0
            }),
            max_frames: 90,
        })
        .then(hard_assert("buffered jump fired", |world| {
            let diagnostics = world.resource::<support::DiagnosticsResource>();
            let log = world.resource::<LabMessageLog>();
            log.last_jump_kind == Some(PlatformerJumpKind::Ground)
                && log.last_jump_used_buffer
                && diagnostics.player_velocity.y > 100.0
        }))
        .then(assertions::custom("jump buffer consumed", |world| {
            let log = world.resource::<LabMessageLog>();
            log.last_jump_kind == Some(PlatformerJumpKind::Ground) && log.last_jump_used_buffer
        }))
        .then(assertions::custom("player launched upward", |world| {
            support::player_state(world)
                .is_some_and(|state| state.phase == PlatformerMotionPhase::Rising)
        }))
        .then(assertions::log_summary("platformer_controller_jump_buffer summary"))
        .then(Action::Screenshot("jump_buffer_launch".into()))
        .build()
}

fn platformer_controller_wall_jump() -> Scenario {
    Scenario::builder("platformer_controller_wall_jump")
        .description("Enter a valid wall slide in the shaft, then jump away and verify the wall-jump side and launch vector.")
        .then(Action::Custom(Box::new(|world| {
            support::teleport_player(world, Vec2::new(-92.0, 36.0), Vec2::new(0.0, -20.0));
            support::set_scripted_control(world, -1.0, false, false, false);
        })))
        .then(Action::WaitUntil {
            label: "wall sliding".into(),
            condition: Box::new(|world| {
                support::player_state(world)
                    .is_some_and(|state| state.phase == PlatformerMotionPhase::WallSliding)
            }),
            max_frames: 90,
        })
        .then(Action::Screenshot("wall_slide".into()))
        .then(Action::Custom(Box::new(|world| {
            support::set_scripted_control(world, -1.0, true, true, false);
        })))
        .then(Action::WaitUntil {
            label: "wall jump launched".into(),
            condition: Box::new(|world| {
                let diagnostics = world.resource::<support::DiagnosticsResource>();
                let log = world.resource::<LabMessageLog>();
                log.last_wall_jump_side == Some(PlatformerWallSide::Left)
                    && diagnostics.player_velocity.x > 100.0
                    && diagnostics.player_velocity.y > 100.0
            }),
            max_frames: 60,
        })
        .then(hard_assert("wall jump launched", |world| {
            let diagnostics = world.resource::<support::DiagnosticsResource>();
            let log = world.resource::<LabMessageLog>();
            log.last_wall_jump_side == Some(PlatformerWallSide::Left)
                && diagnostics.player_velocity.x > 100.0
                && diagnostics.player_velocity.y > 100.0
        }))
        .then(assertions::custom("wall jump side recorded", |world| {
            world.resource::<LabMessageLog>().last_wall_jump_side == Some(PlatformerWallSide::Left)
        }))
        .then(assertions::custom("wall jump launch is up and away", |world| {
            let diagnostics = world.resource::<support::DiagnosticsResource>();
            diagnostics.player_velocity.x > 100.0 && diagnostics.player_velocity.y > 100.0
        }))
        .then(assertions::log_summary("platformer_controller_wall_jump summary"))
        .then(Action::Screenshot("wall_jump_launch".into()))
        .build()
}

fn platformer_controller_moving_platform() -> Scenario {
    Scenario::builder("platformer_controller_moving_platform")
        .description("Ride the moving platform long enough to inherit support motion, then jump and verify horizontal carry-through.")
        .then(Action::Custom(Box::new(|world| {
            support::teleport_player(world, Vec2::new(-40.0, 4.0), Vec2::ZERO);
            support::set_scripted_control(world, 0.0, false, false, false);
        })))
        .then(Action::WaitUntil {
            label: "standing on moving platform".into(),
            condition: Box::new(|world| {
                support::player_state(world)
                    .is_some_and(|state| state.is_grounded && state.support_velocity.length() > 5.0)
            }),
            max_frames: 120,
        })
        .then(Action::Screenshot("moving_platform_ride".into()))
        .then(assertions::custom("support motion detected", |world| {
            support::player_state(world)
                .is_some_and(|state| state.support_velocity.length() > 5.0)
        }))
        .then(assertions::custom("support entity recorded", |world| {
            support::player_state(world).is_some_and(|state| state.support_entity.is_some())
        }))
        .then(Action::Custom(Box::new(|world| {
            support::set_scripted_control(world, 0.0, true, true, false);
        })))
        .then(Action::WaitUntil {
            label: "platform jump launched".into(),
            condition: Box::new(|world| {
                let diagnostics = world.resource::<support::DiagnosticsResource>();
                diagnostics.player_velocity.y > 100.0 && diagnostics.player_velocity.x.abs() > 10.0
            }),
            max_frames: 60,
        })
        .then(hard_assert("platform jump launched", |world| {
            let diagnostics = world.resource::<support::DiagnosticsResource>();
            diagnostics.player_velocity.y > 100.0 && diagnostics.player_velocity.x.abs() > 10.0
        }))
        .then(assertions::custom("jump kept inherited horizontal motion", |world| {
            world.resource::<support::DiagnosticsResource>().player_velocity.x.abs() > 10.0
        }))
        .then(assertions::log_summary("platformer_controller_moving_platform summary"))
        .then(Action::Screenshot("moving_platform_jump".into()))
        .build()
}

fn platformer_controller_one_way() -> Scenario {
    Scenario::builder("platformer_controller_one_way")
        .description("Jump up through a one-way platform, land on it, then drop back through it with explicit input.")
        .then(Action::Custom(Box::new(|world| {
            support::teleport_player(world, Vec2::new(-110.0, -116.0), Vec2::ZERO);
            support::set_scripted_control(world, 0.0, true, true, false);
        })))
        .then(Action::WaitUntil {
            label: "landed on one-way platform".into(),
            condition: Box::new(|world| {
                let diagnostics = world.resource::<support::DiagnosticsResource>();
                diagnostics.grounded
                    && diagnostics.player_position.y > -70.0
                    && diagnostics.player_velocity.y.abs() < 10.0
            }),
            max_frames: 180,
        })
        .then(Action::Screenshot("one_way_landed".into()))
        .then(assertions::custom("one-way support recorded", |world| {
            support::player_state(world).is_some_and(|state| state.support_entity.is_some())
        }))
        .then(Action::Custom(Box::new(|world| {
            support::set_scripted_control(world, 0.0, false, false, true);
        })))
        .then(Action::WaitUntil {
            label: "fell through one-way platform".into(),
            condition: Box::new(|world| {
                let diagnostics = world.resource::<support::DiagnosticsResource>();
                let log = world.resource::<LabMessageLog>();
                log.landed_count >= 2 && diagnostics.player_position.y < -85.0
            }),
            max_frames: 120,
        })
        .then(hard_assert("fell through one-way platform", |world| {
            let diagnostics = world.resource::<support::DiagnosticsResource>();
            let log = world.resource::<LabMessageLog>();
            log.landed_count >= 2 && diagnostics.player_position.y < -85.0
        }))
        .then(assertions::custom("drop-through moved below platform", |world| {
            let diagnostics = world.resource::<support::DiagnosticsResource>();
            let log = world.resource::<LabMessageLog>();
            log.landed_count >= 2 && diagnostics.player_position.y < -85.0
        }))
        .then(assertions::log_summary("platformer_controller_one_way summary"))
        .then(Action::Screenshot("one_way_drop".into()))
        .build()
}
