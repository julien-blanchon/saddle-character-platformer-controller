# Saddle Character Platformer Controller

Reusable 2D platformer controller for Bevy, built on `avian2d` with an input-agnostic movement boundary.

The crate is split into a lean core controller plus opt-in ability packs. `PlatformerControllerPlugin` handles ground movement, jumps, wall slides, wall jumps, moving platforms, one-way platforms, and surface modifiers. Dash, ground pound, and grapple are separate plugins and bundles, so entities that do not use those abilities do not carry their config, intent, or runtime state.

## Quick Start

```toml
[dependencies]
bevy = "0.18"
avian2d = "0.6.0-rc.1"
saddle-character-platformer-controller = { git = "https://github.com/julien-blanchon/saddle-character-platformer-controller" }
```

Core-only setup:

```rust
use avian2d::prelude::*;
use bevy::prelude::*;
use saddle_character_platformer_controller::{
    PlatformerControllerBundle, PlatformerControllerPlugin,
};

fn main() {
    App::new()
        .insert_resource(Time::<Fixed>::from_hz(60.0))
        .add_plugins((DefaultPlugins, PhysicsPlugins::default()))
        .add_plugins(PlatformerControllerPlugin::always_on(FixedUpdate))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Name::new("Player"),
        PlatformerControllerBundle::new(Collider::rectangle(18.0, 30.0))
            .with_transform(Transform::from_xyz(0.0, 40.0, 0.0)),
    ));

    commands.spawn((
        Name::new("Ground"),
        RigidBody::Static,
        Collider::rectangle(640.0, 24.0),
        Transform::from_xyz(0.0, -120.0, 0.0),
    ));
}
```

Opt-in dash setup:

```rust
use saddle_character_platformer_controller::{
    PlatformerControllerPlugin, PlatformerDashBundle, PlatformerDashPlugin,
};

app.add_plugins((
    PlatformerControllerPlugin::always_on(FixedUpdate),
    PlatformerDashPlugin::always_on(FixedUpdate),
));

commands.entity(player).insert(PlatformerDashBundle::default());
```

Populate `PlatformerMovementIntent` from keyboard input, `bevy_enhanced_input`, AI, replays, or networking code. Optional abilities use their own intent components:

- `PlatformerDashIntent`
- `PlatformerGroundPoundIntent`
- `PlatformerGrappleIntent`

## Public API

### Core

| Type | Purpose |
| --- | --- |
| `PlatformerControllerPlugin` | Registers the core locomotion runtime with injectable `activate`, `deactivate`, and `update` schedules |
| `PlatformerControllerSystems` | Public ordering hooks for the core pipeline: `ReadIntent`, `SenseContacts`, `ResolveDirectives`, `ApplyMovement`, `ApplyAbilityMotion`, `ApplyJump`, `WallInteractions`, `MoveControllers`, `SyncState` |
| `PlatformerControllerBundle` | Minimal spawn bundle for a kinematic controller entity |
| `PlatformerControllerConfig` | Gameplay-facing tuning for movement, jumps, corner correction, walls, sensing, platform interaction, and `MoveAndSlide` |
| `PlatformerMovementIntent` | Generic input boundary for horizontal movement, jump press/hold, and one-way drop-through |
| `PlatformerControllerDirectives` | Public single-frame hook used by ability plugins or downstream systems to suppress parts of the core locomotion pass |
| `PlatformerControllerState` | Readable core runtime state: grounded/wall contacts, motion phase, forgiveness timers, support motion, remaining air jumps, and active surface modifier |
| `PlatformerOneWayPlatform` | Marker for jump-through platforms |
| `PlatformerSurfaceModifier` | Per-surface friction, speed, and conveyor-velocity modifiers |
| `PlatformerControllerDebugPlugin` | Optional gizmo-based debug overlay for probes and velocity |

### Optional Ability Packs

| Ability | Plugin | Per-entity bundle | Main public state |
| --- | --- | --- | --- |
| Dash | `PlatformerDashPlugin` | `PlatformerDashBundle` | `PlatformerDashState` |
| Ground pound | `PlatformerGroundPoundPlugin` | `PlatformerGroundPoundBundle` | `PlatformerGroundPoundState` |
| Grapple | `PlatformerGrapplePlugin` | `PlatformerGrappleBundle` | `PlatformerGrappleState` |

Each ability pack also exposes its own config, intent, and `SystemSet` enum:

- Dash: `PlatformerDashConfig`, `PlatformerDashIntent`, `PlatformerDashSystems`
- Ground pound: `PlatformerGroundPoundConfig`, `PlatformerGroundPoundIntent`, `PlatformerGroundPoundSystems`
- Grapple: `PlatformerGrappleConfig`, `PlatformerGrappleIntent`, `PlatformerGrappleSystems`

### Ability Composition

| Type | Purpose |
| --- | --- |
| `PlatformerAbilityComposition` | Resource holding the active cross-ability policy |
| `PlatformerAbilityCompositionPolicy` | Trait for resolving activation conflicts and grapple-detach behavior |
| `PlatformerAbilityKind` | Enum identifying dash, ground pound, or grapple |
| `PlatformerAbilityActivity` | Snapshot of which optional abilities are currently active |
| `PlatformerAbilityActivationResolution` | Policy return value for allow/block/cancel decisions |
| `PlatformerAbilityConflictAction` | Per-ability keep vs cancel action |

### Messages

Core messages:

- `JumpStarted`
- `WallJumpStarted`
- `Landed`
- `AirJumpConsumed`
- `WallClingStarted`

Ability messages:

- `DashStarted`
- `GroundPoundStarted`
- `GroundPoundImpact`
- `GrappleAttached`
- `GrappleDetached`

## Movement Scope

Core locomotion in `0.1.0`:

- Ground movement with separate ground and air acceleration/deceleration
- Jump height derived from `height + time_to_apex`
- Variable jump height via jump cut / low-jump gravity
- Configurable terminal velocity (`max_fall_speed`)
- Coyote time
- Jump buffering
- Configurable air jumps (`max_air_jumps`)
- Wall slide with contact filtering and terminal speed clamp
- Wall jump with tunable launch and steering lock window
- Wall cling with configurable duration and gravity
- Ceiling-lip corner correction and ledge assist
- Walkable-slope filtering via `max_walkable_angle`
- Moving-platform support with configurable velocity inheritance
- One-way / jump-through platforms with explicit drop-through input
- Per-surface physics modifiers via `PlatformerSurfaceModifier`
- Optional debug gizmos

Optional ability packs:

- Dash: configurable charges, cooldown, grounded refill policy, authored direction selection, and burst exit speed
- Ground pound: hover, slam, impact stun, and configurable horizontal cancellation
- Grapple: aim-assisted anchor selection, pendulum swing physics, retract/extend, and detach boost

## Plugin Setup

The core controller is schedule-injectable:

```rust
app.add_plugins(PlatformerControllerPlugin::new(
    OnEnter(MyState::Gameplay),
    OnExit(MyState::Gameplay),
    FixedUpdate,
));
```

The simplest always-on setup is:

```rust
app.add_plugins(PlatformerControllerPlugin::always_on(FixedUpdate));
```

Ability plugins follow the same pattern:

```rust
app.add_plugins((
    PlatformerDashPlugin::always_on(FixedUpdate),
    PlatformerGroundPoundPlugin::always_on(FixedUpdate),
    PlatformerGrapplePlugin::always_on(FixedUpdate),
));
```

Core ordering hooks are exposed through `PlatformerControllerSystems`:

```rust
app.configure_sets(
    FixedUpdate,
    PlatformerControllerSystems::ReadIntent.before(MyGameSet::Simulation),
);
```

Ability packs expose their own `SystemSet` enums for finer-grained ordering inside the optional motion pass when needed.

## Ability Composition

Cross-ability arbitration is no longer hardcoded inside the shared core types. Instead, ability plugins consult `PlatformerAbilityComposition`.

Default policy:

- dash is blocked while ground pound or grapple is active
- ground pound is blocked while dash or grapple is active
- grapple can cancel ground pound on activation, but is blocked by dash or another grapple
- jump detaches grapple by default

Override it by inserting your own policy resource:

```rust
use std::sync::Arc;

use saddle_character_platformer_controller::{
    PlatformerAbilityActivity, PlatformerAbilityActivationResolution,
    PlatformerAbilityComposition, PlatformerAbilityCompositionPolicy,
    PlatformerAbilityConflictAction, PlatformerAbilityKind,
};

struct MyPolicy;

impl PlatformerAbilityCompositionPolicy for MyPolicy {
    fn resolve_activation(
        &self,
        requested: PlatformerAbilityKind,
        _active: PlatformerAbilityActivity,
    ) -> PlatformerAbilityActivationResolution {
        match requested {
            PlatformerAbilityKind::Dash => PlatformerAbilityActivationResolution {
                allow_requested: true,
                dash: PlatformerAbilityConflictAction::Keep,
                ground_pound: PlatformerAbilityConflictAction::Cancel,
                grapple: PlatformerAbilityConflictAction::Keep,
            },
            _ => PlatformerAbilityActivationResolution::allow(),
        }
    }
}

app.insert_resource(PlatformerAbilityComposition(Arc::new(MyPolicy)));
```

## Tuning Overview

Core tuning lives on `PlatformerControllerConfig`:

- `movement.*` controls lateral feel
- `jump.*` controls jump arc shape and forgiveness
- `corner_correction.*` controls head-bonk forgiveness and ledge assist
- `walls.*` controls wall validity, wall slide, wall jump, and wall cling
- `sensing.*` controls walkable-angle classification, probe distance, and one-way filtering
- `platforms.*` controls support-velocity inheritance and drop-through duration
- `move_and_slide.*` controls Avian kinematic solver tolerances

Optional ability tuning lives on separate components:

- `PlatformerDashConfig`
- `PlatformerGroundPoundConfig`
- `PlatformerGrappleConfig`

See [Configuration](docs/configuration.md) for the full parameter reference.

## Examples

Every shipped example includes on-screen instructions and live parameter editing through `saddle-pane`.

| Example | Focus | Run |
| --- | --- | --- |
| `basic` | Core movement, coyote time, jump buffer, and air jumps | `cargo run -p saddle-character-platformer-controller-example-basic` |
| `wall_jumps` | Core wall slide, wall jump, and steering lock tuning | `cargo run -p saddle-character-platformer-controller-example-wall-jumps` |
| `moving_platforms` | Core moving-platform support and velocity inheritance | `cargo run -p saddle-character-platformer-controller-example-moving-platforms` |
| `one_way_platforms` | Core jump-through floors and drop-through input | `cargo run -p saddle-character-platformer-controller-example-one-way-platforms` |
| `surface_modifiers` | Core per-surface friction, speed, and conveyor modifiers | `cargo run -p saddle-character-platformer-controller-example-surface-modifiers` |
| `ground_pound` | Core controller plus `PlatformerGroundPoundPlugin` | `cargo run -p saddle-character-platformer-controller-example-ground-pound` |
| `grapple` | Core controller plus `PlatformerGrapplePlugin` | `cargo run -p saddle-character-platformer-controller-example-grapple` |
| `bevy_enhanced_input` | `bevy_enhanced_input` adapter feeding the core controller and dash ability intents | `cargo run -p saddle-character-platformer-controller-example-bevy-enhanced-input` |
| `full_demo` | Cross-crate demo using the core controller plus dash | `cargo run -p saddle-character-platformer-controller-example-full-demo` |

## Crate-Local Lab

The crate also ships a lab app with BRP and targeted E2E scenarios:

```bash
cargo run -p saddle-character-platformer-controller-lab
cargo run -p saddle-character-platformer-controller-lab --features e2e -- platformer_controller_smoke
cargo run -p saddle-character-platformer-controller-lab --features e2e -- platformer_controller_dash
cargo run -p saddle-character-platformer-controller-lab --features e2e -- platformer_controller_default_ability_policy
```

The lab overlays support velocity, support entity, forgiveness timers, and ability state so screenshots and BRP inspection expose the same state that the crate-local scenarios assert against.

See [examples/lab/README.md](examples/lab/README.md) for the lab workflow.

## Dependencies

- Runtime: `bevy = "0.18"`, `avian2d = "0.6.0-rc.1"`
- Optional adapter example: `bevy_enhanced_input = "0.24"`
- Lab-only verification: `bevy_e2e`, `bevy_brp_extras`

## Known Limitations

- The controller assumes a global `Vec2::Y` up-axis.
- One-way platform behavior is optimized for upward-facing platforms.
- Moving-platform support prefers `LinearVelocity`, otherwise it infers velocity from successive positions.
- The crate is deterministic enough for fixed-step tests and scripted replay-style verification, but it does not currently provide rollback or lockstep helpers.
