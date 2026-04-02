# Configuration

Every controller entity owns a `PlatformerControllerConfig` component with six authored groups:

- `movement`
- `jump`
- `walls`
- `sensing`
- `platforms`
- `move_and_slide`

Ranges below are gameplay-oriented guidance, not hard validation limits.

## `movement`

| Field | Type | Default | Recommended Range | Effect | Notable Interactions |
| --- | --- | --- | --- | --- | --- |
| `max_speed` | `f32` | `220.0` | `120.0..320.0` | Top horizontal speed on flat ground | Higher values usually need higher acceleration to avoid sluggishness |
| `ground_acceleration` | `f32` | `1800.0` | `600.0..3200.0` | How quickly the controller reaches target ground speed | Pairs with `max_speed`; very high values feel snappy but less analog |
| `ground_deceleration` | `f32` | `2400.0` | `800.0..3600.0` | How quickly the controller stops or reverses on ground | Raising this tightens precision platforming feel |
| `air_acceleration` | `f32` | `1100.0` | `300.0..2200.0` | Horizontal steering while airborne | Higher values make air correction easier and combine strongly with apex assist |
| `air_deceleration` | `f32` | `700.0` | `150.0..1800.0` | How quickly lateral speed bleeds off in air when input is released | Low values preserve momentum; high values feel more authored |
| `apex_air_control_multiplier` | `f32` | `1.15` | `0.8..1.6` | Extra air-control multiplier near the jump apex | Works with `jump.apex_velocity_threshold` and `jump.apex_gravity_multiplier` |

## `jump`

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

Derived values:

- `base_gravity = 2 * height / time_to_apex^2`
- `jump_speed = base_gravity * time_to_apex`

These formulas intentionally avoid raw “impulse guessing”.

## `walls`

| Field | Type | Default | Recommended Range | Effect | Notable Interactions |
| --- | --- | --- | --- | --- | --- |
| `probe_distance` | `f32` | `8.0` | `2.0..16.0` | Horizontal shape-cast reach when looking for walls | Too high can make nearby noise count as walls |
| `min_normal_x` | `f32` | `0.7` | `0.5..0.95` | Minimum horizontal normal strength for a valid wall | Higher values reject slanted geometry more aggressively |
| `max_vertical_normal_y` | `f32` | `0.3` | `0.0..0.6` | Maximum allowed vertical component on a wall normal | Helps keep steep slopes from becoming walls |
| `max_contact_height_ratio` | `f32` | `0.82` | `0.5..1.0` | Highest relative contact point that still counts as a wall | Lower values reject tiny foot-level touches |
| `wall_slide_terminal_speed` | `f32` | `110.0` | `30.0..180.0` | Max downward speed while wall sliding | Lower values create stickier slides |
| `wall_slide_gravity_multiplier` | `f32` | `0.55` | `0.2..1.0` | Downward gravity multiplier during wall slide | Works with `wall_slide_terminal_speed` |
| `wall_jump_horizontal_speed` | `f32` | `235.0` | `120.0..340.0` | Horizontal launch away from the wall | Higher values demand a longer steering lock in some games |
| `wall_jump_vertical_speed` | `f32` | `285.0` | `140.0..380.0` | Upward launch on wall jump | Usually tuned alongside normal jump height |
| `wall_jump_steering_lock_time` | `f32` | `0.14` | `0.0..0.25` | Duration of reduced steering after a wall jump | `0.0` means immediate full control |
| `wall_jump_steering_factor` | `f32` | `0.2` | `0.0..1.0` | Input blend during steering lock | `0.0` is a hard lock, `1.0` is no lock at all |
| `wall_slide_requires_input` | `bool` | `true` | `true` or `false` | Require holding toward the wall to slide | `false` makes accidental slides more common but easier to author |

## `sensing`

| Field | Type | Default | Recommended Range | Effect | Notable Interactions |
| --- | --- | --- | --- | --- | --- |
| `max_walkable_angle` | `f32` radians | `46°` | `20°..60°` | Steepest slope that still counts as ground | Higher values reduce sliding on ramps but can blur ground vs wall boundaries |
| `ground_probe_distance` | `f32` | `10.0` | `2.0..20.0` | Downward cast reach for regular ground sensing | Too small can cause edge flicker; too large can feel sticky |
| `ground_snap_distance` | `f32` | `8.0` | `0.0..20.0` | Extra downward snap when settling onto a surface | Helps landing stability and coyote consistency |
| `one_way_normal_alignment` | `f32` | `0.7` | `0.5..0.95` | Dot-product threshold between hit normal and platform up for one-way blocking | Higher values make one-way filtering stricter |

## `platforms`

| Field | Type | Default | Recommended Range | Effect | Notable Interactions |
| --- | --- | --- | --- | --- | --- |
| `velocity_inheritance` | `PlatformVelocityInheritance` | `Horizontal` | `Horizontal`, `Full`, `None` | How support motion contributes to controller velocity | `Full` is best for elevators; `Horizontal` often feels best for side-view platformers |
| `drop_through_duration` | `f32` | `0.18` | `0.08..0.35` | How long one-way platforms ignore the controller after a drop-through press | Longer values reduce accidental re-catch but can overshoot stacked platforms |

## `move_and_slide`

These values tune the Avian2D kinematic solver rather than raw platformer feel.

| Field | Type | Default | Recommended Range | Effect | Notable Interactions |
| --- | --- | --- | --- | --- | --- |
| `skin_width` | `f32` | `0.02` | `0.001..0.08` | Separation margin used by `MoveAndSlide` | Too small can jitter; too large can feel “hovery” |
| `move_and_slide_iterations` | `usize` | `4` | `2..8` | Maximum movement iterations for a single step | Higher values handle complex corners better at extra cost |
| `depenetration_iterations` | `usize` | `4` | `1..8` | Maximum overlap-recovery iterations | Raise only when penetration recovery proves insufficient |
| `max_depenetration_error` | `f32` | `0.001` | `0.0001..0.02` | Error tolerance for overlap recovery | Smaller values recover more exactly but can cost more |
| `max_planes` | `usize` | `16` | `4..32` | Plane budget for slide projection | Mostly relevant in geometry-heavy scenes |

## Tuning Presets

### Tight precision platformer

- lower `time_to_apex`
- higher `ground_deceleration`
- lower `air_acceleration`
- lower `apex_gravity_multiplier`
- stronger `low_jump_gravity_multiplier`
- shorter `coyote_time`

### Floaty exploration platformer

- higher `time_to_apex`
- lower `fall_gravity_multiplier`
- higher `air_acceleration`
- lower `low_jump_gravity_multiplier`
- allow at least one `max_air_jumps`

### Sticky wall-jump challenge

- lower `wall_slide_terminal_speed`
- lower `wall_slide_gravity_multiplier`
- higher `wall_jump_vertical_speed`
- slightly longer `wall_jump_steering_lock_time`
- lower `wall_jump_steering_factor`

