# Saddle Character Platformer Controller

Reusable 2D platformer controller for Bevy, built on `avian2d` with an input-agnostic movement boundary.

The crate is designed as a shared foundation rather than a game-specific protagonist script. Consumers wire it into their own schedules, feed `PlatformerMovementIntent` from any source, and order against the public system sets.

## Quick Start

```toml
[dependencies]
bevy = "0.18"
avian2d = "0.6.0-rc.1"
saddle-character-platformer-controller = { git = "https://github.com/julien-blanchon/saddle-character-platformer-controller" }
```

```rust
use avian2d::prelude::*;
use bevy::prelude::*;
use saddle_character_platformer_controller::{
    PlatformerControllerBundle, PlatformerControllerPlugin, PlatformerControllerSystems,
};

#[derive(States, Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum DemoState {
    #[default]
    Gameplay,
}

fn main() {
    App::new()
        .insert_resource(Time::<Fixed>::from_hz(60.0))
        .add_plugins((DefaultPlugins, PhysicsPlugins::default()))
        .init_state::<DemoState>()
        .add_plugins(PlatformerControllerPlugin::new(
            OnEnter(DemoState::Gameplay),
            OnExit(DemoState::Gameplay),
            FixedUpdate,
        ))
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

Populate `PlatformerMovementIntent` from keyboard input, `bevy_enhanced_input`, AI, replays, or networking code. The controller runtime itself does not require any input plugin.

## Public API

| Type | Purpose |
| --- | --- |
| `PlatformerControllerPlugin` | Registers the runtime with injectable `activate`, `deactivate`, and `update` schedules |
| `PlatformerControllerSystems` | Public ordering hooks: `ReadIntent`, `SenseContacts`, `ApplyMovement`, `ApplyDash`, `ApplyGroundPound`, `ApplyJump`, `WallInteractions`, `ApplyGrapple`, `MoveControllers`, `SyncState` |
| `PlatformerControllerBundle` | Minimal spawn bundle for a kinematic controller entity |
| `PlatformerControllerConfig` | Gameplay-facing tuning for movement, jumps, dash, corner correction, walls, sensing, platform interaction, ground pound, grapple, and `MoveAndSlide` |
| `PlatformerMovementIntent` | Generic input boundary: horizontal axis, jump press/hold, dash press/direction, drop-through, ground pound, and grapple controls |
| `PlatformerControllerState` | Readable runtime state: grounded/wall contacts, phase, forgiveness timers, support motion, remaining air jumps, remaining dash charges, grapple phase, and active surface modifier |
| `PlatformerOneWayPlatform` | Marker for jump-through platforms |
| `PlatformerSurfaceModifier` | Per-surface friction, speed, and conveyor-velocity modifiers (attach to ground entities) |
| `PlatformerGrapplePoint` | Marker for entities that can be targeted by the grapple hook |
| `PlatformerControllerDebugPlugin` | Optional gizmo-based debug overlay for probes and velocity |
| Messages | `JumpStarted`, `DashStarted`, `Landed`, `WallJumpStarted`, `AirJumpConsumed`, `GroundPoundStarted`, `GroundPoundImpact`, `WallClingStarted`, `GrappleAttached`, `GrappleDetached` |

## Movement Scope

Supported in `0.1.0`:

- Ground movement with separate ground and air acceleration/deceleration
- Jump height derived from `height + time_to_apex`
- Variable jump height via jump cut / low-jump gravity
- Configurable terminal velocity (`max_fall_speed`)
- Coyote time
- Jump buffering
- Configurable air jumps (`max_air_jumps`)
- Directional dash with configurable charges, cooldown, and grounded refill policy
- Wall slide with contact filtering and terminal speed clamp
- Wall jump with tunable launch and steering lock window
- Wall cling with configurable duration and gravity
- Ceiling-lip corner correction / head-bonk forgiveness
- Ledge assist (horizontal nudge when barely missing a landing platform)
- Walkable-slope filtering via `max_walkable_angle`
- Moving-platform support with configurable velocity inheritance
- One-way / jump-through platforms with explicit drop-through input
- Per-surface physics modifiers (ice, conveyor belts, mud via `PlatformerSurfaceModifier`)
- Ground pound (hover + slam + impact stun)
- Rope / grappling hook with pendulum swing physics
- Optional debug gizmos

Currently deferred:

- Runtime config blending helpers for powerups or biome variants
- Built-in animation-state or FX binding helpers beyond the public state/messages

## Plugin Setup

The controller is schedule-injectable:

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

Map the public system sets into your own ordering pipeline when needed:

```rust
app.configure_sets(
    FixedUpdate,
    PlatformerControllerSystems::ReadIntent.before(MyGameSet::Simulation),
);
```

## Tuning Overview

- `movement.*` controls lateral feel: top speed, acceleration, deceleration, and apex air control.
- `jump.*` controls arc shape and forgiveness: apex time, gravity multipliers, coyote time, jump buffering, extra jumps, and terminal velocity.
- `dash.*` controls burst traversal: dash distance, duration, cooldown, charge count, and grounded refill behavior.
- `corner_correction.*` controls ceiling-lip forgiveness, ledge assist distance, and nudge step size.
- `walls.*` controls wall validity, slide drag, jump launch, post-wall-jump steering, and wall cling duration/gravity.
- `sensing.*` controls walkable-angle classification, probe distance, and one-way platform filtering.
- `platforms.*` controls support-velocity inheritance and drop-through duration.
- `ground_pound.*` controls hover duration, slam speed, horizontal cancellation, and impact stun.
- `grapple.*` controls range, pull speed, rope physics, swing input force, and detach boost.
- `move_and_slide.*` controls Avian kinematic solver tolerances.

See [Configuration](docs/configuration.md) for the full parameter reference.

## Examples

Every shipped example now includes `saddle-pane` controls for the main movement and camera tuning knobs.

| Example | Purpose | Run |
| --- | --- | --- |
| `basic` | Minimal direct-intent demo with ground movement, coyote time, and double jump | `cargo run -p saddle-character-platformer-controller-example-basic` |
| `wall_jumps` | Wall slide filtering and wall-jump launch tuning | `cargo run -p saddle-character-platformer-controller-example-wall-jumps` |
| `moving_platforms` | Support motion inheritance on kinematic platforms | `cargo run -p saddle-character-platformer-controller-example-moving-platforms` |
| `one_way_platforms` | Jump-through floors and drop-through input | `cargo run -p saddle-character-platformer-controller-example-one-way-platforms` |
| `ground_pound` | Mid-air slam with hover, impact stun, and configurable physics | `cargo run -p saddle-character-platformer-controller-example-ground-pound` |
| `surface_modifiers` | Per-surface friction (ice, conveyor, mud) via `PlatformerSurfaceModifier` | `cargo run -p saddle-character-platformer-controller-example-surface-modifiers` |
| `grapple` | Rope-swing physics with grapple points, retract/extend, and momentum boost | `cargo run -p saddle-character-platformer-controller-example-grapple` |
| `bevy_enhanced_input` | Optional `bevy_enhanced_input` adapter feeding `PlatformerMovementIntent` | `cargo run -p saddle-character-platformer-controller-example-bevy-enhanced-input` |
| `full_demo` | Cross-crate 2D platformer showcase: controller + spritesheet + sprite effects + parallax + pixel camera | `cargo run -p saddle-character-platformer-controller-example-full-demo` |

## Crate-Local Lab

The crate also ships a lab app with BRP and targeted E2E scenarios:

```bash
cargo run -p saddle-character-platformer-controller-lab
cargo run -p saddle-character-platformer-controller-lab --features e2e -- platformer_controller_smoke
cargo run -p saddle-character-platformer-controller-lab --features e2e -- platformer_controller_dash
```

The lab overlays support velocity, support entity, and forgiveness timers so screenshots and BRP inspection expose the same state that the crate-local scenarios assert against.

See [examples/lab/README.md](examples/lab/README.md) for the lab workflow.

## Dependencies

- Runtime: `bevy = "0.18"`, `avian2d = "0.6.0-rc.1"`
- Optional adapter example: `bevy_enhanced_input = "0.24"`
- Lab-only verification: `bevy_e2e`, `bevy_brp_extras`

## Known Limitations

- The controller assumes a global `Vec2::Y` up-axis; arbitrary gravity directions are out of scope.
- One-way platform behavior is optimized for upward-facing platforms and does not attempt arbitrary rotated ghost-platform semantics.
- Moving-platform support derives velocity from `LinearVelocity` when available, otherwise from successive `Position` samples; discontinuous teleports of support bodies will still feel discontinuous.
