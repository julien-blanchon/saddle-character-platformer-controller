use bevy::prelude::*;

#[derive(Resource, Default, Debug)]
pub struct PlatformerControllerRuntime(pub bool);

pub fn activate_runtime(mut runtime: ResMut<PlatformerControllerRuntime>) {
    runtime.0 = true;
}

pub fn deactivate_runtime(mut runtime: ResMut<PlatformerControllerRuntime>) {
    runtime.0 = false;
}

pub fn runtime_is_active(runtime: Res<PlatformerControllerRuntime>) -> bool {
    runtime.0
}
