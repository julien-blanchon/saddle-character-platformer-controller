use saddle_character_platformer_controller_example_support as support;

use bevy::app::AppExit;
use bevy::prelude::*;
use support::{DemoFixedSystems, DemoScene};

fn main() -> AppExit {
    let mut app = App::new();
    support::configure_demo_app(&mut app, DemoScene::WallJumps, false);
    support::install_pane(&mut app);
    app.add_systems(
        FixedUpdate,
        support::drive_keyboard_intent.in_set(DemoFixedSystems::DriveIntent),
    );
    app.run()
}
