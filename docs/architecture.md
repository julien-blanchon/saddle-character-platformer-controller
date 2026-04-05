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
4. `ApplyDash`
5. `ApplyGroundPound`
6. `ApplyJump`
7. `WallInteractions`
8. `ApplyGrapple`
9. `MoveControllers`
10. `SyncState`

The order is intentional:

- `ReadIntent` snapshots buffered player intent and decrements timers once per frame
- `SenseContacts` samples the pre-move world state for coyote time, wall validity, and support motion
- `ApplyMovement` resolves horizontal acceleration against the current support policy
- `ApplyDash` converts dash intent into a temporary authored movement phase before jump/gravity logic runs
- `ApplyGroundPound` handles hover → slam → impact stun phases, overriding velocity during each
- `ApplyJump` resolves jump buffering, coyote jumps, air jumps, gravity shaping, terminal velocity clamping, and wall-jump launch
- `WallInteractions` applies slide-specific downward clamping and wall-cling tracking after jump logic
- `ApplyGrapple` handles grapple firing, pendulum swing physics, rope constraint, and detachment
- `MoveControllers` performs the actual `MoveAndSlide` step, corner correction, ledge assist, ground snapping, and landing detection
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
- `dash_time_remaining`
- `dash_cooldown_remaining`

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

## Dash Model

Dash intent is handled as a dedicated authored phase rather than as a one-frame impulse.

- dash direction prefers the explicit `dash_direction` vector when it exceeds `direction_input_threshold`
- otherwise the runtime falls back to horizontal movement input, current lateral velocity, and finally the last facing sign
- each dash consumes one authored charge from `remaining_dashes`
- grounded contact can optionally refill those charges immediately through `dash.refill_on_ground`
- while `dash_time_remaining > 0`, horizontal movement, jump resolution, and wall-slide clamping all yield to the dash velocity

This keeps dash behavior deterministic and easy to test while still leaving the input source fully game-defined.

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

## Corner Correction and Ledge Assist

Corner correction is applied inside the movement step rather than as a separate teleporting hack.

- after the initial `MoveAndSlide` pass, the runtime detects upward head-bonks that stripped too much vertical motion
- it then retries the same move from small sideways offsets, using `corner_correction.step_size` up to `corner_correction.max_distance`
- a retry is accepted only if it produces a meaningful height gain (`min_height_gain`)

Ledge assist is the horizontal equivalent for landing:

- when the character was airborne and barely misses a ledge edge while falling, the runtime tries small horizontal nudges
- uses the same `step_size` increment up to `ledge_assist_distance`
- only activates when the character was previously airborne (not when walking off a ledge, which should trigger coyote time instead)

## Ground Pound

Ground pound is a three-phase downward action:

1. **Hover**: velocity is zeroed for `hover_duration` seconds (can be skipped with `0.0`)
2. **Slam**: velocity is set to `(0, -fall_speed)` with optional horizontal cancellation
3. **Impact stun**: on ground contact, movement freezes for `impact_stun_duration`

The ground pound overrides all other movement and gravity logic during its active phases. It can be cancelled by dash activation. Fires `GroundPoundStarted` on activation and `GroundPoundImpact` on landing.

## Grapple Hook

The grapple implements pendulum swing physics:

- **Firing**: aim-assisted search for the nearest `PlatformerGrapplePoint` within `max_range` and `aim_assist_angle`
- **Swing**: gravity applies normally (scaled by `swing_gravity_multiplier`), horizontal input adds tangential force
- **Rope constraint**: when the character reaches rope length, velocity is projected tangentially (radial-outward component removed)
- **Pull**: optional `pull_speed` pulls the character toward the anchor
- **Retract/extend**: player can shorten/lengthen the rope via intent

Detaching (jump or explicit release) applies `detach_speed_boost` to current velocity for momentum carry.

## Surface Modifiers

`PlatformerSurfaceModifier` is a component attached to ground entities that modifies movement physics on contact:

- `friction_multiplier`: scales acceleration/deceleration (0.0 = ice, 1.0 = normal, >1.0 = sticky)
- `surface_velocity`: constant velocity added while on the surface (conveyor belts)
- `speed_multiplier`: scales maximum speed on the surface

The modifier is resolved each frame from the current ground contact entity.

## Wall Cling

Wall cling is a timed mechanic that temporarily arrests downward motion on a wall:

- activates when the character touches a valid wall while falling and `wall_cling_max_duration > 0`
- during cling, gravity is scaled by `wall_cling_gravity_multiplier` (0.0 = full stop)
- after the cling timer expires, normal wall slide resumes
- wall jump during cling launches away from the wall as normal
- fires `WallClingStarted` message once on transition (not every frame)

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
- remaining dash charges and dash timers
- current wall contact
- ground pound status
- grapple phase and rope length
- active surface modifier

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
