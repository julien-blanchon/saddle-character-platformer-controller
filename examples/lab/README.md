# Platformer Controller Lab

Crate-local lab for inspecting and verifying the shared `saddle-character-platformer-controller` crate in a real Bevy app.

## Purpose

- keep a richer verification surface than the standalone examples
- expose controller diagnostics and messages for BRP inspection
- keep timer/support state visible in screenshots so E2E failures are easier to interpret
- run targeted E2E scenarios against the public intent boundary
- verify moving platforms, coyote jumps, jump buffering, wall jumps, and one-way platforms without pushing project-specific code into the shared runtime

## Status

Working

## Run

```bash
cargo run -p saddle-character-platformer-controller-lab
```

## E2E

```bash
cargo run -p saddle-character-platformer-controller-lab --features e2e -- platformer_controller_smoke
cargo run -p saddle-character-platformer-controller-lab --features e2e -- platformer_controller_coyote_jump
cargo run -p saddle-character-platformer-controller-lab --features e2e -- platformer_controller_jump_buffer
cargo run -p saddle-character-platformer-controller-lab --features e2e -- platformer_controller_dash
cargo run -p saddle-character-platformer-controller-lab --features e2e -- platformer_controller_wall_jump
cargo run -p saddle-character-platformer-controller-lab --features e2e -- platformer_controller_moving_platform
cargo run -p saddle-character-platformer-controller-lab --features e2e -- platformer_controller_one_way
```

## BRP

The lab uses a dedicated BRP port `15732` by default.

```bash
cargo run -p saddle-character-platformer-controller-lab
BRP_PORT=15732 uv run --project .codex/skills/bevy-brp/script brp status
BRP_PORT=15732 uv run --project .codex/skills/bevy-brp/script brp world query bevy_ecs::name::Name
BRP_PORT=15732 uv run --project .codex/skills/bevy-brp/script brp resource list | rg 'DemoDiagnostics|LabMessageLog|ScriptedControl'
BRP_PORT=15732 uv run --project .codex/skills/bevy-brp/script brp extras screenshot /tmp/saddle-character-platformer-controller-lab.png
```

## Notes

- The lab selects the scene automatically from the E2E scenario name.
- E2E scenarios drive the public `PlatformerMovementIntent` boundary through a scripted lab resource instead of touching private runtime internals.
- `DemoDiagnostics`, `LabMessageLog`, and `ScriptedControl` are reflected resources so BRP can inspect the exact state the scenarios assert against.
- Set `PLATFORMER_CONTROLLER_DEBUG=1` to enable probe and velocity gizmos in the lab.
