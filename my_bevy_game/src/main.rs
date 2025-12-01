use bevy::prelude::*;

// Declare the module (tells Rust to look for hello_plugin.rs)
mod hello_plugin;

// Bring the HelloPlugin struct into scope
use hello_plugin::HelloPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(HelloPlugin)
        .run();
}
