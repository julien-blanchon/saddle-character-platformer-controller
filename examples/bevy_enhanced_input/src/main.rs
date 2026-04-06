use saddle_character_platformer_controller_example_support as support;

use bevy::app::AppExit;
use bevy::prelude::*;
use bevy_enhanced_input::prelude::{Cancel as InputCancel, Press as InputPress, *};
use saddle_character_platformer_controller::{PlatformerDashIntent, PlatformerMovementIntent};
use support::{DemoFixedSystems, DemoPlayer, DemoScene};

#[derive(Debug, InputAction)]
#[action_output(Vec2)]
struct MoveAction;

#[derive(Debug, InputAction)]
#[action_output(bool)]
struct JumpAction;

#[derive(Debug, InputAction)]
#[action_output(bool)]
struct DropAction;

#[derive(Debug, InputAction)]
#[action_output(bool)]
struct DashAction;

fn main() -> AppExit {
    let mut app = App::new();
    support::configure_demo_app(&mut app, DemoScene::Basic, true);
    support::install_pane(&mut app);
    app.add_plugins(EnhancedInputPlugin);
    app.add_input_context::<DemoPlayer>();
    app.add_observer(cache_move_axis);
    app.add_observer(clear_move_axis_on_cancel);
    app.add_observer(clear_move_axis_on_complete);
    app.add_observer(cache_jump_press);
    app.add_observer(cache_jump_hold);
    app.add_observer(clear_jump_hold_on_cancel);
    app.add_observer(clear_jump_hold_on_complete);
    app.add_observer(cache_drop_press);
    app.add_observer(cache_dash_press);
    app.add_systems(PostStartup, attach_demo_actions);
    app.add_systems(
        FixedUpdate,
        reset_default_intent.in_set(DemoFixedSystems::DriveIntent),
    );
    app.run()
}

fn attach_demo_actions(mut commands: Commands, player: Single<Entity, With<DemoPlayer>>) {
    commands.entity(*player).insert(actions!(DemoPlayer[
        (
            Action::<MoveAction>::new(),
            Bindings::spawn((Cardinal::wasd_keys(), Axial::left_stick())),
        ),
        (
            Action::<JumpAction>::new(),
            InputPress::default(),
            bindings![KeyCode::Space, GamepadButton::South],
        ),
        (
            Action::<DropAction>::new(),
            InputPress::default(),
            bindings![KeyCode::KeyS, KeyCode::ArrowDown, GamepadButton::DPadDown],
        ),
        (
            Action::<DashAction>::new(),
            InputPress::default(),
            bindings![KeyCode::ShiftLeft, KeyCode::ShiftRight, GamepadButton::West],
        ),
    ]));
}

fn reset_default_intent(mut intent: Single<&mut PlatformerMovementIntent, With<DemoPlayer>>) {
    intent.move_axis = 0.0;
}

fn cache_move_axis(
    trigger: On<Fire<MoveAction>>,
    mut query: Query<&mut PlatformerMovementIntent, With<DemoPlayer>>,
) {
    if let Ok(mut intent) = query.get_mut(trigger.context) {
        intent.move_axis = trigger.value.x;
    }
}

fn clear_move_axis_on_cancel(
    trigger: On<InputCancel<MoveAction>>,
    mut query: Query<&mut PlatformerMovementIntent, With<DemoPlayer>>,
) {
    if let Ok(mut intent) = query.get_mut(trigger.context) {
        intent.move_axis = 0.0;
    }
}

fn clear_move_axis_on_complete(
    trigger: On<Complete<MoveAction>>,
    mut query: Query<&mut PlatformerMovementIntent, With<DemoPlayer>>,
) {
    if let Ok(mut intent) = query.get_mut(trigger.context) {
        intent.move_axis = 0.0;
    }
}

fn cache_jump_press(
    trigger: On<Start<JumpAction>>,
    mut query: Query<&mut PlatformerMovementIntent, With<DemoPlayer>>,
) {
    if let Ok(mut intent) = query.get_mut(trigger.context) {
        intent.jump_pressed = true;
    }
}

fn cache_jump_hold(
    trigger: On<Fire<JumpAction>>,
    mut query: Query<&mut PlatformerMovementIntent, With<DemoPlayer>>,
) {
    if let Ok(mut intent) = query.get_mut(trigger.context) {
        intent.jump_held = trigger.value;
    }
}

fn clear_jump_hold_on_cancel(
    trigger: On<InputCancel<JumpAction>>,
    mut query: Query<&mut PlatformerMovementIntent, With<DemoPlayer>>,
) {
    if let Ok(mut intent) = query.get_mut(trigger.context) {
        intent.jump_held = false;
    }
}

fn clear_jump_hold_on_complete(
    trigger: On<Complete<JumpAction>>,
    mut query: Query<&mut PlatformerMovementIntent, With<DemoPlayer>>,
) {
    if let Ok(mut intent) = query.get_mut(trigger.context) {
        intent.jump_held = false;
    }
}

fn cache_drop_press(
    trigger: On<Start<DropAction>>,
    mut query: Query<&mut PlatformerMovementIntent, With<DemoPlayer>>,
) {
    if let Ok(mut intent) = query.get_mut(trigger.context) {
        let _ = trigger;
        intent.drop_pressed = true;
    }
}

fn cache_dash_press(
    trigger: On<Start<DashAction>>,
    mut query: Query<&mut PlatformerDashIntent, With<DemoPlayer>>,
) {
    if let Ok(mut intent) = query.get_mut(trigger.context) {
        let _ = trigger;
        intent.pressed = true;
    }
}
