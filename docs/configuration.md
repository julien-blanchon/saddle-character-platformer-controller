# Configuration

The crate now separates core locomotion tuning from optional ability tuning:

- every controller entity owns one `PlatformerControllerConfig`
- entities only own `PlatformerDashConfig`, `PlatformerGroundPoundConfig`, or `PlatformerGrappleConfig` if you insert the matching ability bundle
- cross-ability arbitration lives in the `PlatformerAbilityComposition` resource rather than inside the per-entity config

Ranges below are gameplay-oriented guidance, not hard validation limits.

## Core `PlatformerControllerConfig`

`PlatformerControllerConfig` contains seven authored groups:

- `movement`
- `jump`
- `corner_correction`
- `walls`
- `sensing`
- `platforms`
- `move_and_slide`

### `movement`

| Field | Type | Default | Recommended Range | Effect | Notable Interactions |
| --- | --- | --- | --- | --- | --- |
| `max_speed` | `f32` | `220.0` | `120.0..320.0` | Top horizontal speed on flat ground | Higher values usually need higher acceleration to avoid sluggishness |
| `ground_acceleration` | `f32` | `1800.0` | `600.0..3200.0` | How quickly the controller reaches target ground speed | Pairs with `max_speed`; very high values feel snappy but less analog |
| `ground_deceleration` | `f32` | `2400.0` | `800.0..3600.0` | How quickly the controller stops or reverses on ground | Raising this tightens precision platforming feel |
| `air_acceleration` | `f32` | `1100.0` | `300.0..2200.0` | Horizontal steering while airborne | Higher values make air correction easier and combine strongly with apex assist |
| `air_deceleration` | `f32` | `700.0` | `150.0..1800.0` | How quickly lateral speed bleeds off in air when input is released | Low values preserve momentum; high values feel more authored |
| `apex_air_control_multiplier` | `f32` | `1.15` | `0.8..1.6` | Extra air-control multiplier near the jump apex | Works with `jump.apex_velocity_threshold` and `jump.apex_gravity_multiplier` |

### `jump`

| Field | Type | Default | Recommended Range | Effect | Notable Interactions |
| --- | --- | --- | --- | --- | --- |
| `height` | `f32` | `78.0` | `32.0..128.0` | Target jump height used to derive gravity and launch speed | Strongly coupled with `time_to_apex` |
| `time_to_apex` | `f32` | `0.42` | `0.22..0.65` | Time from jump start to peak | Lower values feel punchier; higher values feel floatier |
| `rise_gravity_multiplier` | `f32` | `1.0` | `0.7..1.4` | Gravity while ascending and still holding jump | Raising it shortens the full jump even if `height` stays the same |
| `fall_gravity_multiplier` | `f32` | `1.7` | `1.0..3.0` | Gravity while descending | Higher values speed up landings and reduce float |
| `low_jump_gravity_multiplier` | `f32` | `2.35` | `1.2..4.0` | Gravity after early jump release | Primary variable-jump-height control |
| `apex_gravity_multiplier` | `f32` | `0.8` | `0.4..1.1` | Gravity near zero vertical speed | Lower values create hang time and softer apex control |
| `apex_velocity_threshold` | `f32` | `26.0` | `6.0..40.0` | Velocity band treated as “near apex” | Affects both apex gravity and `movement.apex_air_control_multiplier` |
| `coyote_time` | `f32` | `0.1` | `0.03..0.18` | Grace window after losing ground contact | Longer values feel forgiving but less strict |
| `jump_buffer_time` | `f32` | `0.12` | `0.03..0.20` | Grace window for a jump pressed before landing | Pairs with landing stability and snap distance |
| `max_air_jumps` | `u32` | `1` | `0..3` | Number of extra jumps after leaving the ground | `0` removes double-jump behavior entirely |
| `max_fall_speed` | `f32` | `0.0` | `0.0..800.0` | Maximum downward speed. `0.0` disables the cap | Prevents extreme fall speeds in tall shafts |

Derived values:

- `base_gravity = 2 * height / time_to_apex^2`
- `jump_speed = base_gravity * time_to_apex`

### `corner_correction`

| Field | Type | Default | Recommended Range | Effect | Notable Interactions |
| --- | --- | --- | --- | --- | --- |
| `max_distance` | `f32` | `10.0` | `0.0..16.0` | Furthest sideways retry distance when a jump clips a ceiling lip | `0.0` disables explicit corner correction |
| `step_size` | `f32` | `2.0` | `1.0..4.0` | Horizontal retry increment for each correction candidate | Smaller steps are more precise but cost more retries |
| `min_upward_speed` | `f32` | `18.0` | `4.0..80.0` | Minimum upward velocity required before the solver attempts correction | Prevents low-energy bumps from jittering sideways |
| `min_height_gain` | `f32` | `1.0` | `0.25..4.0` | Required vertical improvement before a retry is accepted | Higher values make correction stricter |
| `ledge_assist_distance` | `f32` | `4.0` | `0.0..12.0` | Maximum horizontal landing nudge when barely missing a ledge | `0.0` disables ledge assist |

### `walls`

| Field | Type | Default | Recommended Range | Effect | Notable Interactions |
| --- | --- | --- | --- | --- | --- |
| `probe_distance` | `f32` | `8.0` | `2.0..16.0` | Horizontal shape-cast reach when looking for walls | Too high can make nearby noise count as walls |
| `min_normal_x` | `f32` | `0.7` | `0.5..0.95` | Minimum horizontal normal strength for a valid wall | Higher values reject slanted geometry more aggressively |
| `max_vertical_normal_y` | `f32` | `0.3` | `0.0..0.6` | Maximum allowed vertical component on a wall normal | Helps keep steep slopes from becoming walls |
| `max_contact_height_ratio` | `f32` | `0.82` | `0.5..1.0` | Highest relative contact point that still counts as a wall | Lower values reject tiny foot-level touches |
| `wall_slide_terminal_speed` | `f32` | `110.0` | `30.0..180.0` | Max downward speed while wall sliding | Lower values create stickier slides |
| `wall_slide_gravity_multiplier` | `f32` | `0.55` | `0.2..1.0` | Downward gravity multiplier during wall slide | Works with `wall_slide_terminal_speed` |
| `wall_jump_horizontal_speed` | `f32` | `235.0` | `120.0..340.0` | Horizontal launch away from the wall | Higher values may need a longer steering lock |
| `wall_jump_vertical_speed` | `f32` | `285.0` | `140.0..380.0` | Upward launch on wall jump | Usually tuned alongside normal jump height |
| `wall_jump_steering_lock_time` | `f32` | `0.14` | `0.0..0.25` | Duration of reduced steering after a wall jump | `0.0` means immediate full control |
| `wall_jump_steering_factor` | `f32` | `0.2` | `0.0..1.0` | Input blend during steering lock | `0.0` is a hard lock, `1.0` means no lock |
| `wall_slide_requires_input` | `bool` | `true` | `true` or `false` | Require holding toward the wall to slide | `false` makes accidental slides easier to trigger |
| `wall_cling_max_duration` | `f32` | `0.0` | `0.0..2.0` | Maximum time the character clings motionlessly to a wall | `0.0` disables cling |
| `wall_cling_gravity_multiplier` | `f32` | `0.0` | `0.0..1.0` | Gravity while clinging (`0.0` = full stop) | After cling expires the character resumes wall slide |

### `sensing`

| Field | Type | Default | Recommended Range | Effect | Notable Interactions |
| --- | --- | --- | --- | --- | --- |
| `max_walkable_angle` | `f32` radians | `46°` | `20°..60°` | Steepest slope that still counts as ground | Higher values reduce sliding on ramps but can blur ground vs wall boundaries |
| `ground_probe_distance` | `f32` | `10.0` | `2.0..20.0` | Downward cast reach for regular ground sensing | Too small can cause edge flicker |
| `ground_snap_distance` | `f32` | `8.0` | `0.0..20.0` | Extra downward snap when settling onto a surface | Helps landing stability and coyote consistency |
| `one_way_normal_alignment` | `f32` | `0.7` | `0.5..0.95` | Dot-product threshold between hit normal and platform up for one-way blocking | Higher values make filtering stricter |

### `platforms`

| Field | Type | Default | Recommended Range | Effect | Notable Interactions |
| --- | --- | --- | --- | --- | --- |
| `velocity_inheritance` | `PlatformVelocityInheritance` | `Horizontal` | `Horizontal`, `Full`, `None` | How support motion contributes to controller velocity | `Full` is best for elevators; `Horizontal` often feels best for side-view platformers |
| `drop_through_duration` | `f32` | `0.18` | `0.08..0.35` | How long one-way platforms ignore the controller after a drop-through press | Longer values reduce accidental re-catch |

### `move_and_slide`

These values tune the Avian2D kinematic solver rather than raw platformer feel.

| Field | Type | Default | Recommended Range | Effect | Notable Interactions |
| --- | --- | --- | --- | --- | --- |
| `skin_width` | `f32` | `0.02` | `0.001..0.08` | Separation margin used by `MoveAndSlide` | Too small can jitter; too large can feel hovery |
| `move_and_slide_iterations` | `usize` | `4` | `2..8` | Maximum movement iterations for a single step | Higher values handle complex corners better at extra cost |
| `depenetration_iterations` | `usize` | `4` | `1..8` | Maximum overlap-recovery iterations | Raise only when penetration recovery proves insufficient |
| `max_depenetration_error` | `f32` | `0.001` | `0.0001..0.02` | Error tolerance for overlap recovery | Smaller values recover more exactly but can cost more |
| `max_planes` | `usize` | `16` | `4..32` | Plane budget for slide projection | Mostly relevant in geometry-heavy scenes |

## Optional `PlatformerDashConfig`

Attach `PlatformerDashBundle` to opt an entity into dash. Its `PlatformerDashConfig` fields are:

| Field | Type | Default | Recommended Range | Effect | Notable Interactions |
| --- | --- | --- | --- | --- | --- |
| `distance` | `f32` | `84.0` | `36.0..160.0` | Authored travel distance for a full dash burst | Combined with `duration` to derive dash speed |
| `duration` | `f32` | `0.16` | `0.05..0.30` | Time the dash stays active | Lower values feel snappier |
| `cooldown` | `f32` | `0.12` | `0.0..0.40` | Delay before another dash can begin | Important when grounded dashes refill immediately |
| `max_charges` | `u32` | `1` | `0..3` | Maximum dash charges tracked at runtime | `0` disables the mechanic |
| `refill_on_ground` | `bool` | `true` | `true` or `false` | Refill dash charges whenever grounded contact is restored | Works with `allow_ground_dash` |
| `allow_ground_dash` | `bool` | `true` | `true` or `false` | Permit dash activation while grounded | Disable for air-dash-only kits |
| `preserve_vertical_velocity` | `bool` | `false` | `true` or `false` | Keep current `y` velocity during horizontal dashes | Helpful for momentum-heavy action games |
| `direction_input_threshold` | `f32` | `0.2` | `0.05..0.5` | Minimum analog direction magnitude before using `PlatformerDashIntent.direction` | Higher values favor deliberate cardinal dashes |
| `exit_speed_scale` | `f32` | `0.35` | `0.0..1.0` | Velocity multiplier applied when a dash expires | Lower values create crisp stops |

Derived value:

- `dash_speed = distance / duration`

## Optional `PlatformerGroundPoundConfig`

Attach `PlatformerGroundPoundBundle` to opt an entity into ground pound.

| Field | Type | Default | Recommended Range | Effect | Notable Interactions |
| --- | --- | --- | --- | --- | --- |
| `hover_duration` | `f32` | `0.08` | `0.0..0.3` | Brief hover before slamming downward | `0.0` skips hover |
| `fall_speed` | `f32` | `600.0` | `200.0..1200.0` | Downward speed during the slam phase | Higher values create more dramatic impacts |
| `cancel_horizontal_speed` | `bool` | `true` | `true` or `false` | Zero horizontal velocity on activation | Disable for a directional dive variant |
| `impact_stun_duration` | `f32` | `0.1` | `0.0..0.4` | Freeze duration on ground impact before movement resumes | Fires `GroundPoundImpact` with impact speed |

## Optional `PlatformerGrappleConfig`

Attach `PlatformerGrappleBundle` to opt an entity into grapple.

| Field | Type | Default | Recommended Range | Effect | Notable Interactions |
| --- | --- | --- | --- | --- | --- |
| `max_range` | `f32` | `300.0` | `100.0..600.0` | Maximum distance to search for grapple points | Points beyond this range are ignored |
| `pull_speed` | `f32` | `400.0` | `0.0..800.0` | Speed at which the character is pulled toward the anchor (`0.0` = pure swing) | Combined with swing physics |
| `detach_speed_boost` | `f32` | `1.3` | `1.0..2.0` | Velocity multiplier applied on detach | Higher values reward skillful timing |
| `aim_assist_angle` | `f32` radians | `0.35` | `0.1..0.8` | Angle tolerance for finding grapple points | Wider angles make aiming easier |
| `min_rope_length` | `f32` | `20.0` | `5.0..50.0` | Minimum rope length | Prevents clipping through the anchor |
| `retract_speed` | `f32` | `200.0` | `50.0..500.0` | Rope shortening speed per second | Player-controlled via `PlatformerGrappleIntent.retract` |
| `extend_speed` | `f32` | `100.0` | `0.0..300.0` | Rope lengthening speed per second | Player-controlled via `PlatformerGrappleIntent.extend` |
| `swing_gravity_multiplier` | `f32` | `1.0` | `0.5..2.0` | Gravity while swinging | Lower values create floatier swings |
| `swing_input_force` | `f32` | `300.0` | `0.0..600.0` | Tangential force from horizontal input while swinging | Higher values give more active swing control |

## Ability Composition Policy

Cross-ability rules are configured through the `PlatformerAbilityComposition` resource.

Policy surface:

- `resolve_activation(requested, active)` decides whether an ability request is allowed and whether any currently active abilities should be cancelled
- `detach_grapple_on_jump(active)` decides whether a jump input should detach an active grapple

Default policy:

- dash is blocked by active ground pound or grapple
- ground pound is blocked by active dash or grapple
- grapple is blocked by active dash or another grapple, but it cancels active ground pound
- jump detaches grapple

Override the resource when you want a different composition model, such as letting dash cancel ground pound, keeping grapple attached on jump, or allowing multiple actions to coexist.

## Tuning Presets

### Tight precision platformer

- lower `jump.time_to_apex`
- higher `movement.ground_deceleration`
- lower `movement.air_acceleration`
- lower `jump.apex_gravity_multiplier`
- stronger `jump.low_jump_gravity_multiplier`
- shorter `jump.coyote_time`

### Floaty exploration platformer

- higher `jump.time_to_apex`
- lower `jump.fall_gravity_multiplier`
- higher `movement.air_acceleration`
- lower `jump.low_jump_gravity_multiplier`
- allow at least one `jump.max_air_jumps`

### Sticky wall-jump challenge

- lower `walls.wall_slide_terminal_speed`
- lower `walls.wall_slide_gravity_multiplier`
- higher `walls.wall_jump_vertical_speed`
- slightly longer `walls.wall_jump_steering_lock_time`
- lower `walls.wall_jump_steering_factor`

### Aggressive action-platformer dash

- increase `PlatformerDashConfig.distance`
- lower `PlatformerDashConfig.duration`
- keep `PlatformerDashConfig.cooldown` short
- enable `PlatformerDashConfig.allow_ground_dash`
- reduce `PlatformerDashConfig.exit_speed_scale`

### Heavy slam ground pound

- increase `PlatformerGroundPoundConfig.fall_speed`
- reduce `PlatformerGroundPoundConfig.hover_duration`
- keep `PlatformerGroundPoundConfig.cancel_horizontal_speed` enabled
- increase `PlatformerGroundPoundConfig.impact_stun_duration`

### Swing-focused grapple traversal

- lower `PlatformerGrappleConfig.pull_speed` toward `0.0`
- increase `PlatformerGrappleConfig.swing_input_force`
- lower `PlatformerGrappleConfig.swing_gravity_multiplier`
- raise `PlatformerGrappleConfig.max_range`
