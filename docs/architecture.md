# Architecture

## Modular Controller Model

`saddle-character-platformer-controller` now has two layers:

- a **core locomotion plugin** for movement, jump, wall interaction, slopes, moving platforms, one-way platforms, and surface modifiers
- **optional ability plugins** for dash, ground pound, and grapple

This split keeps the base API small:

- `PlatformerControllerBundle` only contains core controller components
- `PlatformerControllerConfig`, `PlatformerMovementIntent`, and `PlatformerControllerState` are core-only
- ability-specific config, intent, state, runtime data, and messages live in the matching ability module

The design goal is to let one game use only the core controller, another add dash, and another install all three ability packs without paying for unrelated state on every entity.

## Why A Kinematic Controller

The crate uses a **kinematic Avian2D body** driven by `MoveAndSlide`, with contact sensing handled as a first-class subsystem through explicit shape casts.

Why this model:

- platformer feel depends on authored acceleration, gravity shaping, and buffered input windows more than rigid-body impulses
- kinematic motion keeps the lateral and jump rules explicit and easy to test
- Avian2D still provides the collision, shapecast, and depenetration primitives needed for slopes, moving platforms, and one-way filtering

The crate does not try to simulate a dynamic avatar and then tune forces into a platformer. Avian is the collision/query backend while the movement rules stay deterministic and gameplay-facing.

## Core System Ordering

The runtime exposes explicit phases through `PlatformerControllerSystems`:

1. `ReadIntent`
2. `SenseContacts`
3. `ResolveDirectives`
4. `ApplyMovement`
5. `ApplyAbilityMotion`
6. `ApplyJump`
7. `WallInteractions`
8. `MoveControllers`
9. `SyncState`

The order is intentional:

- `ReadIntent` snapshots buffered player intent and decrements timers once per frame
- `SenseContacts` samples the pre-move world state for coyote time, wall validity, and support motion
- `ResolveDirectives` gives optional ability plugins or downstream systems a place to suppress parts of the core locomotion pass before it runs
- `ApplyMovement` resolves horizontal acceleration against the current support policy
- `ApplyAbilityMotion` is a reserved lane where ability plugins can author velocity before jump/gravity logic
- `ApplyJump` resolves jump buffering, coyote jumps, air jumps, gravity shaping, terminal velocity clamping, and wall-jump launch
- `WallInteractions` applies wall-slide and wall-cling behavior after jump logic
- `MoveControllers` performs the actual `MoveAndSlide` step, corner correction, ledge assist, ground snapping, and landing detection
- `SyncState` publishes readable state components and emits messages

## Ability Plugin Ordering

Each optional ability plugin exposes its own `SystemSet` enum and plugs itself into the shared core schedule:

- dash: `PlatformerDashSystems::{ResolveDirectives, ApplyDash, SyncState}`
- ground pound: `PlatformerGroundPoundSystems::{ResolveDirectives, ApplyGroundPound, SyncState}`
- grapple: `PlatformerGrappleSystems::{ResolveDirectives, ApplyGrapple, SyncState}`

Those systems do two things:

- write `PlatformerControllerDirectives` to temporarily suppress core locomotion work when an ability owns movement for the frame
- maintain ability-specific runtime state and mirror that state onto public ability components

This means the core controller does not need hardcoded fields like “remaining dashes” or “grapple phase” to support optional traversal packs.

## Contact Sensing

Contact sensing is shape-cast based rather than collision-event based.

Ground sensing:

- casts the controller collider downward with `SpatialQuery::shape_hits`
- keeps only contacts whose normal satisfies `max_walkable_angle`
- filters one-way platforms through `PlatformerOneWayPlatform`, the platform up vector, current motion, and the active drop-through timer

Wall sensing:

- casts the controller collider left and right independently
- ignores one-way platforms
- requires a sufficiently horizontal normal (`min_normal_x`)
- rejects contacts whose vertical normal component is too large (`max_vertical_normal_y`)
- rejects contacts that only touch the lower body near the feet (`max_contact_height_ratio`)

This separation keeps small geometry noise from counting as a valid wall while still letting slopes count as ground.

## Core Runtime State

The core runtime tracks only core bookkeeping:

- jump buffer time remaining
- coyote time remaining
- wall-jump steering lock time remaining
- one-way drop-through time remaining
- remaining air jumps
- support velocity and support entity tracking
- pre-move and post-move ground/wall contacts
- wall cling timers
- surface modifier and pending core messages
- `PlatformerControllerDirectives`

The public `PlatformerControllerState` mirrors the gameplay-relevant subset of that data:

- grounded status
- motion phase
- ground and wall contacts
- support entity and support velocity
- readable forgiveness timers
- remaining air jumps
- active surface modifier

Ability packs mirror their own state on separate components:

- `PlatformerDashState`
- `PlatformerGroundPoundState`
- `PlatformerGrappleState`

## Jump and Timer Design

The jump model is authored from **height** and **time to apex**:

- `base_gravity = 2 * height / time_to_apex^2`
- `jump_speed = base_gravity * time_to_apex`

Vertical feel is then shaped with:

- rise gravity
- fall gravity
- low-jump gravity
- apex gravity

This keeps jump tuning in gameplay-facing units instead of arbitrary impulses.

## Moving Platform Policy

Support bodies are resolved from the best current ground contact.

Velocity inheritance follows `PlatformVelocityInheritance`:

- `Horizontal`: inherit only the platform's `x` motion
- `Full`: inherit both axes
- `None`: ignore support velocity

Support velocity is derived in two ways:

- prefer the support body's `LinearVelocity` when it exists
- otherwise infer velocity from successive `Position` samples of the same support entity

The controller stores the last support entity and its last sampled position so kinematic platforms can still contribute useful motion.

## Ability Composition Policy

Cross-ability arbitration is intentionally centralized in `PlatformerAbilityComposition` rather than spread across hardcoded if/else branches in multiple systems.

The policy trait exposes two hooks:

- `resolve_activation(requested, active)` decides whether a new activation request is allowed and whether any currently active abilities should be cancelled
- `detach_grapple_on_jump(active)` decides whether jump input should detach grapple

Default policy:

- dash is blocked by active ground pound or grapple
- ground pound is blocked by active dash or grapple
- grapple is blocked by active dash or another grapple
- grapple cancels active ground pound when it attaches
- jump detaches grapple

Consumers can replace the resource to author different arbitration without forking the plugin.

## Dash Model

Dash is implemented as an optional authored-motion layer, not a core controller phase.

- dash direction prefers explicit `PlatformerDashIntent.direction` when it exceeds `direction_input_threshold`
- otherwise it falls back to movement input, current lateral velocity, and finally facing sign
- each dash consumes one charge from the dash runtime state
- grounded contact can optionally refill charges via `refill_on_ground`
- while active, dash suppresses core horizontal movement, jump logic, and wall interactions through `PlatformerControllerDirectives`

Because dash is its own plugin, entities without `PlatformerDashBundle` pay none of this state or logic.

## Ground Pound Model

Ground pound is a three-phase optional action:

1. `Hovering`
2. `Slamming`
3. `ImpactStun`

During those phases the plugin:

- authors vertical velocity directly
- optionally zeros horizontal velocity
- suppresses core movement, jump logic, and wall interactions
- emits `GroundPoundStarted` and `GroundPoundImpact`

Its phase and timers are mirrored on `PlatformerGroundPoundState`, not on the core controller state.

## Grapple Model

Grapple is an optional pendulum-motion layer.

Firing:

- searches for the nearest `PlatformerGrapplePoint` within `max_range`
- uses `aim_assist_angle` to accept nearby anchors in the aimed direction

While attached:

- gravity applies with `swing_gravity_multiplier`
- horizontal input adds tangential force
- rope length is clamped with retract/extend input
- outward radial velocity is projected away to keep the body on the rope constraint
- optional `pull_speed` can reel the player toward the anchor

Detaching:

- happens on explicit release or jump, depending on the composition policy
- applies `detach_speed_boost` to carry momentum
- emits `GrappleDetached`

As with the other ability packs, grapple lives outside the core motion phase enum.

## One-Way Platform Policy

One-way platforms are identified by `PlatformerOneWayPlatform`.

They block only when all of these are true:

- drop-through is not currently active
- the hit normal aligns strongly enough with the platform up direction
- the controller is not moving upward through the platform

The intended use is jump-through floors rather than arbitrary rotated one-way geometry.

## Slopes

Slope handling is normal-based rather than geometry-special-cased.

- walkability is determined by `max_walkable_angle`
- the post-move pass can snap downward within `ground_snap_distance`
- non-walkable slopes stay non-ground and are treated like walls or slide surfaces based on their normals

## Wall Interaction Rules

Wall sliding activates only when:

- the controller is airborne
- a valid wall contact exists on the pressed side, unless `wall_slide_requires_input` is disabled
- vertical velocity is downward

Wall jumping:

- launches away from the contacted wall with authored horizontal and vertical speeds
- clears jump buffer and coyote time
- starts a short steering lock window

During the steering lock window, horizontal input is blended by `wall_jump_steering_factor` rather than being ignored entirely.

## Corner Correction and Ledge Assist

Corner correction is applied inside the movement step.

- after the initial `MoveAndSlide` pass, the runtime detects upward head-bonks that stripped too much vertical motion
- it retries the same move from small sideways offsets using `corner_correction.step_size` up to `corner_correction.max_distance`
- a retry is accepted only if it produces a meaningful height gain

Ledge assist is the horizontal equivalent for landing:

- when the character was airborne and barely misses a ledge edge while falling, the runtime tries small horizontal nudges
- it uses the same step size up to `ledge_assist_distance`
- it only activates when the character was previously airborne

## Surface Modifiers

`PlatformerSurfaceModifier` is attached to ground entities and modifies movement physics on contact:

- `friction_multiplier`: scales acceleration and deceleration
- `surface_velocity`: adds conveyor-style velocity
- `speed_multiplier`: scales top speed on the surface

The modifier is resolved each frame from the current ground contact entity.

## Wall Cling

Wall cling is a timed mechanic layered into the core wall interaction pass:

- activates when the character touches a valid wall while falling and `wall_cling_max_duration > 0`
- gravity is scaled by `wall_cling_gravity_multiplier`
- after the cling timer expires, normal wall slide resumes
- wall jump during cling launches away as normal
- the crate emits `WallClingStarted` once on transition

## Debug and Verification Strategy

`PlatformerControllerDebugPlugin` provides optional gizmo visualization for:

- velocity vector
- downward ground probe direction
- left and right wall probe directions

The intended verification surface is:

- `PlatformerControllerState` for core locomotion
- per-ability public state components for optional packs
- public messages for state transitions and authored actions
- fixed-step simulation tests and crate-local E2E scenarios

This keeps the runtime testable without exposing private timer internals or tying the crate to one specific input stack.
