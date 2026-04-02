# Architecture

## Controller Model

`saddle-character-platformer-controller` uses a **kinematic Avian2D body** driven by `MoveAndSlide`, with contact sensing handled as a first-class subsystem through explicit shape casts.

Why this model:

- platformer feel usually depends on authored acceleration, gravity shaping, and buffered input windows rather than rigid-body impulses
- kinematic motion keeps the lateral and jump rules explicit and easy to test
- Avian2D still provides the collision, shapecast, and depenetration primitives needed for slopes, moving platforms, and one-way platform filtering

This crate does **not** try to simulate a dynamic physics avatar and then tune forces into a platformer. It uses Avian as the collision and query backend while the movement rules stay deterministic and gameplay-facing.

## System Ordering

The runtime exposes explicit phases through `PlatformerControllerSystems`:

1. `ReadIntent`
2. `SenseContacts`
3. `ApplyMovement`
4. `ApplyJump`
5. `WallInteractions`
6. `MoveControllers`
7. `SyncState`

The order is intentional:

- `ReadIntent` snapshots buffered player intent and decrements timers once per frame
- `SenseContacts` samples the pre-move world state for coyote time, wall validity, and support motion
- `ApplyMovement` resolves horizontal acceleration against the current support policy
- `ApplyJump` resolves jump buffering, coyote jumps, air jumps, gravity shaping, and wall-jump launch
- `WallInteractions` applies slide-specific downward clamping after the jump logic has finalized vertical intent
- `MoveControllers` performs the actual `MoveAndSlide` step and re-probes the world after movement
- `SyncState` publishes the readable state component and emits messages

## Contact Sensing

Contact sensing is shape-cast based rather than collision-event based.

Ground sensing:

- casts the controller collider downward with `SpatialQuery::shape_hits`
- keeps only contacts whose normal satisfies `max_walkable_angle`
- filters one-way platforms through `PlatformerOneWayPlatform`, the platform's up vector, current motion, and active drop-through timer

Wall sensing:

- casts the controller collider left and right independently
- ignores one-way platforms
- requires a sufficiently horizontal normal (`min_normal_x`)
- rejects contacts whose vertical normal component is too large (`max_vertical_normal_y`)
- rejects contacts that only touch the lower body near the feet (`max_contact_height_ratio`)

This separation keeps small geometry noise from counting as a valid wall while letting slopes still count as ground.

## Jump and Timer Design

The jump model is authored from **height** and **time to apex**:

- `base_gravity = 2 * height / time_to_apex^2`
- `jump_speed = base_gravity * time_to_apex`

Vertical feel is then shaped with multipliers:

- rise gravity
- fall gravity
- low-jump gravity for early release
- apex gravity for softer hang time near the top

Forgiveness timers are tracked in runtime state:

- `jump_buffer_remaining`
- `coyote_time_remaining`
- `wall_jump_lock_remaining`
- `drop_through_remaining`

These timers are intentionally internal bookkeeping. Consumers observe the distilled public state component instead of manipulating timer internals directly.

## Moving Platform Policy

Support bodies are resolved from the best current ground contact.

Velocity inheritance follows `PlatformVelocityInheritance`:

- `Horizontal`: inherit only the platform's `x` motion
- `Full`: inherit `x` and `y`
- `None`: ignore support velocity

Support velocity is derived in two ways:

- prefer the support body's `LinearVelocity` when it exists
- otherwise infer velocity from successive `Position` samples of the same support entity

The controller stores the last support entity and its last sampled position so kinematic platforms can still contribute useful motion.

## One-Way Platform Policy

One-way platforms are identified by the public marker component `PlatformerOneWayPlatform`.

They block only when all of these are true:

- drop-through is not currently active
- the hit normal aligns with the platform's up direction strongly enough
- the controller is not moving upward through the platform

The runtime does not try to reinterpret arbitrary sideways or inverted one-way platforms. The intended use is jump-through floors.

## Slopes

Slope handling is based on surface normals rather than special-case geometry.

- walkability is determined by `max_walkable_angle`
- the post-move pass can snap downward within `ground_snap_distance` when the controller is descending or settling
- non-walkable slopes stay non-ground and are treated like walls or slide surfaces depending on their normals

## Wall Interaction Rules

Wall sliding activates only when:

- the controller is airborne
- a valid wall contact exists on the pressed side, unless `wall_slide_requires_input` is disabled
- vertical velocity is downward

Wall jumping:

- launches away from the contacted wall using authored horizontal and vertical speeds
- clears jump buffer and coyote time
- starts a short steering lock window

During the steering lock window, horizontal input is blended by `wall_jump_steering_factor` instead of being ignored entirely. This keeps the behavior tunable between “hard lock” and “immediate air steer”.

## Corner Correction

Explicit corner correction / head-bonk forgiveness is currently deferred.

The existing runtime already gains some incidental forgiveness from shape casts and `MoveAndSlide`, but it does not perform a dedicated “nudge around the ceiling lip” correction like Celeste-style controllers often do. That absence is deliberate until the feature can be added cleanly without turning the solver into a pile of special cases.

## Debug Strategy

`PlatformerControllerDebugPlugin` provides optional gizmo visualization for:

- velocity vector
- downward ground probe direction
- left/right wall probe directions

The public `PlatformerControllerState` also mirrors the important derived facts for BRP inspection and UI overlays:

- grounded status
- motion phase
- support entity and support velocity
- buffered jump status
- remaining air jumps
- current wall contact

## Determinism Notes

The runtime is designed to be deterministic enough for repeatable **simulation-step tests**, not for lockstep networking.

Good:

- fixed-step tests with manual `Time` progression
- replay-like scripted intent feeding
- AI-driven or E2E-driven movement assertions

Not yet guaranteed:

- bit-for-bit cross-platform determinism
- rollback/prediction serialization helpers
- arbitrary gravity or custom collision-backend portability

